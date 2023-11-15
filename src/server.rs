use std::collections::HashMap;
use std::sync::Arc;
use std::vec;
use std::net::{SocketAddr, ToSocketAddrs};

use tokio::sync::Mutex;
use tokio::net::UdpSocket;

extern crate lazy_static;
use lazy_static::lazy_static;

struct ElectionData{
    load: u32,
    server_address: SocketAddr,
}

lazy_static! {
    static ref SERVER_ADDRESSES: Mutex<Vec<SocketAddr>> = {
        let mut vec = Vec::new();
        vec.push("127.0.0.1:8082".to_socket_addrs().unwrap().next().unwrap());
        vec.push("127.0.0.1:8084".to_socket_addrs().unwrap().next().unwrap());
        Mutex::new(vec)
    };

    static ref LOAD: Mutex<u32> = {
        let load: u32 = 0;
        Mutex::new(load)
    };

    static ref REQUEST_DATA_MAP: Mutex<HashMap<u32, Vec<ElectionData>>> = {
        let map = HashMap::new();
        Mutex::new(map)
    };
}

async fn read_request(socket: &UdpSocket) -> Result<(SocketAddr, Vec<u8>), Box<dyn std::error::Error>> {
    let mut buffer = vec![0; 1024];
    let (length, sender_address) = socket
        .recv_from(&mut buffer)
        .await?;

    Ok((sender_address, buffer[0..length].to_vec()))
}

async fn send_response(socket: &UdpSocket, dest_addr: &SocketAddr, data: &Vec<u8>) -> Result<(), Box<dyn std::error::Error>>{
    socket
        .send_to(&data, dest_addr)
        .await?;

    Ok(())
}

async fn handle_client(clients_socket: &UdpSocket, servers_socket: &UdpSocket) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let (client_address, data) = read_request(&clients_socket).await?;
        let operation_flag = data[0];
        let pay_load = data[1..9].to_vec();
        match operation_flag {
            0 => {
                println!("Received a request to start an election");
                let req_no = u32::from_ne_bytes(pay_load.as_slice().try_into().unwrap());
                init_election(&servers_socket, &client_address, &req_no).await?
            },
            _ => {
                let new_result = process_request(&operation_flag, &pay_load).await?;
                println!("Result is: {}", new_result);
                send_response(&clients_socket, &client_address, &new_result.to_ne_bytes().to_vec()).await?;
            }
        }
    }
}

async fn process_request(operation_flag: &u8, pay_load: &Vec<u8>) -> Result<i64, Box<dyn std::error::Error>> {
    println!("flag is: {}", operation_flag);
    match operation_flag {
        1 => {
            let number = i64::from_ne_bytes(pay_load.as_slice().try_into().unwrap());
            Ok(number.checked_add(1).unwrap_or(i64::MAX))
        }, // Increment with overflow handling
        2 => {
            let number = i64::from_ne_bytes(pay_load.as_slice().try_into().unwrap());
            Ok(number.checked_sub(1).unwrap_or(i64::MIN))
        }, // Decrement with overflow handling
        _ => {
            eprintln!("Invalid operation flag received from client");
            Ok(-1)
        }
    }
}

async fn init_election(servers_socket: &UdpSocket, client_address: &SocketAddr, req_no: &u32) -> Result<(), Box<dyn std::error::Error>>{
    println!("Election really started");
    let election_data_map = &mut *REQUEST_DATA_MAP.lock().await;
    let servers_addresses = SERVER_ADDRESSES.lock().await.clone();
    let mut received_count = 0;
    
    let req_no_bytes = req_no.to_be_bytes().to_vec();
    let load_bytes = LOAD.lock().await.to_be_bytes().to_vec();
    let mut election_data = Vec::new();
    election_data.extend(req_no_bytes);
    election_data.extend(load_bytes);

    for server_address in servers_addresses.as_slice() {
        send_response(servers_socket, server_address, &election_data).await?;   
    }

    while received_count < servers_addresses.len() {
        let (sender_addr, data) = read_request(servers_socket).await?;
        let req_no = u32::from_ne_bytes(data[0..4].try_into().unwrap());
        let load = u32::from_ne_bytes(data[4..8].try_into().unwrap());
        
        let entry = election_data_map
            .entry(req_no)
            .or_insert(Vec::new());
        entry.push(ElectionData{ load, server_address: sender_addr });
        
        received_count += 1;
        println!("request number: {} load:{} server address: {}", req_no, load, sender_addr);
    }
    
    let mut least_load = *LOAD.lock().await;
    let mut least_load_addr = servers_socket.local_addr().unwrap();

    for election_data in election_data_map.get(req_no).unwrap() {
        if election_data.load < least_load {
            least_load = election_data.load;
            least_load_addr = election_data.server_address;
        } 
        else if election_data.load == least_load {
            if election_data.server_address < least_load_addr {
                least_load = election_data.load;
                least_load_addr = election_data.server_address;
            }
        }   
    }

    if least_load_addr == servers_socket.local_addr().unwrap() {
        let response = "I can take your request".as_bytes().to_vec();
        send_response(&servers_socket, &client_address, &response).await?;
    }
    else {
        println!("Leader is: {}", least_load_addr);
    }
    Ok(())
}

async fn handle_server(servers_socket: &UdpSocket) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let mut buffer = vec![0; 1024];
        let (length, sender_address) = servers_socket
            .recv_from(&mut buffer)
            .await?;

        let (sender_addr, data) = read_request(servers_socket).await?;
        let req_no = u32::from_ne_bytes(data[0..4].try_into().unwrap());
        let load = u32::from_ne_bytes(data[4..8].try_into().unwrap());
        let election_data_map = &mut *REQUEST_DATA_MAP.lock().await;
        let entry = election_data_map
            .entry(req_no)
            .or_insert(Vec::new());
        entry.push(ElectionData{ load, server_address: sender_addr });
        
    }
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let servers_socket = UdpSocket::bind("127.0.0.1:8080").await?;
    let clients_socket = UdpSocket::bind("127.0.0.1:8081").await?;
    
    let servers_socket_arc = Arc::new(servers_socket);    
    let clients_socket_arc = Arc::new(clients_socket);    

    let num_threads = 3; // Number of threads to handle clients
    let mut handles = Vec::new();

    for i in 0..num_threads {
        if i == 0
        {
            let s_socket_clone = servers_socket_arc.clone();
            let handle = tokio::spawn( async move {
                handle_server(
                    &s_socket_clone
                ).await.unwrap();
            });
    
            handles.push(handle);

        }
        let c_socket_clone = clients_socket_arc.clone();
      
        
    }


    // Block the main thread to keep the program running
    for handle in handles {
        handle.await?;
    }

    Ok(())
}

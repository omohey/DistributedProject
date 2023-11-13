use std::collections::HashMap;
use std::sync::Arc;
use std::{thread, vec};
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
        vec.push("127.0.0.1:8081".to_socket_addrs().unwrap().next().unwrap());
        vec.push("127.0.0.1:8082".to_socket_addrs().unwrap().next().unwrap());
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

async fn handle_client(socket: &UdpSocket) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let (client_address, data) = read_request(&socket).await?;
        let operation_flag = data[0];
        let pay_load = data[1..9].to_vec();
        let new_result: i64 = process_request(&socket, &operation_flag, &pay_load).await?;
        println!("Result is: {}", new_result);
        send_response(&socket, &client_address, &new_result.to_ne_bytes().to_vec()).await?;
    }
}

async fn process_request(socket: &UdpSocket, operation_flag: &u8, pay_load: &Vec<u8>) -> Result<i64, Box<dyn std::error::Error>> {
    println!("flag is: {}", operation_flag);
    match operation_flag {
        0 => {
            let number = i64::from_ne_bytes(pay_load.as_slice().try_into().unwrap());
            Ok(number.checked_add(1).unwrap_or(i64::MAX))
        }, // Increment with overflow handling
        1 => {
            let number = i64::from_ne_bytes(pay_load.as_slice().try_into().unwrap());
            Ok(number.checked_sub(1).unwrap_or(i64::MIN))
        }, // Decrement with overflow handling
        2 => {
            println!("Election started");
            let req_no = u32::from_ne_bytes(pay_load.as_slice().try_into().unwrap());
            init_election(socket, &req_no).await?;
            Ok(-1)
        },
        _ => {
            eprintln!("Invalid operation flag received from client");
            Ok(-2)
        }
    }
}

async fn init_election(socket: &UdpSocket, req_no: &u32) -> Result<(), Box<dyn std::error::Error>>{
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
        send_response(socket, server_address, &election_data).await?;   
    }

    while received_count != 2 {
        let (sender_addr, data) = read_request(socket).await?;
        if servers_addresses.contains(&sender_addr) {
            let req_no = u32::from_ne_bytes(data[0..4].try_into().unwrap());
            let load = u32::from_ne_bytes(data[4..8].try_into().unwrap());
            
            let entry = election_data_map
                .entry(req_no)
                .or_insert(Vec::new());
            entry.push(ElectionData { load, server_address: sender_addr });
            
            received_count += 1;
            println!("request number: {} load:{} server address: {}", req_no, load, sender_addr);
        }

        // TODO::handle if you receive from a client instead during an election
    }
    
    let mut least_load = *LOAD.lock().await;
    let mut least_load_addr = socket.local_addr().unwrap();

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

    if least_load_addr == socket.local_addr().unwrap() {
        println!("I am the leader");
    }
    else {
        println!("Leader is: {}", least_load_addr);
    }
    Ok(())
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let socket = UdpSocket::bind("127.0.0.1:8080").await?;
    let socket_arc = Arc::new(socket);    
    println!("Server listening on 127.0.0.1:8080...");
    
    let num_threads = 4; // Number of threads to handle client
    for _ in 0..num_threads {
        let socket_clone =Arc::clone(&socket_arc);
        tokio::spawn(async move {
            let _ = handle_client(&socket_clone).await;
        });
    }

    // Block the main thread to keep the program running
    for _ in 0..num_threads {
        thread::park();
    }

    Ok(())
}

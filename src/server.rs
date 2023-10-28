extern crate lazy_static;
use std::sync::{Arc, Mutex};
use std::thread;
use std::collections::HashMap;
use std::net::{SocketAddr, ToSocketAddrs};
use tokio::net::UdpSocket;
use lazy_static::lazy_static;


struct ElectionData{
    load: u32,
    server_address: SocketAddr,
}

lazy_static! {
    static ref REQUEST_DATA_MAP: Mutex<HashMap<u32, Vec<ElectionData>>> = {
        let map = HashMap::new();
        Mutex::new(map)
    };
    
    static ref SERVER_ADDRESSES: Mutex<Vec<SocketAddr>> = {
        let vec = Vec::new();
        vec.push("127.0.0.1:8081".to_socket_addrs().unwrap().next().unwrap());
        vec.push("127.0.0.1:8082".to_socket_addrs().unwrap().next().unwrap());
        Mutex::new(vec)
    };

    static ref CUR_SOCKET: Mutex<UdpSocket> = {
        let socket = match UdpSocket::bind("127.0.0.1:8080").await {
            Ok(s) => s,
            Err(e) => panic!("Failed to bind socket {}", e);
        };
        Mutex::new(socket);
    };

    static ref LOAD: Mutex<u32> = {
        let load: u32 = 0;
        Mutex::new(load)
    };
}

async fn read_request(socket: &UdpSocket) -> Result<(SocketAddr, Vec<u8>), Box<dyn std::error::Error>> {
    let mut buffer = vec![0; 1024];
    let (length, sender_address) = socket
        .recv_from(&mut buffer)
        .await?;
        // .expect("Failed to receive data from client");

    Ok((sender_address, buffer[0..length].to_vec()))
}

async fn send_response(socket: &UdpSocket, dest_addr: &SocketAddr, data: &Vec<u8>) -> Result<(), Box<dyn std::error::Error>>{
    socket
        .send_to(&data, dest_addr)
        .await?;
        // .expect(&format!("Failed to send response to {:?}", dest_addr.to_string()));
    Ok(())
}

fn process_request(socket: &UdpSocket, server_addresses: &Vec<SocketAddr>, req_no: &u32, load: &u32, operation_flag: &u8, number: &i64) -> i64 {
// fn process_request(operation_flag: &u8, number: &i64) -> i64 {
    match operation_flag {
        0 => number.checked_add(1).unwrap_or(i64::MAX), // Increment with overflow handling
        1 => number.checked_sub(1).unwrap_or(i64::MIN), // Decrement with overflow handling
        2 => {
            init_election(socket, server_addresses, load, req_no);
            -1
        },
        _ => {
            eprintln!("Invalid operation flag received from client");
            -2
        }
    }
}

async fn init_election(socket: &UdpSocket, server_addresses: &Vec<SocketAddr>, cur_load: &u32, req_no: &u32) -> Result<(), Box<dyn std::error::Error>>{
    let mut election_data_map = REQUEST_DATA_MAP.lock().unwrap();
    let mut election_data = Vec::new();
    let req_no_bytes = req_no.to_be_bytes().to_vec();
    let load_bytes = cur_load.to_be_bytes().to_vec();
    election_data.extend(req_no_bytes);
    election_data.extend(load_bytes);
    for server_address in server_addresses {
        send_response(socket, server_address, &election_data).await?;   
    }
    for _ in 0 .. server_addresses.len() {
        let (sender_addr, data) = read_request(socket).await?;
        if server_addresses.contains(&sender_addr) {
            let req_no = u32::from_be_bytes(data[0..3].try_into().unwrap());
            let load = u32::from_be_bytes(data[4..7].try_into().unwrap());
            let entry = election_data_map
                .entry(req_no)
                .or_insert(Vec::new());
            entry.push(ElectionData { load: load, server_address: sender_addr });
            println!("request number: {} load:{} server address: {}", req_no, load, sender_addr);
        }
    }
    
    let mut least_load = *cur_load;
    let mut least_load_addr = socket.local_addr().unwrap();

    for election_data in election_data_map.get(req_no).unwrap() {
        if election_data.load < least_load {
            least_load = election_data.load;
            least_load_addr = election_data.server_address.clone();
        } 
        else if election_data.load == least_load {
            if election_data.server_address < least_load_addr {
                least_load = election_data.load;
                least_load_addr = election_data.server_address.clone();
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

async fn handle_client(socket: UdpSocket, server_addresses: &Vec<SocketAddr>, ) -> Result<(), Box<dyn std::error::Error>> {
// async fn handle_client(socket: &UdpSocket) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let (client_address, data) = read_request(&socket).await?;
        let operation_flag = data[0];
        result = process_request(&socket, server_addresses, req_no, load, &operation_flag, number)
        println!("Result is: {}", new_result);
        send_response(&socket, &client_address, &new_result.to_be_bytes().to_vec()).await?;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let socket = UdpSocket::bind("127.0.0.1:8080").await?;

    let socket = Arc::new(socket);
    
    println!("Server listening on 127.0.0.1:8080...");
    
    let num_threads = 4; // Number of threads to handle clients
    
    // for _ in 0..num_threads {
    //     let socket_clone =Arc::clone(&socket);
    //     tokio::spawn(async move {
    //         if let Err(err) = handle_client(&socket_clone).await {
    //             eprintln!("Error in handle_client: {}", err);
    //         }
    //     });
    // }
    

    // for _ in 0..num_threads {
    //     let cloned_socket = Arc::clone(&socket_clone);
    //     tokio::spawn(handle_client(cloned_socket));
    // } 

    // Block the main thread to keep the program running
    for _ in 0..num_threads {
        thread::park();
    }

    Ok(())
}

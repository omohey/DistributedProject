use std::collections::HashMap;
use std::sync::Arc;
use std::vec;
use std::net::{SocketAddr, ToSocketAddrs};
use std::net::Ipv4Addr;
use std::net::IpAddr;
use std::fs::File;
use std::io::{Read, Write};


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
        vec.push("127.0.0.1:8080".to_socket_addrs().unwrap().next().unwrap());
        vec.push("127.0.0.1:8082".to_socket_addrs().unwrap().next().unwrap());
        Mutex::new(vec)
    };

    static ref LOAD: Mutex<u32> = {
        let load: u32 = 0;
        Mutex::new(load)
    };

    static ref REQUEST_DATA_MAP: Mutex<HashMap<SocketAddr, Vec<ElectionData>>> = {
        let map = HashMap::new();
        Mutex::new(map)
    };
}

async fn read_request(socket: &UdpSocket) -> Result<(SocketAddr, Vec<u8>), Box<dyn std::error::Error>> {
    const INITIAL_BUFFER_SIZE: usize = 65000; // Initial buffer size, change as needed

    let mut buffer = vec![0u8; INITIAL_BUFFER_SIZE]; // Create a buffer with an initial size

    let (length, sender_address) = socket.recv_from(&mut buffer).await?;

    buffer.resize(length, 0); // Resize the buffer to fit the received data

    Ok((sender_address, buffer))
    // Ok((sender_address, buffer[0..length].to_vec()))
}

async fn send_response(socket: &UdpSocket, dest_addr: &SocketAddr, data: &Vec<u8>) -> Result<(), Box<dyn std::error::Error>>{
    socket
        .send_to(&data, dest_addr)
        .await?;

    Ok(())
}

async fn handle_client(clients_socket: &UdpSocket, servers_socket: &UdpSocket) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        println!("HERE1");
        let (client_address, data) = read_request(&clients_socket).await?;
        println!("HERE2");
        let operation_flag :u8 = data[0];
        println!("flag is: {}", operation_flag);
        // let pay_load = data[1..5].to_vec();
        match operation_flag {
            0 => {
                println!("Received a request to start an election");
                let servers_addresses = SERVER_ADDRESSES.lock().await.clone();
                let myload : u32= *LOAD.lock().await;
                // need to delete data for key with client address from REQUEST_DATA_MAP
                let election_data_map = &mut *REQUEST_DATA_MAP.lock().await;
                election_data_map.remove(&client_address);
                
                let mut buffer = myload.to_be_bytes().to_vec();
                // add client address with port number to buffer in 6 bytes
                let client_address_bytes: [u8; 4];
                let port_bytes: [u8; 2]; 
                if let SocketAddr::V4(v4) = client_address {
                    client_address_bytes = v4.ip().octets();
                    port_bytes = v4.port().to_be_bytes();
                }
                else {
                    client_address_bytes = [0; 4];
                    port_bytes = [0; 2];
                }
                buffer.extend(client_address_bytes.iter());
                buffer.extend(port_bytes.iter());
                println!("HERE3");
                for server_address in servers_addresses.as_slice() {
                    send_response(servers_socket, server_address, &buffer).await?;   
                }   
                println!("HERE4");    
            },
            1 => {
                // remove data[0] and save the rest as an image
                let mut buffer = data[1..].to_vec();

                println!("Received an image from client");
                let mut image_bytes = Vec::new();
                let mut f = File::open("./src/default.jpeg")?;
                f.read_to_end(&mut image_bytes)?;

                // append to default the image received from client
                image_bytes.append(&mut buffer);
                

                // send the image to the client
                send_response(&clients_socket, &client_address, &image_bytes).await?;



            },
            _ => {
                // let new_result = process_request(&operation_flag, &pay_load).await?;
                // println!("Result is: {}", new_result);
                // send_response(&clients_socket, &client_address, &new_result.to_ne_bytes().to_vec()).await?;
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

// async fn init_election(servers_socket: &UdpSocket, client_address: &SocketAddr, req_no: &u32) -> Result<(), Box<dyn std::error::Error>>{
//     println!("Election really started");
//     let election_data_map = &mut *REQUEST_DATA_MAP.lock().await;
//     let servers_addresses = SERVER_ADDRESSES.lock().await.clone();
//     let mut received_count = 0;
    
//     let req_no_bytes = req_no.to_be_bytes().to_vec();
//     let load_bytes = LOAD.lock().await.to_be_bytes().to_vec();
//     let mut election_data = Vec::new();
//     election_data.extend(req_no_bytes);
//     election_data.extend(load_bytes);

//     for server_address in servers_addresses.as_slice() {
//         send_response(servers_socket, server_address, &election_data).await?;   
//     }

//     while received_count < servers_addresses.len() {
//         let (sender_addr, data) = read_request(servers_socket).await?;
//         let req_no = u32::from_ne_bytes(data[0..4].try_into().unwrap());
//         let load = u32::from_ne_bytes(data[4..8].try_into().unwrap());
        
//         let entry = election_data_map
//             .entry(req_no)
//             .or_insert(Vec::new());
//         entry.push(ElectionData{ load, server_address: sender_addr });
        
//         received_count += 1;
//         println!("request number: {} load:{} server address: {}", req_no, load, sender_addr);
//     }
    
//     let mut least_load = *LOAD.lock().await;
//     let mut least_load_addr = servers_socket.local_addr().unwrap();

//     for election_data in election_data_map.get(req_no).unwrap() {
//         if election_data.load < least_load {
//             least_load = election_data.load;
//             least_load_addr = election_data.server_address;
//         } 
//         else if election_data.load == least_load {
//             if election_data.server_address < least_load_addr {
//                 least_load = election_data.load;
//                 least_load_addr = election_data.server_address;
//             }
//         }   
//     }

//     if least_load_addr == servers_socket.local_addr().unwrap() {
//         let response = "I can take your request".as_bytes().to_vec();
//         send_response(&servers_socket, &client_address, &response).await?;
//     }
//     else {
//         println!("Leader is: {}", least_load_addr);
//     }
//     Ok(())
// }

async fn handle_server(servers_socket: &UdpSocket, client_socket: &UdpSocket) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let mut buffer = vec![0; 1024];
        let (length, sender_address) = servers_socket
            .recv_from(&mut buffer)
            .await?;

        let load_no = u32::from_ne_bytes([buffer[3], buffer[2], buffer[1], buffer[0]]);
        // extract client address from buffer
        let client_addr = SocketAddr::new(Ipv4Addr::new(buffer[4], buffer[5], buffer[6], buffer[7]).into(), u16::from_ne_bytes([buffer[9], buffer[8]]));
        println!("Load of sender {} is {} for client {}", sender_address, load_no, client_addr);
        let election_data_map = &mut *REQUEST_DATA_MAP.lock().await;
        let entry = election_data_map
            .entry(client_addr)
            .or_insert(Vec::new());
        entry.push(ElectionData{ load: load_no, server_address: sender_address });
        let server_len = SERVER_ADDRESSES.lock().await.len();
        println!("Server length is: {}", server_len);
        if entry.len() == server_len {
            let my_load = &mut *LOAD.lock().await;

            let mut least_load = *my_load;
            let mut least_load_addr = client_socket.local_addr().unwrap();
            println!("Entry length is: {}", entry.len());
        
            for i in 0..entry.len(){
                if entry[i].load < least_load {
                    least_load = entry[i].load;
                    least_load_addr = entry[i].server_address;
                }
                else {
                    if entry[i].load == least_load {
                        if entry[i].server_address < least_load_addr {
                            least_load = entry[i].load;
                            least_load_addr = entry[i].server_address;
                        }
                    }
                }
                
            }
            if least_load_addr == client_socket.local_addr().unwrap() {
                println!("I am the leader sending to client with address: {}", client_addr);
                let response = "Hello, client! I am the leader please send your request to me";
                *my_load += 1;
                client_socket.send_to(response.as_bytes(), client_addr).await?;
            }
            else {
                println!("Leader is: {}", least_load_addr);
            }
        }
        
    }
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let servers_socket = UdpSocket::bind("127.0.0.1:8084").await?;
    let clients_socket = UdpSocket::bind("127.0.0.1:8085").await?;

    println!("Server started at {}", servers_socket.local_addr().unwrap());

    
    let servers_socket_arc = Arc::new(servers_socket);    
    let clients_socket_arc = Arc::new(clients_socket);    

    let num_threads = 3; // Number of threads to handle clients
    let mut handles = Vec::new();

    for i in 0..num_threads {
        if i == 0
        {
            let s_socket_clone = servers_socket_arc.clone();
            let c_socket_clone = clients_socket_arc.clone();
            let handle = tokio::spawn( async move {
                handle_server(
                    &s_socket_clone,
                    &c_socket_clone
                ).await.unwrap();
            });
    
            handles.push(handle);

        }
        else if i == 1
        {
            let s_socket_clone = servers_socket_arc.clone();
            let c_socket_clone = clients_socket_arc.clone();
            let handle = tokio::spawn( async move {
                handle_client(
                    &c_socket_clone,
                    &s_socket_clone
                ).await.unwrap();
            });
    
            handles.push(handle);
        }
        else{
        }
            
    }


    // Block the main thread to keep the program running
    for handle in handles {
        handle.await?;
    }

    Ok(())
}

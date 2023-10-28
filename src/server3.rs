// server.rs

use std::net::UdpSocket;
use std::net::{SocketAddr, ToSocketAddrs};
use std::collections::HashMap;

#[derive(Copy, Clone)]
struct ElectionData{
    load: i32,
    server_address: SocketAddr,
}




fn main() {
    // There will be 3 servers in total, each listening on a different port. Save the ip and port of each server in a vector
    let servers = vec!["127.0.0.1:8080", "127.0.0.1:8081", "127.0.0.1:8082"];
    let myip = servers[2];
    let mut serversAddr = Vec::new();
    serversAddr.push("127.0.0.1:8080".to_socket_addrs().unwrap().next().unwrap());
    serversAddr.push("127.0.0.1:8081".to_socket_addrs().unwrap().next().unwrap());
    let mut my_load: i32 = 2;

    let mut election_data: HashMap<i32, Vec<ElectionData>> = HashMap::new(); 
    let mut ClientAddresses : HashMap<i32, SocketAddr> = HashMap::new();


    let socket = UdpSocket::bind("127.0.0.1:8082").expect("Failed to bind to address");
    println!("Server listening on 127.0.0.1:8082");

    let mut buffer = [0; 1024];
    loop {
        let (size, source) = socket.recv_from(&mut buffer).expect("Failed to receive data");
        // let request = String::from_utf8_lossy(&buffer[..size]);
        // Check if the source is one of the other servers
        if !servers.contains(&source.to_string().as_str()) {
            // println!("Received request: {} from {}", request, source);
            // let response = "Hello, client!";
            // socket.send_to(response.as_bytes(), source).expect("Failed to send response");
            if size == 4 {
                let received_integer = i32::from_ne_bytes(buffer[0..4].try_into().unwrap());
                println!("Received integer: {}", received_integer);
                ClientAddresses.insert(received_integer, source);
                // let mut election_data = Vec::new();
                // let req_no = request.parse::<u32>().unwrap();
                // let req_no_bytes = req_no.to_be_bytes().to_vec();
                // let load_bytes = my_load.to_be_bytes().to_vec();
                // election_data.extend(req_no_bytes);
                // election_data.extend(load_bytes);
                let send_buffer = [
                    received_integer.to_ne_bytes(),
                    my_load.to_ne_bytes(),
                ]
                .concat();
                for server in &serversAddr {
                    socket.send_to(&send_buffer, server).expect("Failed to send response");
                }
                continue;
            }
            else {
                println!("Received invalid number of bytes for client: {}", size);
                continue;
            }
        }
        else
        {
            if size == 8 {
                let req_no = i32::from_ne_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
                let load_no = i32::from_ne_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]);
                println!("Received integers: ({}, {})", req_no, load_no);
                let entry = election_data
                .entry(req_no)
                .or_insert(Vec::new());
                entry.push(ElectionData { load: load_no, server_address: source });
                if entry.len() == 2 {
                    let mut least_load = my_load;
                    let mut least_load_addr = myip.to_socket_addrs().unwrap().next().unwrap();
                
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
                    // for data in entry {
                    //     if data.load < least_load {
                    //         least_load = data.load;
                    //         least_load_addr = data.server_address.clone();
                    //     } 
                    //     else if data.load == least_load {
                    //         if data.server_address < least_load_addr {
                    //             least_load = data.load;
                    //             least_load_addr = data.server_address.clone();
                    //         }
                    //     }   
                    // }
                    if least_load_addr == socket.local_addr().unwrap() {
                        let client_addr = ClientAddresses.get(&req_no).unwrap();
                        let response = "Hello, client! I am the leader please send your request to me";
                        my_load += 1;
                        socket.send_to(response.as_bytes(), client_addr).expect("Failed to send response");
                        // socket.send_to(, client_addr).expect("Failed to send response");
                        // socket.send_to(&send_buffer, server).expect("Failed to send response");

                    }
                    else {
                        println!("Leader is: {}", least_load_addr);
                    }



                    // let mut min_load = entry[0].load;
                    // let mut min_load_server = entry[0].server_address;
                    // for i in 1..3 {
                    //     if entry[i].load < min_load {
                    //         min_load = entry[i].load;
                    //         min_load_server = entry[i].server_address;
                    //     }
                    // }
                    // println!("The server with the minimum load is: {}", min_load_server);
                }
            } else {
                println!("Received invalid number of bytes for server: {}", size);
            }
            // let response = "Hello, server!";
            // socket.send_to(response.as_bytes(), source).expect("Failed to send response");
            continue;
        }
    }
}
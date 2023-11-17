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
        vec.push("127.0.0.1:8082".to_socket_addrs().unwrap().next().unwrap());
        vec.push("127.0.0.1:8084".to_socket_addrs().unwrap().next().unwrap());
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

    static ref IMAGES: Mutex<HashMap<SocketAddr, HashMap<u8, Vec<u8>> > > = {
        let map = HashMap::new();
        Mutex::new(map)
    };

    static ref DOWN: Mutex<bool> = {
        let down = false;
        Mutex::new(down)
    };
}

async fn read_request(socket: &UdpSocket) -> Result<(SocketAddr, Vec<u8>), Box<dyn std::error::Error>> {
    const INITIAL_BUFFER_SIZE: usize = 65300; // Initial buffer size, change as needed

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
        let down = &*DOWN.lock().await;
        let load = &mut *LOAD.lock().await;
        println!("HERE2");
        let operation_flag :u8 = data[0];
        println!("flag is: {}", operation_flag);
        if (*down) && (*load == 0) {
            println!("I am down");
            continue;
        }
        if (*down) && (operation_flag == 0){
            println!("I am down no election");
            continue;
        }
        
        // let pay_load = data[1..5].to_vec();
        match operation_flag {
            0 => {
                println!("Received a request to start an election");
                let servers_addresses = SERVER_ADDRESSES.lock().await.clone();
                println!("HERE7");
                let myload : u32= load.clone();
                println!("HERE6");
                // need to delete data for key with client address from REQUEST_DATA_MAP
                let election_data_map = &mut *REQUEST_DATA_MAP.lock().await;
                election_data_map.remove(&client_address);
                println!("HERE5");
                
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

                let frag_no :u8 = buffer[0];
                let no_frags :u8 = buffer[1];

                let images = &mut *IMAGES.lock().await;
                let image = images
                    .entry(client_address)
                    .or_insert(HashMap::new());
                image.insert(frag_no, buffer[2..].to_vec());


                if image.len() < no_frags as usize {
                    continue;
                }
                let mut image_bytes = Vec::new();
                for i in 0..no_frags {
                    image_bytes.extend_from_slice(&image.get(&i).unwrap());
                }


                images.remove(&client_address);


                println!("Received an image from client");
                let mut defualt = Vec::new();
                let mut f = File::open("./src/default.jpeg")?;
                f.read_to_end(&mut defualt)?;

                // append to default the image received from client
                defualt.extend_from_slice(&image_bytes); // kanet .append                

                let image_size = defualt.len();
                println!("Image size: {}", image_size);

                let mut no_frags = (image_size / 65000) as u8;
                if image_size % 65000 != 0 {
                    no_frags += 1;
                }

                for i in 0..no_frags {
                    let mut buffer = Vec::new();
                    let frag_no = i as u8;
                    buffer.push(frag_no);
                    buffer.push(no_frags);
                    let start : usize = i as usize * 65000 as usize ;
                    let mut end : usize= (i + 1) as usize * 65000 as usize;
                    if end > image_size {
                        end = image_size;
                    }
                    buffer.extend_from_slice(&defualt[start..end]);
                    send_response(&clients_socket, &client_address , &buffer).await?;
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
                
                *load -= 1;
                // send the image to the client
                // send_response(&clients_socket, &client_address, &defualt).await?;
            },
            _ => {
                // let new_result = process_request(&operation_flag, &pay_load).await?;
                // println!("Result is: {}", new_result);
                // send_response(&clients_socket, &client_address, &new_result.to_ne_bytes().to_vec()).await?;
            }
        }
    }
}

async fn fault_tolerance(servers_socket: &UdpSocket) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        // read from terminal
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim();
        // If character is 'd' then make the server down and if it is 'u' then make the server up
        if input == "d" {
            let down = &mut *DOWN.lock().await;
            *down = true;
            println!("I am now down");
            // send to all servers that i am down
            let servers_addresses = SERVER_ADDRESSES.lock().await.clone();

            let mut buffer = Vec::new();
            let x = "I am down".as_bytes();
            buffer.extend_from_slice(x);
            for server_address in servers_addresses.as_slice() {
                send_response(&servers_socket, server_address, &buffer).await?;   
            }

        }
        else if input == "u" {
            let down = &mut *DOWN.lock().await;
            if (*down){
                *down = false;
                println!("I am now up");

                let servers_addresses = SERVER_ADDRESSES.lock().await.clone();

                let mut buffer = Vec::new();
                let x = "I am up".as_bytes();
                buffer.extend_from_slice(x);
                for server_address in servers_addresses.as_slice() {
                    send_response(&servers_socket, server_address, &buffer).await?;   
                }
            }
            
        }
        else {
            println!("Invalid input");
        }
    }    
}


async fn handle_server(servers_socket: &UdpSocket, client_socket: &UdpSocket) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let mut buffer = vec![0; 1024];
        let (length, sender_address) = servers_socket
            .recv_from(&mut buffer)
            .await?;

        // if i am down do not respond
        let down = &*DOWN.lock().await;
        if *down {
            println!("I am downS");
            continue;
        }

        // if received "I am down" from another server remove them from SERVER_ADDRESSES
        let x = "I am down".as_bytes();
        if buffer[0..x.len()] == x[..] {
            let mut servers_addresses = SERVER_ADDRESSES.lock().await;
            let mut index = 0;
            for i in 0..servers_addresses.len() {
                if servers_addresses[i] == sender_address {
                    index = i;
                    break;
                }
            }
            servers_addresses.remove(index);
            println!("Server {} is down", sender_address);
            continue;
        }

        let y = "I am up".as_bytes();
        if buffer[0..y.len()] == y[..] {
            let servers_addresses = &mut *SERVER_ADDRESSES.lock().await;
            servers_addresses.push(sender_address.to_socket_addrs().unwrap().next().unwrap());
            println!("Server {} is up", sender_address);
            continue;
        }

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
    let servers_socket = UdpSocket::bind("127.0.0.1:8080").await?;
    let clients_socket = UdpSocket::bind("127.0.0.1:8081").await?;

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
        else if i == 2 {
            let s_socket_clone = servers_socket_arc.clone();
            let handle = tokio::spawn( async move {
                fault_tolerance(&s_socket_clone).await.unwrap();
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

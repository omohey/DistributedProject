use std::collections::HashMap;
use std::sync::Arc;
use std::vec;
use std::net::{SocketAddr, ToSocketAddrs};
use std::net::Ipv4Addr;
use std::net::IpAddr;
use std::fs::File;
use std::io::{Read, Write};
use std::thread;
use std::time::Duration;


use tokio::sync::Mutex;
use tokio::net::UdpSocket;
use sysinfo::{CpuExt, System, SystemExt};


extern crate lazy_static;
use lazy_static::lazy_static;

struct ElectionData{
    load: u32,
    server_address: SocketAddr,
}

struct DirectoryEntry{
    online: u8,
    clients_waiting: Vec<String>,
}

lazy_static! {
    static ref SERVER_ADDRESSES: Mutex<Vec<String>> = {
        let mut vec = Vec::new();
        vec.push("127.0.0.2:8080".to_string());
        vec.push("127.0.0.3:8080".to_string());
        Mutex::new(vec)
    };

    static ref LOAD: Mutex<u32> = {
        let load: u32 = 0;
        Mutex::new(load)
    };

    static ref CPU_LOAD: Mutex<u32> = {
        let load: u32 = 0;
        Mutex::new(load)
    };

    static ref REQUEST_DATA_MAP: Mutex<HashMap<String, Vec<ElectionData>>> = {
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

    static ref DIRECTORY: Mutex<HashMap<String, DirectoryEntry>> = {
        let map = HashMap::new();
        Mutex::new(map)
    };

    static ref NOTIFY: Mutex<bool> = {
        let notify = false;
        Mutex::new(notify)
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
        
        match operation_flag {
            0 => {  // election
                println!("Received a request to start an election");
                let servers_addresses = SERVER_ADDRESSES.lock().await.clone();
                println!("HERE7");
                let myload : u32= load.clone();

                let mut sys = System::new_all();

                // First we update all information of our `System` struct.
                sys.refresh_all();
            
                sys.refresh_cpu(); // Refreshing CPU information.
                let mut avgCpuUsage = 0.0;
                for cpu in sys.cpus() {
                    avgCpuUsage += cpu.cpu_usage();
                    println!("{}% ", cpu.cpu_usage());
                }
                avgCpuUsage /= sys.cpus().len() as f32;
                let cpuload : u32 = (avgCpuUsage * 100.0) as u32;
                // let global CPU_LOAD = cpuload;
                let loadglobal = &mut *CPU_LOAD.lock().await;
                *loadglobal = cpuload;

                let mut buffer = cpuload.to_be_bytes().to_vec();
                println!("HERE6");
                // need to delete data for key with client address from REQUEST_DATA_MAP
                let election_data_map = &mut *REQUEST_DATA_MAP.lock().await;
                election_data_map.remove(&client_address.to_string());
                println!("HERE5");
                
                // let mut buffer = myload.to_be_bytes().to_vec();
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
                    // send_response(servers_socket, server_address, &buffer).await?; 
                    servers_socket.send_to(&buffer, server_address.to_socket_addrs().unwrap().next().unwrap()).await?;
                }   
                println!("HERE4");    
            },
            1 => { // encrypt image
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
                defualt.extend_from_slice(&image_bytes);             

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
            },
            2 => {  // register as online
                // Add client address to directory
                let directory = &mut *DIRECTORY.lock().await;
                if directory.contains_key(&client_address.to_string()) {
                    let entry = directory
                        .get_mut(&client_address.to_string());
                    entry.unwrap().online = 1;
                    let entry = directory
                        .get_mut(&client_address.to_string());
                    let clients_waiting = &mut entry.unwrap().clients_waiting;
                    let no_clients = clients_waiting.len();
                    println!("No clients waiting: {}", no_clients);
                    if (no_clients == 0) {
                        continue;
                    }

                    // only the server with the least address will send the response
                    // get my address
                    let mut min = servers_socket.local_addr().unwrap();

                    for server_address in SERVER_ADDRESSES.lock().await.as_slice() {
                        if (*server_address).to_socket_addrs().unwrap().next().unwrap() < min {
                            min = (*server_address).to_socket_addrs().unwrap().next().unwrap();
                        }
                    }

                    println!("Min address is: {}", min);

                    if min != servers_socket.local_addr().unwrap() {
                        continue;
                    }

                    for i in 0..no_clients {
                        let client_addr = clients_waiting[i].clone();

                        // send to client_addr that client_address is online
                        let input: u8 = 3;
                        let mut buffer = Vec::new();
                        buffer.push(input);
                        println!("sending to client with address: {}", client_addr);
                        let message = "The client with address ".to_string() + &client_address.to_string() + " is online";
                        let message_bytes = message.as_bytes();
                        buffer.extend_from_slice(&message_bytes);
                        let client_sock = client_addr.to_socket_addrs().unwrap().next().unwrap();
                        let cl_ip = client_sock.ip();
                        let cl_port = client_sock.port() + 1;
                        let cl_addr = SocketAddr::new(cl_ip, cl_port);
                        send_response(&clients_socket, &cl_addr , &buffer).await?;
                    }
                    
                }
                else {
                    directory.insert(client_address.to_string(), DirectoryEntry{ online: 1, clients_waiting: Vec::new() });
                }
                // print directory for debugging
                for (key, value) in directory.iter() {
                    println!("{}: {}", key, value.online);
                }

            },
            3 => {  // register as offline
                let directory = &mut *DIRECTORY.lock().await;
                if directory.contains_key(&client_address.to_string()) {
                    let entry = directory
                        .get_mut(&client_address.to_string());
                    entry.unwrap().online = 0;
                }
            },
            4 => { // send online clients
                let directory = &mut *DIRECTORY.lock().await;
                let mut buffer = Vec::new();
                let mut online_clients = Vec::new();

                for (key, value) in directory.iter() {
                    if value.online == 1 {
                        online_clients.push(key.clone());
                    }
                }
                
                let no_clients = online_clients.len();
                buffer.push(no_clients as u8);

                for i in 0..no_clients {
                    let client_addr = online_clients[i].clone();
                    let client_addr_bytes: [u8; 4];
                    let port_bytes: [u8; 2]; 
                    if let SocketAddr::V4(v4) = client_addr.parse::<SocketAddr>().unwrap() {
                        client_addr_bytes = v4.ip().octets();
                        port_bytes = v4.port().to_be_bytes();
                    }
                    else {
                        client_addr_bytes = [0; 4];
                        port_bytes = [0; 2];
                    }
                    buffer.extend(client_addr_bytes.iter());
                    buffer.extend(port_bytes.iter());
                }
                send_response(&clients_socket, &client_address , &buffer).await?;
                // decrement load
                *load -= 1;
            },
            5 => { // add client to waitlist 

                // get client address from buffer
                let client_addr = SocketAddr::new(Ipv4Addr::new(data[1], data[2], data[3], data[4]).into(), u16::from_ne_bytes([data[6], data[5]]));
                let directory = &mut *DIRECTORY.lock().await;
                if directory.contains_key(&client_addr.to_string()) {
                    let entry = directory
                        .get_mut(&client_addr.to_string());
                    entry.unwrap().clients_waiting.push(client_address.to_string());
                    println!("Added client {} to waitlist of client {}", client_address, client_addr);
                }
                else {
                    directory.insert(client_addr.to_string(), DirectoryEntry{ online: 0, clients_waiting: vec![client_address.to_string()] });
                }

            }
            _ => {
                println!("Invalid operation flag");
            }
        }
    }
}

async fn fault_tolerance(servers_socket: &UdpSocket) -> Result<(), Box<dyn std::error::Error>> {
    let mut down_old = false;
    loop {
        // sleep for 10 miliseconds
        let down = &*DOWN.lock().await;
        if *down == down_old {
            continue;
        }
        else {
            down_old = *down;
            if *down == false {
                println!("I am now up2");

                let servers_addresses = SERVER_ADDRESSES.lock().await.clone();

                let mut buffer = Vec::new();
                let x = "I am up".as_bytes();
                buffer.extend_from_slice(x);
                for server_address in servers_addresses.as_slice() {
                    servers_socket.send_to(&buffer, server_address.to_socket_addrs().unwrap().next().unwrap()).await?;
                    // send_response(&servers_socket, server_address, &buffer).await?;   
                }
            }
        }



        std::thread::sleep(std::time::Duration::from_millis(10));
        // //read from terminal
        // let mut input = String::new();
        // std::io::stdin().read_line(&mut input)?;
        // let input = input.trim();
        // // If character is 'd' then make the server down and if it is 'u' then make the server up
        // if input == "d" {
        //     let down = &mut *DOWN.lock().await;
        //     *down = true;
        //     println!("I am now down");
        //     // send to all servers that i am down
        //     let servers_addresses = SERVER_ADDRESSES.lock().await.clone();

        //     let mut buffer = Vec::new();
        //     let x = "I am down".as_bytes();
        //     buffer.extend_from_slice(x);
        //     for server_address in servers_addresses.as_slice() {
        //         servers_socket.send_to(&buffer, server_address.to_socket_addrs().unwrap().next().unwrap()).await?;
        //         // send_response(&servers_socket, server_address, &buffer).await?;   
        //     }

        // }
        // else if input == "u" {
        //     let down = &mut *DOWN.lock().await;
        //     if (*down){
        //         *down = false;
        //         println!("I am now up");

        //         let servers_addresses = SERVER_ADDRESSES.lock().await.clone();

        //         let mut buffer = Vec::new();
        //         let x = "I am up".as_bytes();
        //         buffer.extend_from_slice(x);
        //         for server_address in servers_addresses.as_slice() {
        //             servers_socket.send_to(&buffer, server_address.to_socket_addrs().unwrap().next().unwrap()).await?;
        //             // send_response(&servers_socket, server_address, &buffer).await?;   
        //         }
        //     }
            
        // }
        // else {
        //     println!("Invalid input");
        // }
    }    
}


async fn handle_server(servers_socket: &UdpSocket, client_socket: &UdpSocket) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let mut buffer = vec![0; 1024];
        let (length, sender_address) = servers_socket
            .recv_from(&mut buffer)
            .await?;

        // if i am down do not respond
        let down = &mut *DOWN.lock().await;
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
                if servers_addresses[i].to_socket_addrs().unwrap().next().unwrap() == sender_address {
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
            servers_addresses.push(sender_address.to_string());
            // send the updated directory to all servers
            let directory = &mut *DIRECTORY.lock().await;
            let mut buffer = Vec::new();
            let no_entries = directory.len();
            let input = "Directory".as_bytes();
            buffer.extend_from_slice(input);
            
            buffer.push(no_entries as u8);
            for (key, value) in directory.iter() {
                let client_addr = key.parse::<SocketAddr>().unwrap();
                let client_addr_bytes: [u8; 4];
                let port_bytes: [u8; 2]; 
                if let SocketAddr::V4(v4) = client_addr {
                    client_addr_bytes = v4.ip().octets();
                    port_bytes = v4.port().to_be_bytes();
                }
                else {
                    client_addr_bytes = [0; 4];
                    port_bytes = [0; 2];
                }
                buffer.extend(client_addr_bytes.iter());
                buffer.extend(port_bytes.iter());
                buffer.push(value.online);
                let no_clients: u8 = value.clients_waiting.len() as u8;
                buffer.push(no_clients as u8);
                for i in 0..value.clients_waiting.len() {
                    let client_addr = value.clients_waiting[i].clone();
                    let client_addr_bytes: [u8; 4];
                    let port_bytes: [u8; 2]; 
                    if let SocketAddr::V4(v4) = client_addr.parse::<SocketAddr>().unwrap() {
                        client_addr_bytes = v4.ip().octets();
                        port_bytes = v4.port().to_be_bytes();
                    }
                    else {
                        client_addr_bytes = [0; 4];
                        port_bytes = [0; 2];
                    }
                    buffer.extend(client_addr_bytes.iter());
                    buffer.extend(port_bytes.iter());
                }
            }
            println!("Server {} is up", sender_address);
            // send to the server with sender_address the directory
            servers_socket.send_to(&buffer, sender_address).await?;
            println!("Directory sent");
            println!("Directory len: {}", directory.len());
            for (key, value) in directory.iter() {
                println!("{}: {}", key, value.online);
                for i in 0..value.clients_waiting.len() {
                    println!("{}: {}", key, value.clients_waiting[i]);
                }
            }
            continue;
        }

        // if received "Directory" from another server update my directory
        let z = "Directory".as_bytes();
        if buffer[0..z.len()] == z[..] {
            let mut directory = &mut *DIRECTORY.lock().await;
            directory.clear();
            let no_entries = buffer[z.len()];
            let mut index = z.len() + 1;
            for i in 0..no_entries {
                let client_addr = SocketAddr::new(Ipv4Addr::new(buffer[index], buffer[index + 1], buffer[index + 2], buffer[index + 3]).into(), u16::from_ne_bytes([buffer[index + 5], buffer[index + 4]]));
                let online = buffer[index + 6];
                let no_clients = buffer[index + 7];
                let mut clients_waiting = Vec::new();
                for j in 0..no_clients {
                    let client_addr = SocketAddr::new(Ipv4Addr::new(buffer[index + 8 + (j * 6) as usize], buffer[index + 9 + (j * 6)as usize], buffer[index + 10 + (j * 6)as usize], buffer[index + 11 + (j * 6) as usize]).into(), u16::from_ne_bytes([buffer[index + 13 + (j * 6) as usize], buffer[index + 12 + (j * 6) as usize]]));
                    clients_waiting.push(client_addr.to_string());
                }
                directory.insert(client_addr.to_string(), DirectoryEntry{ online: online, clients_waiting: clients_waiting });
                index += 8 + no_clients as usize * 6;
            }
            println!("Directory updated");
            println!("Directory len: {}", directory.len());
            for (key, value) in directory.iter() {
                println!("{}: {}", key, value.online);
                for i in 0..value.clients_waiting.len() {
                    println!("{}: {}", key, value.clients_waiting[i]);
                }
            }

            continue;
        }
        let load_no = u32::from_ne_bytes([buffer[3], buffer[2], buffer[1], buffer[0]]);
        // extract client address from buffer
        let client_addr = SocketAddr::new(Ipv4Addr::new(buffer[4], buffer[5], buffer[6], buffer[7]).into(), u16::from_ne_bytes([buffer[9], buffer[8]]));
        println!("Load of sender {} is {} for client {}", sender_address, load_no, client_addr);
        let election_data_map = &mut *REQUEST_DATA_MAP.lock().await;
        let entry = election_data_map
            .entry(client_addr.to_string())
            .or_insert(Vec::new());
        entry.push(ElectionData{ load: load_no, server_address: sender_address });
        let THE_SERVERS = SERVER_ADDRESSES.lock().await.clone();
        let server_len = THE_SERVERS.len();
        println!("Server length is: {}", server_len);
        println!("Entry length is: {}", entry.len());
        for i in 0..entry.len(){
            println!("Load of server {} is {} for client {}", entry[i].server_address, entry[i].load, client_addr);
        }
        if entry.len() == server_len {

            let my_load = &mut *CPU_LOAD.lock().await;

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
                let my_load = &mut *LOAD.lock().await;
                *my_load += 1;
                client_socket.send_to(response.as_bytes(), client_addr).await?;
            }
            else {
                println!("Leader is: {}", least_load_addr);

                let mut highest_load = *my_load;
                let mut highest_load_addr = client_socket.local_addr().unwrap();

                if entry.len() == 1{
                    continue;
                }
            
                for i in 0..entry.len(){
                    if entry[i].load > highest_load {
                        highest_load = entry[i].load;
                        highest_load_addr = entry[i].server_address;
                    }
                    else {
                        if entry[i].load == highest_load {
                            if entry[i].server_address > highest_load_addr {
                                highest_load = entry[i].load;
                                highest_load_addr = entry[i].server_address;
                            }
                        }
                    }
                    
                }

                let my_load = &mut *LOAD.lock().await;
                if *my_load != 0 {
                    continue;
                } 
                

                println!("Highest load is: {}", highest_load);
                println!("Highest load address is: {}", highest_load_addr);

                if highest_load_addr == client_socket.local_addr().unwrap() {
                    println!("I am going down");

                    *down = true;
                    // send to all servers that i am down
                    
                    let servers_addresses = THE_SERVERS;
                    let mut buffer = Vec::new();
                    let x = "I am down".as_bytes();
                    buffer.extend_from_slice(x);
                    let length = servers_addresses.len();
                    println!("Length is: {}", length);
                    for server_address in servers_addresses.as_slice() {
                        println!("Sending to server: {}", server_address);
                        servers_socket.send_to(&buffer, server_address.to_socket_addrs().unwrap().next().unwrap()).await?;
                        // send_response(&servers_socket, server_address, &buffer).await?;   
                    }

                    // timeout for 10 seconds 
                    let handle = tokio::spawn(async move {
                        thread::sleep(Duration::from_secs(10));
                        let down = &mut *DOWN.lock().await;
                        *down = false;
                        println!("I am now up");

                        // let servers_addresses = SERVER_ADDRESSES.lock().await.clone();

                        // let mut buffer = Vec::new();
                        // let x = "I am up".as_bytes();
                        // buffer.extend_from_slice(x);
                        // for server_address in servers_addresses.as_slice() {
                        //     servers_socket.send_to(&buffer, server_address.to_socket_addrs().unwrap().next().unwrap()).await?;
                        //     // send_response(&servers_socket, server_address, &buffer).await?;   
                        // }
                    });
                }
            }
        }
            
    }
}


use std::env;
extern crate sysinfo;



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

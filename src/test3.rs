use std::net::{UdpSocket, SocketAddr, ToSocketAddrs};
use std::convert::TryInto;
use std::thread;
use std::time::{Duration, Instant};
use std::fs::File;
use std::io::{Read, Write};
use std::io;
use std::num;
use std::collections::HashMap;
use std::sync::Arc;
use std::net::Ipv4Addr;
use tokio::sync::Mutex;


use std::fs;


// use std::f64;

extern crate lazy_static;
use lazy_static::lazy_static;

lazy_static! {
    static ref SENT_IMAGES: Mutex<HashMap<String, Vec<u8>>> = {
        let mut m = HashMap::new();
        Mutex::new(m)
    };
}



// To check if the response is election result or processed request
enum ServerReply {
    Address(SocketAddr),
    Data(Vec<u8>),
    None,
}

// For later bundling a request based on the feature - to be modified
fn bundle_request(operation_flag: &u8, number: &i64) -> Vec<u8> {
    let mut request = Vec::new();
    request.push(*operation_flag);
    request.extend_from_slice(&number.to_be_bytes());
    request
}

// send request to all servers to init an election
fn send_request_to_servers(socket: &UdpSocket, server_addresses: &Vec<SocketAddr>, data: &[u8]) {    
    // loop over all servers and send request to each of them
    for &server_address in server_addresses {
        socket
            .send_to(data, server_address)
            .expect("Failed to send data to server");
    }    
}

// to send to one server after election ends
fn send_request_to_server(socket: &UdpSocket, server_addr: &SocketAddr, data: &[u8]) {
    socket
        .send_to(data, server_addr)
        .expect("Failed to send data to server");
}

// get responses to the socket
// fn read_response(socket: &UdpSocket) -> ServerReply {
fn read_response(socket: &UdpSocket) -> Vec<u8> {
    let mut buffer = vec![0; 1024];
    // Receive the result from the server
    let (length, server_addr) = socket
        .recv_from(&mut buffer)
        .expect("Failed to receive data from server");
    
    // receive a flag to indicate the type of reply, 0 is election result (server address), 1 is the processed request's result
    // let reply_flag = buffer[0];

    // // returning the appropriate result
    // match reply_flag {
    //     0 => return ServerReply::Address(server_addr),
    //     1 => {
    //         return ServerReply::Data(buffer[1..length].to_vec());
    //     },
    //     _ => ServerReply::None
    // }
    return buffer[0..length].to_vec();
}

fn send_data(socket: &UdpSocket, address: &str, data: &[u8]) -> io::Result<usize> {
    socket.send_to(data, address)
}

async fn listen_clients(server_socket: &UdpSocket, listen_socket: &UdpSocket) -> Result<(), Box<dyn std::error::Error>> {
    loop{

        const INITIAL_BUFFER_SIZE: usize = 65300; // Initial buffer size, change as needed
    
        let mut buffer2 = vec![0u8; INITIAL_BUFFER_SIZE]; // Create a buffer with an initial size
        let (size, source) = listen_socket.recv_from(&mut buffer2)?;

        buffer2.resize(size, 0); // Resize the buffer to fit the received data

        let request_flag :u8 = buffer2[0];

        
        if request_flag == 0 { // client wants to get my images
            println!("Client {} wants to get my images, please press 4", source);
        }
        else if request_flag == 1 { // client wants to increase their accesses
            println!("Client {} wants to increase their accesses, please press 4", source);
        }
        else if request_flag == 2 { // client wants to decrease access
            println!("Client {} wants to decrease access, please press 4", source);
        }
        else if request_flag == 3 { // server says that client is online
            // message received starts from the second byte
            let client_addr_bytes = &buffer2[1..size];
            // get message as string
            let client_addr = String::from_utf8_lossy(client_addr_bytes);
            println!("Received message from server\nMessage: {}", client_addr);
        }
        
    }
    Ok(())
}


async fn main_thread(socket: &UdpSocket, client_socket: &UdpSocket) -> Result<(), Box<dyn std::error::Error>> 
{


    let server1_address = "127.0.0.1:8081";
    let server2_address = "127.0.0.2:8081";
    let server3_address = "127.0.0.3:8081";

    let mut iteration = 0;

    loop {
        println!("What do you want to do? \n0: Register as online \n1: Go offline\n2: Request increase in access from client\n3: See received images\n4: Receive request from clients\n5: Request all online clients from server\n6: Decrease access");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim();
        // convert the input to u8

        let input:u8 = input.parse().unwrap();
        if input == 0 { // register as online to all servers
            let input:u8 = 2;
            let mut buffer = Vec::new();
            buffer.push(input); 
            println!("Sending request to server {}: {}", server1_address, input);

            socket.send_to(&input.to_ne_bytes(), server1_address.to_socket_addrs().unwrap().next().unwrap());
            socket.send_to(&input.to_ne_bytes(), server2_address.to_socket_addrs().unwrap().next().unwrap());
            socket.send_to(&input.to_ne_bytes(), server3_address.to_socket_addrs().unwrap().next().unwrap());            
        }
        else if input == 1{ // go offline to all servers
            let input:u8 = 3;
            let mut buffer = Vec::new();
            buffer.push(input); 
            println!("Sending request to server {}: {}", server1_address, input);

            socket.send_to(&input.to_ne_bytes(), server1_address.to_socket_addrs().unwrap().next().unwrap());
            socket.send_to(&input.to_ne_bytes(), server2_address.to_socket_addrs().unwrap().next().unwrap());
            socket.send_to(&input.to_ne_bytes(), server3_address.to_socket_addrs().unwrap().next().unwrap());
        } 
        else if input == 2 { // request access increase from client
            // get list of all files in ./src/received
            let paths = fs::read_dir("./src/received")?;
            let mut files = Vec::new();
            // add all .jpg or .jpeg files to the list
            for path in paths {
                let path = path?.path();
                let extension = path.extension();
                if extension == Some("jpg".as_ref()) || extension == Some("jpeg".as_ref()) {
                    files.push(path);
                }
            }

            // print the list of files
            println!("Choose file to increase accesses:");
            for (i, file) in files.iter().enumerate() {
                let path_without_prefix = file.strip_prefix("./src/received").unwrap();
                println!("{}: {}", i, path_without_prefix.display());
            }

            // get the user's choice
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            let input = input.trim();
            let input:usize = input.parse().unwrap();

            // get current number of accesses
            let file_path = files[input].clone();
            let mut f = File::open(file_path.clone())?;
            let mut image_bytes = Vec::new();
            f.read_to_end(&mut image_bytes)?;
            let len = image_bytes.len();
            let no_accesses = image_bytes[len - 1];
            println!("You have {} accesses left\nHow much more do you want?", no_accesses);

            // get the user's choice
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            let input = input.trim();
            let increase_access:u8 = input.parse().unwrap();

            // send to client to request access increase
            let request_flag :u8 = 1;
            let mut send_buffer = Vec::new();
            send_buffer.push(request_flag);
            send_buffer.push(increase_access);

            // get the client address using file name
            let path_without_prefix = file_path.strip_prefix("./src/received").unwrap();
            // find index of the first i in the file name
            let mut i = 0;
            for c in path_without_prefix.to_str().unwrap().chars() {
                if c == 'i' {
                    break;
                }
                i += 1;
            }
            // get the client address
            let the_client_addr = path_without_prefix.to_str().unwrap()[0..i].to_string();
            println!("i: {}", i);


            let mut image_no;
            // get the image number by getting the number after letter e and before the .
            let mut e = 0;
            for c in path_without_prefix.to_str().unwrap().chars() {
                if c == 'e' {
                    break;
                }
                e += 1;
            }
            e += 1;
            let mut dot = e;
            for c in path_without_prefix.to_str().unwrap()[e..].chars() {
                if c == '.' {
                    break;
                }
                dot += 1;
            }

            println!("e: {}, dot: {}", e, dot);

            image_no = path_without_prefix.to_str().unwrap()[e..dot].to_string();
            println!("image_no: {}", image_no);
            let image_no:u8 = image_no.parse().unwrap();
            println!("image_no: {}", image_no);

            send_buffer.push(image_no);


            // Check that client is online
              // do election
              let input:u8 = 0;
              let mut buffer = Vec::new();
              buffer.push(input);
      
              println!("Sending request to server {}: {}", server1_address, input);
      
              socket.send_to(&input.to_ne_bytes(), server1_address.to_socket_addrs().unwrap().next().unwrap()); 
              socket.send_to(&input.to_ne_bytes(), server2_address.to_socket_addrs().unwrap().next().unwrap());
              socket.send_to(&input.to_ne_bytes(), server3_address.to_socket_addrs().unwrap().next().unwrap());
      
              let mut buffer1 = [0; 1024];
              let (size, source) = socket.recv_from(&mut buffer1)?;
      
              let response1 = String::from_utf8_lossy(&buffer1);
      
              println!("Received response from server: {}", response1);
              println!("The server address is: {}", source);

              let selected_server = source;
  
              // request all online clients from server
              let input:u8 = 4;
              let mut buffer = Vec::new();
              buffer.push(input);
  
              println!("Sending request to server {}: {}", source, input);
  
              socket.send_to(&input.to_ne_bytes(), source).unwrap();
  
              const INITIAL_BUFFER_SIZE: usize = 65300; // Initial buffer size, change as needed
      
              let mut buffer2 = vec![0u8; INITIAL_BUFFER_SIZE]; // Create a buffer with an initial size
              let (size, source) = socket.recv_from(&mut buffer2)?;
  
              buffer2.resize(size, 0); // Resize the buffer to fit the received data 
  
              let no_clients :u8 = buffer2[0];
  
              // add all clients except the myself to a vector
              let mut clients = Vec::new();
              for i in 0..no_clients {
                  let client_addr = SocketAddr::new(Ipv4Addr::new(buffer2[1 + i as usize * 6], buffer2[2 + i as usize * 6], buffer2[3 + i as usize * 6], buffer2[4 + i as usize * 6]).into(), u16::from_be_bytes([buffer2[5 + i as usize * 6], buffer2[6 + i as usize * 6]]));
                  if client_addr == socket.local_addr().unwrap() {
                      continue;
                  }
                  clients.push(client_addr);
  
              }

                // check if the client is online
                let mut client_online = false;
                for client in clients {
                    if client.to_string() == the_client_addr {
                        client_online = true;
                        break;
                    }
                }

                if client_online == false {
                    println!("Client {} is offline\nDo you want to get a notification when he is back online? (y/n)", the_client_addr);
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input)?;
                    let input = input.trim();

                    if input == "y" {
                        // send to server to add the client to the waiting list
                        let input:u8 = 5;
                        let mut buffer = Vec::new();
                        buffer.push(input);

                        // send the client address
                        let client_addr_bytes: [u8; 4];
                        let port_bytes: [u8; 2]; 
                        if let SocketAddr::V4(v4) = the_client_addr.parse::<SocketAddr>().unwrap() {
                            client_addr_bytes = v4.ip().octets();
                            port_bytes = v4.port().to_be_bytes();
                        }
                        else {
                            client_addr_bytes = [0; 4];
                            port_bytes = [0; 2];
                        }
                        buffer.extend(client_addr_bytes.iter());
                        buffer.extend(port_bytes.iter()); 

                        println!("Sending request to server {}: {}", selected_server, input);

                        socket.send_to(&buffer, server1_address).unwrap();
                        socket.send_to(&buffer, server2_address).unwrap();
                        socket.send_to(&buffer, server3_address).unwrap();
                    }
                    else if input == "n" {
                    }

        

                }
                else {
                    println!("Sending request to client {}: {}", the_client_addr, input);

                    socket.send_to(&send_buffer, the_client_addr.to_socket_addrs().unwrap().next().unwrap());

                    // send to same ip but port + 1
                    let client = the_client_addr.to_socket_addrs().unwrap().next().unwrap();
                    let cl_ip = client.ip();
                    let cl_port = client.port() + 1;
                    let cl_addr = SocketAddr::new(cl_ip, cl_port);
                    let input:u8 = 1;
                    let mut buffer = Vec::new();
                    buffer.push(input);


                    socket.send_to(&buffer, cl_addr).unwrap();

                    // receive the client's response
                    const INITIAL_BUFFER_SIZE: usize = 65300; // Initial buffer size, change as needed

                    let mut buffer2 = vec![0u8; INITIAL_BUFFER_SIZE]; // Create a buffer with an initial size
                    let (size, source) = socket.recv_from(&mut buffer2)?;

                    buffer2.resize(size, 0); // Resize the buffer to fit the received data

                    let response = buffer2[0];

                    if response == 0 {
                        println!("Client denied your request");
                    }
                    else if response == 1 {
                        println!("Client approved your request");
                        // update the number of accesses
                        image_bytes[len - 1] = image_bytes[len - 1] + increase_access;
                        // write to the image to update the number of accesses
                        let mut f = File::create(file_path)?;
                        f.write_all(&image_bytes)?;
                    }


                }

        }
        else if input == 3 { // decrypt and view one of the photos in ./src/received
            // get a list of all files in ./src/received
            let paths = fs::read_dir("./src/received")?;
            let mut files = Vec::new();
            // add all .jpg or .jpeg files to the list
            for path in paths {
                let path = path?.path();
                let extension = path.extension();
                if extension == Some("jpg".as_ref()) || extension == Some("jpeg".as_ref()) {
                    files.push(path);
                }
            }

            // print the list of files
            println!("Choose a file to decrypt:");
            for (i, file) in files.iter().enumerate() {
                let path_without_prefix = file.strip_prefix("./src/received").unwrap();
                println!("{}: {}", i, path_without_prefix.display());
            }

            // get the user's choice
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            let input = input.trim();
            let input:usize = input.parse().unwrap();

            // get the file path
            let file_path = files[input].clone();

            // read the file
            let mut f = File::open(file_path.clone())?;
            let mut image_bytes = Vec::new();
            f.read_to_end(&mut image_bytes)?;

            // get the number of accesses allowed
            let len = image_bytes.len();
            let no_accesses = image_bytes[len - 1];
            println!("You have {} accesses left", no_accesses);

            if no_accesses <= 0 {
                println!("You have no more accesses left");
                continue;
            }
            
            // decrement the number of accesses
            let no_accesses = no_accesses - 1;
            image_bytes[len - 1] = no_accesses;

            // write to the image to update the number of accesses
            let mut f = File::create(file_path)?;
            f.write_all(&image_bytes)?;



            // search array for first instance of 0xFFD9
            let mut prevByte = 0;
            let mut indx = 0;
            for i in 0..image_bytes.len() {
                if prevByte == 0xFF && image_bytes[i] == 0xD9 {
                    println!("Found 0xFFD9 at index {}", i);
                    indx = i + 1;
                    break;
                }
                prevByte = image_bytes[i];
            }

            // image_bytes.truncate(indx);
            let imageWrite = &image_bytes[indx..];       

            // write the image to a file
            let mut f = File::create("./src/decrypted.jpg")?;  
            f.write_all(&imageWrite)?;

            // set timeout to 10 seconds then delete the file
            let timeout = Duration::from_secs(10);
            thread::sleep(timeout);
            fs::remove_file("./src/decrypted.jpg")?;          
        }
        else if input == 4 { // receive request from clients
            // receive the request from the client
            const INITIAL_BUFFER_SIZE: usize = 65300; // Initial buffer size, change as needed
    
            let mut buffer2 = vec![0u8; INITIAL_BUFFER_SIZE]; // Create a buffer with an initial size
            let (size, source) = socket.recv_from(&mut buffer2)?;

            buffer2.resize(size, 0); // Resize the buffer to fit the received data

            let request_flag :u8 = buffer2[0];

            println!("HERE");

            println!("flag {}", request_flag);

            if request_flag == 0 { // client wants to get my images
                // get a list of all files in ./src/images/compressed
                let paths = fs::read_dir("./src/images/compressed")?;
                let mut files = Vec::new();
                // add all .jpg or .jpeg files to the list
                for path in paths {
                    let path = path?.path();
                    let extension = path.extension();
                    if extension == Some("jpg".as_ref()) || extension == Some("jpeg".as_ref()) {
                        files.push(path);
                    }
                }

                // put the number of files in the first byte of the buffer and the images in the rest of the buffer
                let no_files :u8 = files.len().try_into().unwrap();
                let mut buffer = Vec::new();
                buffer.push(no_files);
                for file in files.clone() {
                    let mut f = File::open(file)?;
                    let mut image_bytes = Vec::new();
                    f.read_to_end(&mut image_bytes)?;
                    buffer.extend_from_slice(&image_bytes);
                }

                // send the images to the client
                socket.send_to(&buffer, source).unwrap();  

                // receive the client's choice
                const INITIAL_BUFFER_SIZE: usize = 65300; // Initial buffer size, change as needed

                let mut buffer2 = vec![0u8; INITIAL_BUFFER_SIZE]; // Create a buffer with an initial size
                let (size, source) = socket.recv_from(&mut buffer2)?;

                buffer2.resize(size, 0); // Resize the buffer to fit the received data

                let choice :u8 = buffer2[0];

                // add the choice to the hashmap
                let mut sent_images = &mut *SENT_IMAGES.lock().await;
                sent_images.entry(source.to_string()).or_insert(Vec::new()).push(choice);


                // get the file path
                let file_path = files[choice as usize].clone();

                // get the file name without the prefix
                let path_without_prefix = file_path.strip_prefix("./src/images/compressed").unwrap();

                // read the uncompressed file from ./src/images/
                let mut f = File::open(format!("./src/images/{}", path_without_prefix.display()))?;
                let mut image_bytes = Vec::new();
                f.read_to_end(&mut image_bytes)?;

                let the_client_address = source;


                //********************************************************************************************* */

                // elect a server and encryption

                let input:u8 = 0;
                let mut buffer = Vec::new();
                buffer.push(input);
        
                println!("Sending request to server {}: {}", server1_address, input);
        
                socket.send_to(&input.to_ne_bytes(), server1_address.to_socket_addrs().unwrap().next().unwrap()); 
                socket.send_to(&input.to_ne_bytes(), server2_address.to_socket_addrs().unwrap().next().unwrap());
                socket.send_to(&input.to_ne_bytes(), server3_address.to_socket_addrs().unwrap().next().unwrap());
        
                let mut buffer1 = [0; 1024];
                let (size, source) = socket.recv_from(&mut buffer1)?;
        
                let response1 = String::from_utf8_lossy(&buffer1);
        
                println!("Received response from server: {}", response1);
                println!("The server address is: {}", source);
        
        
        
                    let image_size = image_bytes.len();
                    println!("Image size: {}", image_size);
        
                    let flag:u8 = 1;
                    let mut no_frags = (image_size / 65000) as u8;
                    if image_size % 65000 != 0 {
                        no_frags += 1;
                    }
        
                    for i in 0..no_frags {
                        let mut buffer = Vec::new();
                        buffer.push(flag);
                        let frag_no = i as u8;
                        buffer.push(frag_no);
                        buffer.push(no_frags);
                        let start : usize = i as usize * 65000 as usize ;
                        let mut end : usize= (i + 1) as usize * 65000 as usize;
                        println!("i: {}, start: {}, end: {}", i,  start, end);
                        if end > image_size {
                            end = image_size;
                        }
                        buffer.extend_from_slice(&image_bytes[start..end]);
                        send_data(&socket, source.to_string().as_str(), &buffer)?;
                        // sleep for 10ms
                        
                        std::thread::sleep(std::time::Duration::from_millis(1));
                    }
        
                    // // receive the encrypted image from the server
        
                    let mut buffer2 = vec![0u8; INITIAL_BUFFER_SIZE]; // Create a buffer with an initial size
                    let (size, source) = socket.recv_from(&mut buffer2)?;
        
                    buffer2.resize(size, 0); // Resize the buffer to fit the received data 
        
                    // read data in the from buffer in the order of frag_no, no_frags, image
                    let frag_no :u8 = buffer2[0];
                    let no_frags :u8 = buffer2[1];
                    let mut image = HashMap::new();
                    image.insert(frag_no, buffer2[2..].to_vec());
                    while image.len() < no_frags as usize {
                        let (size, source) = socket.recv_from(&mut buffer2)?;
                        buffer2.resize(size, 0);
                        let frag_no :u8 = buffer2[0];
                        image.insert(frag_no, buffer2[2..].to_vec());
                    }
                    let mut image_bytes = Vec::new();
                    for i in 0..no_frags {
                        image_bytes.extend_from_slice(&image.get(&i).unwrap());
                    }

                    // ask user how many accesses should be allowed
                    println!("How many accesses should be allowed?");
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input)?;
                    let input = input.trim();
                    let input:u8 = input.parse().unwrap();

                    // put the number of accesses as the last byte in the buffer
                    image_bytes.push(input);

                    // send the encrypted image to the client

                    let image_size = image_bytes.len();
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
                        println!("i: {}, start: {}, end: {}", i,  start, end);
                        if end > image_size {
                            end = image_size;
                        }
                        buffer.extend_from_slice(&image_bytes[start..end]);
                        send_data(&socket, the_client_address.to_string().as_str(), &buffer)?;
                        // sleep for 10ms
                        
                        std::thread::sleep(std::time::Duration::from_millis(1));
                    }



            }
            else if request_flag == 1 { // client wants to increase their accesses
                println!("HERE2");
                let newAccesses = buffer2[1];
                let image_no = buffer2[2];

                // get the file path of all images in ./src/images
                let paths = fs::read_dir("./src/images")?;
                let mut files = Vec::new();
                // add all .jpg or .jpeg files to the list
                for path in paths {
                    let path = path?.path();
                    let extension = path.extension();
                    if extension == Some("jpg".as_ref()) || extension == Some("jpeg".as_ref()) {
                        files.push(path);
                    }
                }

                // get the file path of the image with index image_no
                let file_path = files[image_no as usize].clone();
                // get name without prefix
                let path_without_prefix = file_path.strip_prefix("./src/images").unwrap();


                println!("Client {} wants to increase their accesses by {} for image {}\nApprove (y/n)", source, newAccesses, path_without_prefix.display());
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                let input = input.trim();

                if input == "y" {
                    // send 1 to client to approve
                    let input:u8 = 1;
                    let mut buffer = Vec::new();
                    buffer.push(input);

                    println!("Sending request to client {}: {}", source, input);

                    socket.send_to(&input.to_ne_bytes(), source).unwrap();
                }
                else if input == "n" {
                    // send 0 to client to deny
                    let input:u8 = 0;
                    let mut buffer = Vec::new();
                    buffer.push(input);

                    println!("Sending request to client {}: {}", source, input);

                    socket.send_to(&input.to_ne_bytes(), source).unwrap();
                }
                
            }
            else if request_flag == 2 { // client wants to decrease access
                println!("HERE");
                let image_no = buffer2[1];
                let newAccesses = buffer2[2];

                println!("HERE");

                // path of image will be client address + 'image' + image_no + '.jpg'
                let mut path = source.to_string();
                path.push_str("image");
                path.push_str(image_no.to_string().as_str());
                path.push_str(".jpg");

                // path will have ./src/received/ at the beginning
                path.insert_str(0, "./src/received/");
                println!("Client {} will decrease your accesses to {} for image {}\n", source, newAccesses, path);


                // read the file
                let mut f = File::open(path.clone())?;
                let mut image_bytes = Vec::new();
                f.read_to_end(&mut image_bytes)?;

                // get the number of accesses allowed
                let len = image_bytes.len();
                image_bytes[len - 1] = newAccesses;

                // write to the image to update the number of accesses
                let mut f = File::create(path)?;
                f.write_all(&image_bytes)?;
                
            }
            else if request_flag == 3 { // server says that client is online
                // message received starts from the second byte
                let client_addr_bytes = &buffer2[1..size];
                // get message as string
                let client_addr = String::from_utf8_lossy(client_addr_bytes);
                println!("Received message from server\nMessage: {}", client_addr);

                
            }
        }
        else if input == 5 { // do election and request all online clients from server
            
            // do election
            let input:u8 = 0;
            let mut buffer = Vec::new();
            buffer.push(input);
    
            println!("Sending request to server {}: {}", server1_address, input);
    
            socket.send_to(&input.to_ne_bytes(), server1_address.to_socket_addrs().unwrap().next().unwrap()); 
            socket.send_to(&input.to_ne_bytes(), server2_address.to_socket_addrs().unwrap().next().unwrap());
            socket.send_to(&input.to_ne_bytes(), server3_address.to_socket_addrs().unwrap().next().unwrap());
    
            let mut buffer1 = [0; 1024];
            let (size, source) = socket.recv_from(&mut buffer1)?;
    
            let response1 = String::from_utf8_lossy(&buffer1);
    
            println!("Received response from server: {}", response1);
            println!("The server address is: {}", source);

            // request all online clients from server
            let input:u8 = 4;
            let mut buffer = Vec::new();
            buffer.push(input);

            println!("Sending request to server {}: {}", source, input);

            socket.send_to(&input.to_ne_bytes(), source).unwrap();

            const INITIAL_BUFFER_SIZE: usize = 65300; // Initial buffer size, change as needed
    
            let mut buffer2 = vec![0u8; INITIAL_BUFFER_SIZE]; // Create a buffer with an initial size
            let (size, source) = socket.recv_from(&mut buffer2)?;

            buffer2.resize(size, 0); // Resize the buffer to fit the received data 

            let no_clients :u8 = buffer2[0];

            // add all clients except the myself to a vector
            let mut clients = Vec::new();
            for i in 0..no_clients {
                let client_addr = SocketAddr::new(Ipv4Addr::new(buffer2[1 + i as usize * 6], buffer2[2 + i as usize * 6], buffer2[3 + i as usize * 6], buffer2[4 + i as usize * 6]).into(), u16::from_be_bytes([buffer2[5 + i as usize * 6], buffer2[6 + i as usize * 6]]));
                if client_addr == socket.local_addr().unwrap() {
                    continue;
                }
                clients.push(client_addr);

            }
            let n = clients.len();
            println!("Received {} clients", n);
            for (i, client) in clients.iter().enumerate() {
                println!("{}: {}", i, client);
            }
            println!("Do you want to send to view their images? (y/n)");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            let input = input.trim();

            if input == "n" {
                continue;
            }
            else if input == "y" {
                // choose a client
                println!("Choose a client to request their images:");
                for (i, client) in clients.iter().enumerate() {
                    println!("{}: {}", i, client);
                }

                // get the user's choice
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                let input = input.trim();
                let input:usize = input.parse().unwrap();

                // get the client address
                let client_addr = clients[input];

                // send request to client
                let input:u8 = 0;
                let mut buffer = Vec::new();
                buffer.push(input);

                println!("Sending request to client {}: {}", client_addr, input);
                /* CHANGETHIS */
                socket.send_to(&input.to_ne_bytes(), client_addr).unwrap();
                let input = 0;
                let mut buffer = Vec::new();
                buffer.push(input);
                let cl_ip = client_addr.ip();
                let cl_port = client_addr.port() + 1;
                let cl_addr = SocketAddr::new(cl_ip, cl_port);
                socket.send_to(&buffer, cl_addr).unwrap();



                // receive the images from the client
                const INITIAL_BUFFER_SIZE: usize = 65300; // Initial buffer size, change as needed

                let mut buffer2 = vec![0u8; INITIAL_BUFFER_SIZE]; // Create a buffer with an initial size
                let (size, source) = socket.recv_from(&mut buffer2)?;

                buffer2.resize(size, 0); // Resize the buffer to fit the received data

                // read data in the from buffer as the number of images and the images
                let no_images :u8 = buffer2[0];

                // create a hashmap to hold the images
                let mut images = HashMap::new();

                // read the images from the buffer once you find 0xFFD9 then put the image in the hashmap
                let mut prevByte = 0;
                let mut indx = 0;
                let mut start = 1;
                let mut iteration = 0;
                println!("buffer2.len(): {}", buffer2.len());
                for i in 1..buffer2.len() {
                    if prevByte == 0xFF && buffer2[i] == 0xD9 {
                        indx = i + 1;
                        println!("the image number {} starts at {} and ends at {}", iteration, start, indx);
                        let image = &buffer2[start..indx];
                        images.insert(iteration, image.to_vec());
                        start = indx;
                        iteration += 1;
                    }
                    prevByte = buffer2[i];
                }



                
                // go through the images and write them to files
                for i in 0..no_images {
                    let mut image_bytes = Vec::new();
                    image_bytes.extend_from_slice(&images.get(&i).unwrap());
                    let mut f = File::create(format!("./src/received/temp/image{}.jpg", i))?;
                    f.write_all(&image_bytes)?;
                }

                // prompt the user to choose an image to get from the client
                println!("Choose an image to get from the client:\nWrite the number of the image you want to get.");
                for i in 0..no_images {
                    println!("{}: image{}.jpg", i, i);
                }

                // get the user's choice
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                let input = input.trim();
                let input:u8 = input.parse().unwrap();   

                // delete the images in ./src/received/temp
                let paths = fs::read_dir("./src/received/temp")?;
                for path in paths {
                    let path = path?.path();
                    let extension = path.extension();
                    if extension == Some("jpg".as_ref()) || extension == Some("jpeg".as_ref()) {
                        fs::remove_file(path)?;
                    }
                }

                // send the choice to the client
                let mut buffer = Vec::new();
                buffer.push(input);   

                println!("Sending request to client {}: {}", client_addr, input);

                socket.send_to(&input.to_ne_bytes(), source).unwrap();  

                // receive the image from the client
                let mut buffer2 = vec![0u8; INITIAL_BUFFER_SIZE]; // Create a buffer with an initial size
                let (size, source) = socket.recv_from(&mut buffer2)?;
    
                buffer2.resize(size, 0); // Resize the buffer to fit the received data 
    
                // read data in the from buffer in the order of frag_no, no_frags, image
                let frag_no :u8 = buffer2[0];
                let no_frags :u8 = buffer2[1];
                let mut image = HashMap::new();
                image.insert(frag_no, buffer2[2..].to_vec());
                while image.len() < no_frags as usize {
                    let (size, source) = socket.recv_from(&mut buffer2)?;
                    buffer2.resize(size, 0);
                    let frag_no :u8 = buffer2[0];
                    image.insert(frag_no, buffer2[2..].to_vec());
                }
                let mut image_bytes = Vec::new();
                for i in 0..no_frags {
                    image_bytes.extend_from_slice(&image.get(&i).unwrap());
                }   

                // write the image to a file
                let mut f = File::create(format!("./src/received/{}image{}.jpg", source, input))?;
                f.write_all(&image_bytes)?;


            }
            
        }
        else if input == 6 { // decrease accesses of a client
            // show user all images sent to clients
            let mut sent_images = &mut *SENT_IMAGES.lock().await;
            println!("Choose a client to decrease their accesses:");
            // loop over keys and print them
            for (i, key) in sent_images.keys().enumerate() {
                println!("{}: {}", i, key);
            }

            // get the user's choice
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            let input = input.trim();
            let input:usize = input.parse().unwrap();

            // get the client address
            let client_addr = sent_images.keys().nth(input).unwrap().clone();

            // loop over files in ./src/images and get all files
            let paths = fs::read_dir("./src/images")?;
            let mut files = Vec::new();
            // add all .jpg or .jpeg files to the list
            for path in paths {
                let path = path?.path();
                let extension = path.extension();
                if extension == Some("jpg".as_ref()) || extension == Some("jpeg".as_ref()) {
                    files.push(path);
                }
            }

            // show user all images sent to the client
            println!("Choose an image to decrease their accesses:");
            // loop over values for that client and print them
            for (i, value) in sent_images.get(&client_addr).unwrap().iter().enumerate() {
                println!("{}: {}", value, files[*value as usize].clone().strip_prefix("./src/images").unwrap().display());
            }

            // get the user's choice
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            let input = input.trim();
            let choice:usize = input.parse().unwrap();
            
            // send this to the client
            let input:u8 = 2;
            let mut buffer = Vec::new();
            buffer.push(input);
            buffer.push(choice as u8);

            println!("What do you want to decrease the accesses to?");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            let input = input.trim();
            let input:u8 = input.parse().unwrap();

            buffer.push(input);

            println!("Sending request to client {}: {}", client_addr, input);

            socket.send_to(&buffer, client_addr.clone()).unwrap();
            // send to same ip but port + 1
            let client = client_addr.to_socket_addrs().unwrap().next().unwrap();
            let cl_ip = client.ip();
            let cl_port = client.port() + 1;
            let cl_addr = SocketAddr::new(cl_ip, cl_port);
            let input:u8 = 2;
            let mut buffer = Vec::new();
            buffer.push(input);
            socket.send_to(&buffer, cl_addr).unwrap();
        }

        // Making the thread sleep for 1 second
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

}



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {


    let server1_address = "127.0.0.1:8081";
    let server2_address = "127.0.0.2:8081";
    let server3_address = "127.0.0.3:8081";

    const INITIAL_BUFFER_SIZE: usize = 65300; // Initial buffer size, change as needed


    let socket = UdpSocket::bind("127.0.0.1:0")?; // Binding to 0 allows the OS to choose an available port
    // let c_socket_clone = UdpSocket::bind("127.0.0.1:0")?;
    // create a socket with port number equal to socket + 1
    let listening_socket = UdpSocket::bind("127.0.0.1:".to_string() + &(socket.local_addr()?.port() + 1).to_string())?;
    println!("Client started on port {}", socket.local_addr()?);
    println!("Listening socket started on port {}", listening_socket.local_addr()?);



    // measure time to finish
    let start = Instant::now();
    for i in 0..500
    {

        let server1_address = "127.0.0.1:8081";
        let server2_address = "127.0.0.2:8081";
        let server3_address = "127.0.0.3:8081";

        let input:u8 = 0;
        let mut buffer = Vec::new();
        buffer.push(input);

        // println!("Sending request to server {}: {}", server1_address, input);

        socket.send_to(&input.to_ne_bytes(), server1_address.to_socket_addrs().unwrap().next().unwrap()); 
        socket.send_to(&input.to_ne_bytes(), server2_address.to_socket_addrs().unwrap().next().unwrap());
        socket.send_to(&input.to_ne_bytes(), server3_address.to_socket_addrs().unwrap().next().unwrap());



        

        let mut buffer1 = [0; 1024];
        let (size, source) = socket.recv_from(&mut buffer1)?;

        // read the uncompressed file from ./src/images/
        let mut f = File::open("./src/default1.jpeg")?;
        let mut image_bytes = Vec::new();
        f.read_to_end(&mut image_bytes)?;


        let image_size = image_bytes.len();
        println!("Image size: {}", image_size);

        let flag:u8 = 1;
        let mut no_frags = (image_size / 65000) as u8;
        if image_size % 65000 != 0 {
            no_frags += 1;
        }

        for i in 0..no_frags {
            let mut buffer = Vec::new();
            buffer.push(flag);
            let frag_no = i as u8;
            buffer.push(frag_no);
            buffer.push(no_frags);
            let start : usize = i as usize * 65000 as usize ;
            let mut end : usize= (i + 1) as usize * 65000 as usize;
            println!("i: {}, start: {}, end: {}", i,  start, end);
            if end > image_size {
                end = image_size;
            }
            buffer.extend_from_slice(&image_bytes[start..end]);
            send_data(&socket, source.to_string().as_str(), &buffer)?;
            // sleep for 10ms
            
            // std::thread::sleep(std::time::Duration::from_millis(1));
        }

        // // receive the encrypted image from the server

        let mut buffer2 = vec![0u8; INITIAL_BUFFER_SIZE]; // Create a buffer with an initial size
        let (size, source) = socket.recv_from(&mut buffer2)?;

        buffer2.resize(size, 0); // Resize the buffer to fit the received data 

        // read data in the from buffer in the order of frag_no, no_frags, image
        let frag_no :u8 = buffer2[0];
        let no_frags :u8 = buffer2[1];
        let mut image = HashMap::new();
        image.insert(frag_no, buffer2[2..].to_vec());
        while image.len() < no_frags as usize {
            let (size, source) = socket.recv_from(&mut buffer2)?;
            buffer2.resize(size, 0);
            let frag_no :u8 = buffer2[0];
            image.insert(frag_no, buffer2[2..].to_vec());
        }
        let mut image_bytes = Vec::new();
        for i in 0..no_frags {
            image_bytes.extend_from_slice(&image.get(&i).unwrap());
        }


        let image_size = image_bytes.len();
        println!("Image size: {}", image_size);

        // save the image to a file
        let mut f = File::create(format!("./src/received/testt{}.jpg", i))?;
        f.write_all(&image_bytes)?;
    }

    let duration = start.elapsed();
    println!("Time elapsed in expensive_function() is: {:?}", duration);
    



    Ok(())
}

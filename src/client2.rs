use std::net::{UdpSocket, SocketAddr, ToSocketAddrs};
use std::convert::TryInto;
use std::thread;
use std::time::{Duration, Instant};
use std::fs::File;
use std::io::{Read, Write};
use std::io;
use std::num;
use std::collections::HashMap;

// use std::f64;





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

// handling election result or processed request
// fn handle_response(server_reply: &ServerReply, socket: &UdpSocket, data: &[u8]) {
//     match server_reply {
//         ServerReply::Address(address) => {
//             send_request_to_server(socket, &address, data);
//             let server_reply: ServerReply = read_response(socket);
//             handle_response(&server_reply, socket, data);
//         },
//         ServerReply::Data(reply_data) => {
//             println!("Server replied with {:?}", i64::from_be_bytes(reply_data.as_slice().try_into().unwrap()));
//         },
//         ServerReply::None => {}
//     }
// }

fn send_data(socket: &UdpSocket, address: &str, data: &[u8]) -> io::Result<usize> {
    socket.send_to(data, address)
}
fn main() -> io::Result<()> {


    let socket = UdpSocket::bind("10.7.57.199:0")?; // Binding to 0 allows the OS to choose an available port
    println!("Client started on port {}", socket.local_addr()?);

    // Get the maximum UDP payload size allowed by the OS
    

    let server1_address = "10.7.57.254:8081";
    let server2_address = "10.7.57.176:8081";
    let server3_address = "10.7.57.72:8081";

    let mut iteration = 0;

    loop {
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


            let mut image_bytes = Vec::new();
            let mut f;
            f = File::open("./src/image3.jpg")?;
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
                
                std::thread::sleep(std::time::Duration::from_millis(1));
            }

            // // receive the encrypted image from the server
            const INITIAL_BUFFER_SIZE: usize = 65300; // Initial buffer size, change as needed

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
            let mut f = File::create("./src/encrypted.jpg")?;
            f.write_all(&image_bytes)?;

        // Making the thread sleep for 1 second
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}

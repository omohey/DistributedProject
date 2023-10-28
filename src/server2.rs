// server.rs

use std::net::UdpSocket;

fn main() {
    // There will be 3 servers in total, each listening on a different port. Save the ip and port of each server in a vector
    let servers = vec!["127.0.0.1:8080", "127.0.0.1:8081", "127.0.0.1:8082"];
    let myip = servers[1];


    let socket = UdpSocket::bind("127.0.0.1:8081").expect("Failed to bind to address");
    println!("Server listening on 127.0.0.1:8081");

    let mut buffer = [0; 1024];
    loop {
        let (size, source) = socket.recv_from(&mut buffer).expect("Failed to receive data");
        let request = String::from_utf8_lossy(&buffer[..size]);
        // Check if the source is one of the other servers
        if !servers.contains(&source.to_string().as_str()) {
            println!("Received request: {} from {}", request, source);
            let response = "Hello, client!";
            socket.send_to(response.as_bytes(), source).expect("Failed to send response");
            continue;
        }
        else
        {
            println!("Received request: {} from {}", request, source);
            let response = "Hello, server!";
            socket.send_to(response.as_bytes(), source).expect("Failed to send response");
            continue;
        }
    }
}
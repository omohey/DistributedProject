// server.rs

use std::net::UdpSocket;

fn main() {
    let socket = UdpSocket::bind("127.0.0.1:8080").expect("Failed to bind to address");
    println!("Server listening on 127.0.0.1:8080");

    let mut buffer = [0; 1024];
    loop {
        let (size, source) = socket.recv_from(&mut buffer).expect("Failed to receive data");
        let request = String::from_utf8_lossy(&buffer[..size]);
        println!("Received request: {} from {}", request, source);

        let response = "Hello, client!";
        socket.send_to(response.as_bytes(), source).expect("Failed to send response");
    }
}
// client.rs

use std::net::UdpSocket;
use std::io;

fn main() -> io::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:0")?; // Binding to 0 allows the OS to choose an available port
    socket.connect("127.0.0.1:8080")?;
    let mut increment = 0;
    loop {
        let input;
        // println!("Type a message to send to the server:");
        // io::stdin().read_line(&mut input)?;

        increment += 1;
        input = increment.to_string();
        socket.send(input.as_bytes())?;

        let mut buffer = [0; 1024];
        socket.recv_from(&mut buffer)?;

        let response = String::from_utf8_lossy(&buffer);
        println!("Received response: {}", response);
        // make the thread sleep for 1 second
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

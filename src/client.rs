use std::net::UdpSocket;
use std::io;


fn send_data(socket: &UdpSocket, address: &str, data: &[u8]) -> io::Result<usize> {
    socket.send_to(data, address)
}

fn main() -> io::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:0")?; // Binding to 0 allows the OS to choose an available port

    let server1_address = "127.0.0.1:8080";
    let server2_address = "127.0.0.1:8081";
    let server3_address = "127.0.0.1:8082";

    let mut increment: i32 = 0;
    loop {
        let input:i32;
        increment += 1;
        input = increment;

        send_data(&socket, server1_address, &input.to_ne_bytes())?;
        send_data(&socket, server2_address, &input.to_ne_bytes())?;
        send_data(&socket, server3_address, &input.to_ne_bytes())?;

        // Receiving from the server1
        let mut buffer1 = [0; 1024];
        let (size, source) = socket.recv_from(&mut buffer1)?;

        let response1 = String::from_utf8_lossy(&buffer1);

        println!("Received response from server: {}", response1);
        println!("The server address is: {}", source);

        // Making the thread sleep for 1 second
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

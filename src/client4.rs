use std::net::UdpSocket;
use std::io;

fn send_data(socket: &UdpSocket, address: &str, data: &[u8]) -> io::Result<usize> {
    socket.send_to(data, address)
}

fn main() -> io::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:0")?; // Binding to 0 allows the OS to choose an available port

    let server1_address = "127.0.0.1:8080";
    let server2_address = "127.0.0.1:8081";

    let mut increment = 0;
    loop {
        let input;
        increment += 1;
        input = increment.to_string();

        send_data(&socket, server1_address, input.as_bytes())?;
        send_data(&socket, server2_address, input.as_bytes())?;

        // Receiving from the server1
        let mut buffer1 = [0; 1024];
        socket.recv_from(&mut buffer1)?;

        // Receiving from the server2
        let mut buffer2 = [0; 1024];
        socket.recv_from(&mut buffer2)?;

        let response1 = String::from_utf8_lossy(&buffer1);
        let response2 = String::from_utf8_lossy(&buffer2);

        println!("Received response from server 1: {}", response1);
        println!("Received response from server 2: {}", response2);

        // Making the thread sleep for 1 second
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

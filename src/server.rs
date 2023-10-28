use std::convert::TryInto;
use std::net::SocketAddr;
use std::sync::Arc;
use std::thread;
use tokio::net::UdpSocket;

async fn read_request(socket: &UdpSocket) -> Result<(SocketAddr, Vec<u8>), Box<dyn std::error::Error>> {
    let mut buffer = vec![0; 1024];

    let (length, sender_address) = socket
        .recv_from(&mut buffer)
        .await?;
        // .expect("Failed to receive data from client");

    Ok((sender_address, buffer[0..length].to_vec()))
}

async fn send_response(socket: &UdpSocket, dest_addr: &SocketAddr, data: &Vec<u8>) -> Result<(), Box<dyn std::error::Error>>{
    socket
        .send_to(&data, dest_addr)
        .await?;
        // .expect(&format!("Failed to send response to {:?}", dest_addr.to_string()));
    Ok(())
}

// fn process_request(server_addresses: &Vec<SocketAddr>, load: &i64, operation_flag: &u8, number: &i64) -> i64 {
fn process_request(operation_flag: &u8, number: &i64) -> i64 {
    match operation_flag {
        0 => number.checked_add(1).unwrap_or(i64::MAX), // Increment with overflow handling
        1 => number.checked_sub(1).unwrap_or(i64::MIN), // Decrement with overflow handling
        // 2 => {
        //     init_election(server_addresses, load);
        //     -1
        // },
        _ => {
            eprintln!("Invalid operation flag received from client");
            -2
        }
    }
}

fn init_election(server_addresses: &Vec<SocketAddr>, load: &i64) {
    // for &server_address in server_addresses {

    // }
}

// async fn handle_client(server_addresses: &Vec<SocketAddr>, socket: UdpSocket) -> Result<(), Box<dyn std::error::Error>> {
async fn handle_client(socket: &UdpSocket) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let (client_address, data) = read_request(&socket).await?;
        let operation_flag = data[0];
        let number = i64::from_be_bytes(data[1..].try_into().unwrap());
        let new_result = process_request(&operation_flag, &number);
        println!("Result is: {}", new_result);
        send_response(&socket, &client_address, &new_result.to_be_bytes().to_vec()).await?;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let socket = UdpSocket::bind("127.0.0.1:8080").await?;

    let socket = Arc::new(socket);
    
    println!("Server listening on 127.0.0.1:8080...");
    
    let num_threads = 4; // Number of threads to handle clients
    
    for _ in 0..num_threads {
        let socket_clone =Arc::clone(&socket);
        tokio::spawn(async move {
            if let Err(err) = handle_client(&socket_clone).await {
                eprintln!("Error in handle_client: {}", err);
            }
        });
    }
    

    // for _ in 0..num_threads {
    //     let cloned_socket = Arc::clone(&socket_clone);
    //     tokio::spawn(handle_client(cloned_socket));
    // } 

    // Block the main thread to keep the program running
    for _ in 0..num_threads {
        thread::park();
    }

    Ok(())
}

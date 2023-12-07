use std::net::{UdpSocket, SocketAddr, ToSocketAddrs};
use std::convert::TryInto;
use std::thread;
use std::time::{Duration, Instant};

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
fn read_response(socket: &UdpSocket) -> ServerReply {
    let mut buffer = vec![0; 1024];
    // Receive the result from the server
    let (length, server_addr) = socket
        .recv_from(&mut buffer)
        .expect("Failed to receive data from server");
    
    // receive a flag to indicate the type of reply, 0 is election result (server address), 1 is the processed request's result
    let reply_flag = buffer[0];

    // returning the appropriate result
    match reply_flag {
        0 => return ServerReply::Address(server_addr),
        1 => {
            return ServerReply::Data(buffer[1..length].to_vec());
        },
        _ => ServerReply::None
    }
}

// handling election result or processed request
fn handle_response(server_reply: &ServerReply, socket: &UdpSocket, data: &[u8]) {
    match server_reply {
        ServerReply::Address(address) => {
            send_request_to_server(socket, &address, data);
            let server_reply: ServerReply = read_response(socket);
            handle_response(&server_reply, socket, data);
        },
        ServerReply::Data(reply_data) => {
            println!("Server replied with {:?}", i64::from_be_bytes(reply_data.as_slice().try_into().unwrap()));
        },
        ServerReply::None => {}
    }
}

fn main() {
    let number_of_requests = 5;
    let delay_duration = Duration::from_secs(1);
    let mut total_duration = Duration::new(0, 0);

    let client_socket = UdpSocket::bind("0.0.0.0:0").expect("failed to bind socket");

    let mut server_addresses = Vec::new();
    server_addresses.push("127.0.0.1:8080".to_socket_addrs().unwrap().next().unwrap());
    server_addresses.push("127.0.0.1:8081".to_socket_addrs().unwrap().next().unwrap());
    server_addresses.push("127.0.0.1:8082".to_socket_addrs().unwrap().next().unwrap());

    for i in 0..number_of_requests {
        let iteration_start = Instant::now();

        let number_to_increment:i64 = 102;
        let request = bundle_request(&0, &number_to_increment);
        send_request_to_servers(&client_socket, &server_addresses, &request);
        let server_reply: ServerReply = read_response(&client_socket);
        handle_response(&server_reply, &client_socket, &request);

        let number_to_decrement:i64 = 110;
        let request = bundle_request(&1, &number_to_decrement);
        send_request_to_servers(&client_socket, &server_addresses, &request);
        let server_reply = read_response(&client_socket);
        handle_response(&server_reply, &client_socket, &request);

        let iteration_time = iteration_start.elapsed();
        total_duration += iteration_time;

        println!("Iteration {} took: {:?}", i + 1, iteration_time);

        // Add a delay between requests for better synchronization
        thread::sleep(delay_duration);
    }

    // Calculate and print the average duration
    let average_duration = total_duration / number_of_requests as u32;
    println!("Average iteration time: {:?}", average_duration);
}

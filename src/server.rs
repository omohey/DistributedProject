// server.rs

use std::convert::TryInto;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn read_request(stream: &mut TcpStream) -> (u8, i64) {
    let mut buffer = [0; 9]; // Changed to 9 to accommodate increment/decrement flag and i64

    // Read the request from the client
    if let Err(e) = stream.read_exact(&mut buffer) {
        eprintln!("Error reading from client: {}", e);
        return (2, 0); // Return an error flag and default value
    }

    let operation_flag = buffer[0];
    let result = i64::from_be_bytes(buffer[1..].try_into().unwrap_or_else(|e| {
        eprintln!("Error converting bytes to number: {}", e);
        [0; 8] // Default to 0 in case of an error
    }));

    (operation_flag, result)
}

fn process_request(operation_flag: u8, number: i64) -> i64 {
    match operation_flag {
        0 => number.wrapping_add(1), // Increment
        1 => number.wrapping_sub(1), // Decrement
        _ => {
            eprintln!("Invalid operation flag received from client");
            0
        }
    }
}

fn send_response(stream: &mut TcpStream, response: i64) {
    // Send the result back to the client
    if let Err(e) = stream.write_all(&response.to_be_bytes()) {
        eprintln!("Error writing to client: {}", e);
    }
}

fn handle_client(mut stream: TcpStream) {
    let (operation_flag, number) = read_request(&mut stream);

    // Process the request
    let new_result = process_request(operation_flag, number);

    // Send the result back to the client
    send_response(&mut stream, new_result);
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").expect("Failed to bind to address");

    println!("Server listening on 127.0.0.1:8080...");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // Spawn a new thread for each client connection
                thread::spawn(|| handle_client(stream));
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}

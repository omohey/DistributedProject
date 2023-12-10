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


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let server1_address = "127.0.0.1:8081";
    let server2_address = "127.0.0.2:8081";
    let server3_address = "127.0.0.3:8081";

    let socket = UdpSocket::bind("127.0.0.1:0")?; // Binding to 0 allows the OS to choose an available port
    // let c_socket_clone = UdpSocket::bind("127.0.0.1:0")?;
    // create a socket with port number equal to socket + 1
    let listening_socket = UdpSocket::bind("127.0.0.1:".to_string() + &(socket.local_addr()?.port() + 1).to_string())?;
    println!("Client started on port {}", socket.local_addr()?);
    println!("Listening socket started on port {}", listening_socket.local_addr()?);


    // measure the time it takes to do the election
    let start = Instant::now();
    for i in 0..1000 {
        
        // do election
        let input:u8 = 0;
        let mut buffer = Vec::new();
        buffer.push(input);

        // println!("Sending request to server {}: {}", server1_address, input);

        socket.send_to(&input.to_ne_bytes(), server1_address.to_socket_addrs().unwrap().next().unwrap()); 
        socket.send_to(&input.to_ne_bytes(), server2_address.to_socket_addrs().unwrap().next().unwrap());
        socket.send_to(&input.to_ne_bytes(), server3_address.to_socket_addrs().unwrap().next().unwrap());



        

        let mut buffer1 = [0; 1024];
        let (size, source) = socket.recv_from(&mut buffer1)?;

        let response1 = String::from_utf8_lossy(&buffer1);

        // println!("Received response from server: {}", response1);
        // println!("The server address is: {}", source);
        println!("{}", i);

        // add timeout 1ms
        let timeout = Duration::from_millis(12);
        thread::sleep(timeout);

    }
    //22 61 11 54 26 10 27 4 19 14 8 13 25 2 8 15


    let duration = start.elapsed();
    println!("Time elapsed in expensive_function() is: {:?}", duration);

    

    Ok(())
}

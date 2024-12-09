use std::{collections::HashMap, io::ErrorKind, net::UdpSocket};

pub struct Server {
}

impl Server {
    pub fn run() {
        // First, create a socket for UDP connection, binding to a device-selected port on localhost
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to create socket");
    socket.set_nonblocking(true).expect("Failed to set non-blocking mode");

    // Display server information
    let local_addr = socket.local_addr().expect("Failed to get local address");
    println!("Server started at:\n - IP: {}:{}", local_addr.ip(), local_addr.port());
    println!("Messages:");

    // Create a buffer for sending data
    let mut buf = [0; 1024];

    let mut connections: HashMap<String, String> = HashMap::new();

    loop {
        // Receive data from clients
        let (amt, src) = match socket.recv_from(&mut buf) {
            Ok((amt, src)) => (amt, src),
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => {
                // No data available at the moment, try again later
                continue;
            },
            Err(err) => {
                // Handle other errors
                println!("Error: {}", err);
                break;
            }
        };

        println!("{}", src);

        // Convert the received data to a string and send a response
        let received = std::str::from_utf8(&buf[..amt]).expect("Invalid UTF-8 data");
        socket.send_to("Ok".as_bytes(), src).expect("Failed to send data");

        // Insert the received data into the connections HashMap
        connections.insert(src.to_string(), received.to_string());

        // Send data to all connected clients
        for connection in &connections {
            socket
                .send_to(
                    serde_json::to_string(&connections)
                        .unwrap()
                        .as_bytes(),
                    connection.0,
                )
                .expect("Failed to send data");
        }

    }
    }
}
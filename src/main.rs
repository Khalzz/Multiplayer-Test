use app::App;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::net::{SocketAddr, UdpSocket};
use std::{io, string, thread, time};

mod app;
mod game_object;

mod ui {
    pub mod text;
}

mod input {
    pub mod button_module;
}

mod gameplay {
    pub mod play;
}

struct Position {
    src: SocketAddr,
    data: String
}

fn main() -> Result<(), String> {
    print!("{}[2J", 27 as char);

    println!("You want to do a [server] or a [client]");
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
        
    if input.trim() == "server"{
        server()
    } else if input.trim() == "client" {
        println!("Enter the IP to connect");
        let mut ip = String::new();
        io::stdin().read_line(&mut ip).expect("Failed to read line");

        if ip.trim() != "" {
            let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to create socket");
            socket.set_nonblocking(true).expect("Failed to set non-blocking mode");

            let local_addr = socket.local_addr().expect("Failed to get local address");
            println!("Client started at:\n - ip: {}:{}", local_addr.ip(), local_addr.port());
        
            socket.send_to("Connected".as_bytes(), ip.trim().to_owned()).expect("Failed to send data");
        
            let app = App::new("Arrowner", socket, ip.trim().to_owned());
            app.render();
        
        }
    }

    Ok(())
}

fn server() {
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

    // Set tick rate to 60 ticks per second

    // Start the main server loop
    loop {
        let start_time = time::Instant::now();

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
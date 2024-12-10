use std::{collections::HashMap, io::ErrorKind, net::{SocketAddr, UdpSocket}, time::Instant};

use crate::{engine::time::Timing, gameplay::server_game_logic::ServerGameLogic};


/// # Server
/// Server is a struct that will handle the creation and data obtaining from the clients, general elements of the server like timing, and more data.
pub struct Server {
    pub time: Timing,
    pub connections: HashMap<String, String>,
    pub last_data_sent: String,
}

impl Server {
    const TICK_RATE: f32 = 60.0;

    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
            time: Timing::new(),
             last_data_sent: "".to_string()
        }
    }

    pub fn run(&mut self) {
        match UdpSocket::bind("0.0.0.0:0") {
            Ok(socket) => {
                socket.set_nonblocking(true).expect("Failed to set non-blocking mode");
                
                Self::server_init_info(&socket);
                
                // Create a buffer for sending data
                let mut buf = [0; 1024];

                let mut server_game_logic = ServerGameLogic::new();
                let mut time_step = Instant::now();
            
                loop {
                    if time_step.elapsed().as_secs_f32() >= (1.0 / Server::TICK_RATE) {
                        time_step = Instant::now();
                        self.time.update();
                        server_game_logic.update(self);

                        // recieve data from the client (load the client to "connection" if its the first time getting data from them)
                        match socket.recv_from(&mut buf) {
                            Ok((amt, src)) => {
                                // Convert the received data to a string and send a response
                                let received = std::str::from_utf8(&buf[..amt]).expect("Invalid UTF-8 data");
                                socket.send_to("Ok".as_bytes(), src).expect("Failed to send data");
                        
                                // Insert the received data into the connections HashMap
                                match self.connections.get_mut(&src.to_string()) {
                                    Some(existent_connection) => {
                                        *existent_connection = received.to_string();
                                    },
                                    None => {
                                        println!("The user {} has connected to the server", &src.to_string());
                                        self.connections.insert(src.to_string(), received.to_string());
                                    },
                                }
                            },
                            Err(ref err) if err.kind() == ErrorKind::WouldBlock => {
                                // println!("this will jump everything if there is no data");
                            },
                            Err(err) => {
                                // Handle other errors
                                println!("Error: {}", err);
                                break;
                            }
                        }

                        // update server game logic and send positions to all clients
                        let data_to_send = serde_json::to_string(&server_game_logic.returnable).unwrap();

                        if data_to_send != self.last_data_sent {
                            // Send data to all connected clients
                            for connection in &self.connections {
                                socket
                                    .send_to(
                                        data_to_send.as_bytes(),
                                        connection.0,
                                    )
                                    .expect("Failed to send data");
                                self.last_data_sent = data_to_send.clone();
                            }
                        }
                    } 
                }
            },
            Err(err) => eprintln!("The binding of the ip to a udp socket was not successfull: {}", err),
        }
    }

    fn server_init_info(socket: &UdpSocket) {
        match socket.local_addr() {
            Ok(local_address) => {
                println!("Server started at:\n - IP: {}:{}", local_address.ip(), local_address.port());
            },
            Err(_) => todo!(),
        }
        println!("Messages:");
    
    }
}
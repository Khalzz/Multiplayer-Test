use app::App;
use networking::server::Server;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::net::{SocketAddr, UdpSocket};
use std::{io, string, thread, time};

mod app;
mod game_object;

mod ui {
    pub mod text;
}

mod engine {
    pub mod time;
}

mod input {
    pub mod button_module;
}

mod gameplay {
    pub mod play;
    pub mod server_game_logic;
}

mod networking {
    pub mod server;
}

fn main() -> Result<(), String> {
    print!("{}[2J", 27 as char); // clean the terminal

    println!("You want to do a [server] or a [client]");
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
        
    if input.trim() == "server"{
        let mut server = Server::new();
        server.run();
    } else if input.trim() == "client" {
        println!("Enter the IP to connect");
        let mut ip = String::new();

        match io::stdin().read_line(&mut ip) {
            Ok(_) => {
                if ip.trim() != "" {
                    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to create socket");
                    socket.set_nonblocking(true).expect("Failed to set non-blocking mode");
        
                    let local_addr = socket.local_addr().expect("Failed to get local address");
                    println!("Client started at:\n - ip: {}:{}", local_addr.ip(), local_addr.port());
                
                    socket.send_to("Connected".as_bytes(), ip.trim().to_owned()).expect("Failed to send data");
                    
                    let app = App::new("Multiplayer Testing", socket, ip.trim().to_owned());
                    app.render();
                }
            },
            Err(error) => eprintln!("Something went wrong: {}", error),
        }

        
    }

    Ok(())
}
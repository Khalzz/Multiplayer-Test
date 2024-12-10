use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::networking::server::Server;

use super::play::Packet;

#[derive(Serialize, Deserialize, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Serialize, Deserialize)]
pub struct Returnable {
    pub players_data: HashMap<String, Position>
}

/// # Server Game Logic
/// This struct is defined to set all the game logic that will be setted from the server, including stuff like movement, and more based entirely on user input.
/// This logic is called PER USER, so we can return data like "position, states and more" from the user itself, and based on that 

pub struct ServerGameLogic {
    pub returnable: Returnable
}

impl ServerGameLogic {
    pub fn new() -> Self {
        Self {
            returnable: Returnable {
                players_data: HashMap::new()
            }
        }
    }

    /// # Update
    /// This function is called once per each connection in the server so we can handle the logic of each connection element.
    /// 
    /// ## Params:
    /// - Connection_id: The client id of the user connected to the server.
    /// 
    /// ## Returns:
    /// - Position of the user to send the client
    pub fn update(&mut self, server: &mut Server) {
        for (key, data) in &server.connections {
            let deserialized_data: Result<Packet, serde_json::Error> = serde_json::from_str(&data);

            match deserialized_data {
                Ok(usable_data) => {
                    match self.returnable.players_data.get_mut(&usable_data.id) {
                        Some(existent_player) => {
                            if usable_data.controls.right {
                                existent_player.x += 200.0 * server.time.delta_time;
                            }
                            if usable_data.controls.left {
                                existent_player.x -= 200.0 * server.time.delta_time;
                            }
                            if usable_data.controls.up {
                                existent_player.y -= 200.0 * server.time.delta_time;
                            }
                            if usable_data.controls.down {
                                existent_player.y += 200.0 * server.time.delta_time;
                            }
                        },
                        None => {
                            self.returnable.players_data.insert(usable_data.id, Position { x: 0.0, y: 0.0 });
                        },
                    }
                },
                Err(err) => eprintln!("The value of: {} was not deserialized succesfully, and the reason was: {}", data, err),
            }
        }
    }
}
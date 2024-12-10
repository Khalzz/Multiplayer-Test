use std::{collections::HashMap, net::{IpAddr, SocketAddr}, time::{Duration, Instant}};

use rand::{distributions::Alphanumeric, Rng};
use sdl2::{event::Event, keyboard::Keycode, pixels::Color, rect::Rect, ttf::Font};
use serde::{Deserialize, Serialize};
use crate::{app::{App, AppState}, game_object::GameObject, gameplay::server_game_logic::Position, input::button_module::{Button, TextAlign}};

pub struct GameLogic { // here we define the data we use on our script
    last_frame: Instant,
    pub start_time: Instant,
    ui_elements: Vec<Button>,
    frame_count: u32,
    frame_timer: Duration,
    fps: u32,
    controls: Controls,
    send_packet: Instant,
    
    // networking
    pub last_packet_sent: Option<Packet>,
    pub instance_id: String, // this value is for id-ing the client instance
    players: HashMap<String, GameObject>

} 

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct Controls {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool
}

#[derive(Serialize, Deserialize, PartialEq)]
pub struct Packet {
    pub controls: Controls,
    pub id: String
}

impl GameLogic {
    // this is called once
    pub fn new(app: &mut App) -> Self {
        // UI ELEMENT
        let ui_points = Button::new(GameObject { active: true, x:((app.width/2) - 70 ) as f32, y: 10.0, width: 140.0, height: 30.0}, Some(String::from("Points")),Color::RGB(200, 100, 100), Color::WHITE, Color::RGB(200, 10, 0), Color::RGB(200, 0, 0),None, TextAlign::Center);
        let timer = Button::new(GameObject {active: true, x:10 as f32, y: 30.0, width: 0.0, height: 0.0},Some(String::from("Timer")),Color::RGB(100, 100, 100),Color::WHITE,Color::RGB(0, 200, 0),Color::RGB(0, 0, 0),None, TextAlign::Left);
        let framerate = Button::new(GameObject {active: true, x:10 as f32, y: 10.0, width: 0.0, height: 0.0},Some(String::from("Framerate")),Color::RGBA(100, 100, 100, 0),Color::WHITE,Color::RGB(0, 200, 0),Color::RGB(0, 0, 0),None, TextAlign::Left);

        // UI LISTS
        let ui_elements = vec![ui_points, timer, framerate];

        // we will send this value to the server so we can make differenciation between the user instance and the other players
        // this value should be added based entirely on the connection of a new game
        let instance_id: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();

        Self {
            last_frame: Instant::now(),
            start_time: Instant::now(),
            frame_count: 0,
            frame_timer: Duration::new(0, 0),
            fps: 0,
            ui_elements,
            controls: Controls {
                left: false,
                right: false,
                up: false,
                down: false,
            },
            send_packet: Instant::now(),
            last_packet_sent: None,
            instance_id,
            players: HashMap::new()
        }
    }

    // this is called every frame
    pub fn update(&mut self, _font: &Font, mut app_state: &mut AppState, mut event_pump: &mut sdl2::EventPump, app: &mut App) {
        self.display_framerate(app);
        self.ui_elements[2].render(&mut app.canvas, &app.texture_creator, _font);

        // create the packet to send
        let packet = Packet {
            controls: self.controls.clone(),
            id: self.instance_id.clone()
        };

        // send the packet or not based on the state of the packet itself
        match &self.last_packet_sent {
            Some(last_packet) => {
                if last_packet != &packet {
                    self.send_packet(app, &packet);
                    self.last_packet_sent = Some(packet);
                }
            },
            None => {
                self.send_packet(app, &packet);
                self.last_packet_sent = Some(packet);
            },
        }

        match &app.received {
            Some(returned) => {
                for (id, position) in &returned.players_data {
                    match self.players.get_mut(id) {
                        Some(player) => {
                            // if the player exists, just change the position of itself
                            if *id == self.instance_id {
                                app.canvas.set_draw_color(Color::RGB(100, 100, 200));
                            } else {
                                app.canvas.set_draw_color(Color::RGB(100, 100, 100));
                            }

                            player.x = position.x;
                            player.y = position.y;
                            app.canvas.fill_rect(Rect::new(player.x as i32, player.y as i32, player.width as u32, player.height as u32)).unwrap();
                        },
                        None => {
                            // if the player dont exists, instance it in the map
                            self.players.insert(id.to_string(), GameObject {
                                active: true,
                                x: position.x,
                                y: position.y,
                                width: 40.0,
                                height: 40.0,
                            });
                        },
                    }

                }
            },
            None => {},
        }

        Self::event_handler(self, &mut app_state, &mut event_pump);
    }

    fn event_handler(&mut self, app_state: &mut AppState, event_pump: &mut sdl2::EventPump) {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::KeyDown { keycode, .. } => {
                    match keycode {
                        Some(key) => {
                            match key {
                                Keycode::Up     => self.controls.up     = true,
                                Keycode::Down   => self.controls.down   = true,
                                Keycode::Left   => self.controls.left   = true,
                                Keycode::Right  => self.controls.right  = true,
                                
                                _ => {}
                            }
                        },
                        None => {},
                    }
                },
                sdl2::event::Event::KeyUp { keycode, .. } => {
                    match keycode {
                        Some(key) => {
                            match key {
                                Keycode::Up     => self.controls.up     = false,
                                Keycode::Down   => self.controls.down   = false,
                                Keycode::Left   => self.controls.left   = false,
                                Keycode::Right  => self.controls.right  = false,
                                _ => {}
                            }
                        },
                        None => {},
                    }
                },
                Event::Quit { .. } => {
                    app_state.is_running = false;
                } 
                _ => {}
            }
        }
    }


    // Test to instead of sending the position, just send the controllers state and let the server do the other stuff

    // the controler packet will only be sent if there is a change on the controller struct
    fn send_packet(&mut self, app: &mut App, packet: &Packet) {
        app.socket.send_to(serde_json::to_string(&packet).unwrap().as_bytes(), app.connect_to.to_owned()).unwrap();
        self.send_packet = Instant::now()
    }

    // Test instead of sending info every time the player presses a button only a certian amount of times each second
    /* 
    fn send_position(&mut self, app: &mut App) {
        let sendable = Packet {
            x: self.player.x,
            y: self.player.y,
        };

    if self.send_packet.elapsed().as_millis() > 1000/60 {
        app.socket.send_to(serde_json::to_string(&sendable).unwrap().as_bytes(), app.connect_to.to_owned()).unwrap();
        self.send_packet = Instant::now()
    }
    }
    */

    fn display_framerate(&mut self, app: &mut App) {
        // Render FPS text
        let fps_text = format!("FPS: {}", app.time.get_fps());
        self.ui_elements[2].text = Some(fps_text);
    }
}
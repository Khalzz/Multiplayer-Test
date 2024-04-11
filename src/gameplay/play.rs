use std::{net::{IpAddr, SocketAddr}, time::{Duration, Instant}};

use sdl2::{event::Event, keyboard::Keycode, pixels::Color, rect::Rect, ttf::Font};
use serde::{Deserialize, Serialize};
use crate::{app::{App, AppState}, game_object::GameObject, input::button_module::{Button, TextAlign}};

pub struct GameLogic { // here we define the data we use on our script
    last_frame: Instant,
    pub start_time: Instant,
    ui_elements: Vec<Button>,
    frame_count: u32,
    frame_timer: Duration,
    fps: u32,
    player: GameObject,
    controls: Controls,
    send_packet: Instant
} 

pub struct Controls {
    left: bool,
    right: bool,
    up: bool,
    down: bool
}

#[derive(Serialize, Deserialize)]
pub struct Packet {
    x: f32,
    y: f32,
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
            player: GameObject {
                active: true,
                x: 10.0,
                y: 10.0,
                width: 40.0,
                height: 40.0,
            },
            send_packet: Instant::now()
        }
    }

    // this is called every frame
    pub fn update(&mut self, _font: &Font, mut app_state: &mut AppState, mut event_pump: &mut sdl2::EventPump, app: &mut App) {
        
        let delta_time = self.delta_time();
        self.display_framerate(delta_time);
        self.ui_elements[2].render(&mut app.canvas, &app.texture_creator, _font);


        if self.controls.up {
            self.player.y -= 500.0 * delta_time.as_secs_f32();
            self.send_position(app);
        } 
        if self.controls.down {
            self.player.y += 500.0 * delta_time.as_secs_f32();
            self.send_position(app);

        }
        if self.controls.left {
            self.player.x -= 500.0 * delta_time.as_secs_f32();
            self.send_position(app);

        } 
        if self.controls.right {
            self.player.x += 500.0 * delta_time.as_secs_f32();
            self.send_position(app);

        }

        app.canvas.set_draw_color(Color::RGB(100, 100, 100)); // it must be a Color::RGB() or other

        match &app.received {
            Some(connections) => {
                for data in connections {
                    let local_addr = app.socket.local_addr().expect("Failed to get local address");

                    if data.0 != &format!("{}:{}", local_addr.ip(), local_addr.port()) {
                        let splitted: Vec<&str> = data.1.split("-").collect();
                        // println!("{}", splitted[0]);
                        match serde_json::from_str::<Packet>(&data.1) {
                            Ok(deserialized) => {
                                // send this only 60 times per second
                                app.canvas.fill_rect(Rect::new(deserialized.x as i32, deserialized.y as i32, self.player.width as u32, self.player.height as u32)).unwrap();
                            },
                            Err(_) => {},
                        }
                    }
                }
            },
            None => {},
        }
        app.canvas.fill_rect(Rect::new(self.player.x as i32, self.player.y as i32, self.player.width as u32, self.player.height as u32)).unwrap();

        Self::event_handler(self, &mut app_state, &mut event_pump);
    }

    fn event_handler(&mut self, app_state: &mut AppState, event_pump: &mut sdl2::EventPump) {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::KeyDown { keycode: Some(Keycode::Up), .. } => {
                    self.controls.up = true;
                },
                sdl2::event::Event::KeyUp { keycode: Some(Keycode::Up), .. } => {
                    self.controls.up = false;
                },
                sdl2::event::Event::KeyDown { keycode: Some(Keycode::Down), .. } => {
                    self.controls.down = true;
                },
                sdl2::event::Event::KeyUp { keycode: Some(Keycode::Down), .. } => {
                    self.controls.down = false;
                },
                sdl2::event::Event::KeyDown { keycode: Some(Keycode::Left), .. } => {
                    self.controls.left = true;
                },
                sdl2::event::Event::KeyUp { keycode: Some(Keycode::Left), .. } => {
                    self.controls.left = false;
                },
                sdl2::event::Event::KeyDown { keycode: Some(Keycode::Right), .. } => {
                    self.controls.right = true;
                },
                sdl2::event::Event::KeyUp { keycode: Some(Keycode::Right), .. } => {
                    self.controls.right = false;
                },
                Event::KeyDown { keycode: Some(Keycode::Escape), .. }  => {
                    app_state.is_running = false;
                }, Event::Quit { .. } => {
                    app_state.is_running = false;
                } 
                _ => {}
            }
        }
    }


    // Test to instead of sending the position, just send the controllers state and let the server do the other stuff


    // Test instead of sending info every time the player presses a button only a certian amount of times each second
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

    fn display_framerate(&mut self, delta_time: Duration) {
        self.frame_count += 1;
        self.frame_timer += delta_time;

        // Calculate FPS every second
        if self.frame_timer >= Duration::from_secs(1) {
            self.fps = self.frame_count;
            self.frame_count = 0;
            self.frame_timer -= Duration::from_secs(1); // Remove one second from the timer
        }

        // Render FPS text
        let fps_text = format!("FPS: {}", self.fps);
        self.ui_elements[2].text = Some(fps_text);
    }

    fn delta_time(&mut self) -> Duration {
        let current_time = Instant::now();
        let delta_time = current_time.duration_since(self.last_frame); // this is our Time.deltatime
        self.last_frame = current_time;
        return delta_time
    }
}
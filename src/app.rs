use std::collections::HashMap;
use std::env;

use std::io::ErrorKind;
use std::net::UdpSocket;
use sdl2::pixels::Color;
use sdl2::render::TextureCreator;
use sdl2::video::WindowContext;
use sdl2::{video::Window, Sdl, render::Canvas};
use serde::{Deserialize, Serialize};
use crate::gameplay::play;

pub enum GameState {
    Playing,
}

pub struct AppState {
    pub is_running: bool,
    pub state: GameState,
}

pub struct ConnectionData {
    pub socket: UdpSocket,
    pub connect_to: String
}

pub struct App {
    pub context: Sdl,
    pub width: u32,
    pub height: u32,
    pub canvas: Canvas<Window>,
    pub texture_creator: TextureCreator<WindowContext>,
    pub socket: UdpSocket,
    pub connect_to: String,
    pub received: Option<HashMap<String, String>>
}

impl App {
    pub fn new(title: &str, socket: UdpSocket, connect_to: String) -> App{
        // base sdl2
        let context = sdl2::init().expect("SDL2 wasn't initialized");
        let video_susbsystem = context.video().expect("The Video subsystem wasn't initialized");

        let current_display = video_susbsystem.current_display_mode(0).unwrap();
        
        let width = current_display.w as u32;
        let height = current_display.h as u32;

        env::set_var("SDL_VIDEO_MINIMIZE_ON_FOCUS_LOSS", "0"); // this is highly needed so the sdl2 can alt tab without generating bugs

        let window = video_susbsystem.window(title, 1280, 720 as u32).vulkan().build().expect("The window wasn't created");
        let mut canvas = window.into_canvas().accelerated().present_vsync().build().expect("the canvas wasn't builded");
        
        canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
        let texture_creator = canvas.texture_creator();

        App {
            context,
            width,
            height,
            canvas,
            texture_creator,
            socket,
            connect_to,
            received: None
        }
    }

    

    pub fn render(mut self) {
        let mut app_state = AppState { is_running: true, state: GameState::Playing };
        let mut event_pump = self.context.event_pump().unwrap();

        let ttf_context = sdl2::ttf::init().unwrap(); // we create a "context"
        let use_font = "./assets/fonts/Inter-Thin.ttf";
        let mut _font = ttf_context.load_font(use_font, 20).unwrap();

        let mut play = play::GameLogic::new(&mut self);

        // networking
        let mut buf = [0; 1024];

        while app_state.is_running {
            
            self.canvas.set_draw_color(Color::RGBA(40, 40, 40, 100));
            self.canvas.clear();

            match self.socket.recv_from(&mut buf) {
                Ok((amt, src)) => {
                    // println!("Received {} bytes from {}", amt, src);
                    let received = std::str::from_utf8(&buf[..amt]).unwrap();
                    match serde_json::from_str(&received) {
                        Ok(deserialized) => {
                            self.received = deserialized
                        },
                        Err(_) => {},
                    }
                },
                Err(ref err) if err.kind() == ErrorKind::WouldBlock => {
                    // No data available at the moment, try again later
                },
                Err(err) => {
                    // Handle other errors
                    println!("Error: {}", err);
                    break;
                }
            }
    
            
            
            match app_state.state {
                GameState::Playing => {
                    play.update(&_font, &mut app_state, &mut event_pump, &mut self);
                },
            }
            self.canvas.present();

            
        }
    }
}
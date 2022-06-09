// TODO 

// 1. 
//learn more about euclid crate, seems really useful
// use euclid::*;

// 2. 
// remember to create a binary (use booleans) 2d array from the 2d environnment array - substantially more efficient than working with 3 numbers (RGB) for each look
// i.e. do the math during initialization vs each iteration


use ::image::{open, DynamicImage, ImageResult, ImageBuffer};
use rand::Rng;
use sdl2::event::Event;
use sdl2::image::{self, InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{Texture, WindowCanvas};
use std::time::{Duration, SystemTime};
use std::io;

const BOT_WIDTH: i32 = 15;
const BOT_HEIGHT: i32 = 10;
const BOT_STEP_SIZE: i32 = 10;
const LOCK_TO_60_FPR: bool = true;
const NUM_BOTS: i32 = 100;
const FPS: i32 = 30; 

#[derive(Debug)]
struct Map {
    num_rows: i32,
    num_cols: i32,
    pixel_stream: Vec<u8>,
    environment_grid: Vec<Vec<(u8, u8, u8)>>,
}

// TODO change this function into Map::new(stream: Vec<u8>, height: i32, width: i32)
fn build_map(stream: Vec<u8>, height: i32, width: i32) -> Map {
    Map {
        num_rows: height,
        num_cols: width,
        environment_grid: Map::convert_stream_to_3d_array(&stream, height, width),
        pixel_stream: stream,
    }
}

impl Map {
    fn get_pixel_rgb(&self, x: i32, y: i32) -> (u8, u8, u8) {
        let offset: i32 = (y * self.num_rows + x) * 3;
        let r = self.pixel_stream[offset as usize];
        let g = self.pixel_stream[(offset + 1) as usize];
        let b = self.pixel_stream[offset as usize + 2];
        (r, g, b)
    }

    fn set_pixel_rgb(&mut self, x: i32, y: i32, (r, g, b): (u8, u8, u8)) {
        // TODO push existing RGB onto pixel history if 
        let offset: i32 = (y * self.num_rows + x) * 3;
        self.pixel_stream[offset as usize] = r;
        self.pixel_stream[(offset + 1) as usize] = g;
        self.pixel_stream[offset as usize + 2] = b;
    }

    fn convert_stream_to_3d_array(stream: &Vec<u8>, height: i32, width: i32) -> Vec<Vec<(u8, u8, u8)>> {
        let mut return_vec: Vec<Vec<(u8, u8, u8)>> = Vec::new();
        for row in 0..height {
            let mut row_vec: Vec<(u8, u8, u8)> = Vec::new();
            for col in 0..width {
                let offset: i32 = (row * width + col) * 3;
                let r = stream[offset as usize];
                let g = stream[(offset + 1) as usize];
                let b = stream[offset as usize + 2];
                row_vec.push((r, g, b));
            }
            return_vec.push(row_vec);
        }
        return_vec
    }

}

#[derive(Debug)]
struct Bot {
    id: i32, // could go with &str here, but requires knowledge of lifetime specifiers
    x: i32,
    y: i32,
    color: Color,
    direction: f64,
    path: Vec<(i32, i32)>,
    // path_draw_buffer: Vec<>,
}

impl Bot {
    fn explore(&mut self, map: &Map) {
        let mut direction_change: f64 = 0.0;
        let mut max_change = 0.5;
        loop {
            let direction_change: f64 = rand::thread_rng().gen_range(-max_change..=max_change);
            let delta_x = (self.direction + direction_change).cos() * BOT_STEP_SIZE as f64;
            let proposed_x = self.x + delta_x as i32;
            let delta_y = (self.direction + direction_change).sin() * BOT_STEP_SIZE as f64;
            let proposed_y = self.y + delta_y as i32;

            // println!("proposed points x={} y={}", proposed_x, proposed_y);
            if 0 > proposed_y || proposed_y >= map.num_rows || 0 > proposed_x || proposed_x >= map.num_cols {
                max_change += 0.05;
                continue
            }

            let (r, g, b) = map.get_pixel_rgb(proposed_x, proposed_y);
            if r + g + b <= 30{
                self.direction = self.direction + direction_change;
                self.x = proposed_x;
                self.y = proposed_y;
                break;
            }
            else {
                max_change = max_change + 0.1;
            }
        };
        // TODO check for destination within certain proximity 
    
    }
}




fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}

fn render(canvas: &mut WindowCanvas, texture: &mut Texture, bots: &Vec<Bot>, rect: Rect, start_time: SystemTime, pixel_data: &Vec<u8>) -> Result<(), String> {
    canvas.clear();
    texture.update(rect, &pixel_data, (3*750) as usize);
    canvas.copy(&texture, None, None)?;
    

    for bot in bots {
        canvas.set_draw_color(bot.color);
        canvas
            .fill_rect(Rect::new(
                bot.x - BOT_WIDTH / 2,
                bot.y - BOT_HEIGHT / 2,
                BOT_WIDTH as u32,
                BOT_HEIGHT as u32,
            ))
            .expect("Couldn't draw rect for bot");
    }
    
    // Update window title with FPS
    if let Ok(elapsed) = start_time.elapsed() {
        let window = canvas.window_mut();
        let title = format!(
            "Swarm Pathfinding - FPS: {}", 1_000_000_000 / elapsed.as_nanos()
        );
        window.set_title(&title).map_err(|e| e.to_string())?;
    }
    
    canvas.present();

    Ok(())
}

// fn get_pixel_in_array(array: Vec<u8, Global>) {

// }

fn main() -> Result<(), String> {
    let val: f64 = std::f64::consts::PI;
    println!("{}", val.cos());

    // Read in image file and unpack
    let rgb8 = open("assets/environment1.png").unwrap().into_rgb8();
    println!("Analyzed width is {}, height is {}", rgb8.width(), rgb8.height());
    
    print_type_of(&rgb8);

    let mut pixel_data = rgb8.to_vec();
    let map: Map = build_map(pixel_data, rgb8.height() as i32, rgb8.width() as i32);

    // Init window and canvas
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let _image_context = image::init(InitFlag::PNG | InitFlag::JPG)?;
    let window = video_subsystem
        .window("Swarm Pathfinding", rgb8.width(), rgb8.height())
        .position_centered()
        .build()
        .expect("could not initialize video subsystem");

    let mut canvas = window
        .into_canvas()
        .build()
        .expect("could not make a canvas");

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, rgb8.width(), rgb8.height()).unwrap();
    let rect = Rect::new(0,0, rgb8.width(), rgb8.height());

    let starting_x = 100;
    let starting_y = 300;

    // Bot creation
    let bot_colors: Vec<Color> = Vec::new(); 
    let mut bots: Vec<Bot> = Vec::new();
    for i in 0..NUM_BOTS {
        let mut bot_color = Color::RGB(0, 0, 0);
        loop {
            // Produce unique color for bot
            let r = rand::thread_rng().gen_range(0..=255);
            let g = rand::thread_rng().gen_range(0..=255);
            let b = rand::thread_rng().gen_range(0..=255);
            bot_color = Color::RGB(r, g, b);
            if !bot_colors.contains(&bot_color) {
                break;
            }
        }
        bots.push({
            Bot {
                id: i,
                x: starting_x,
                y: starting_y,
                color: bot_color,
                direction: rand::thread_rng().gen_range(0.0..=(std::f64::consts::PI * 2.0)),
                path: Vec::new(),
            }
        });
    }

    // Thread creation


    
    // Main loop
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump()?;
    'running: loop {
        let start_time = SystemTime::now();
        // Check for exit condition (Escape key or window closed)
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'running;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => {
                    for bot in &mut bots {
                        bot.y -= 5;
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => {
                    for bot in &mut bots {
                        bot.y += 5;
                    }
                }
                _ => {}
            }
        }

        for bot in &mut bots {
            bot.explore(&map);
        }
        
        if LOCK_TO_60_FPR {
            // Sleep for a bit - FORCES 60 FRAMES A SECOND
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / FPS as u32));
        }

        // Very last step: Render
        render(&mut canvas, &mut texture, &bots, rect, start_time, &map.pixel_stream).expect("Woops");        
    }

    Ok(())
}

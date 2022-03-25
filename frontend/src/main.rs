use std::time::Duration;
use std::fs;
use nfd::Response;
use rand::Rng;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Mod};

use core::{Emulator, Error, Display};
use frontend::{create_audio_device, SquareWave};

const WINDOW_TITLE: &str = "CHIP-8";
const WINDOW_SIZE: u32 = 15;
const WIDTH: u32 = 64;
const HEIGHT: u32 = 32;
const FPS: u32 = 60;
const CPF: u32 = 700 / FPS;

const LIGHT_GREEN: Color = Color::RGB(80, 255, 80);
const LIGHT_YELLOW: Color = Color::RGB(255, 255, 80);
const LIGHT_RED: Color = Color::RGB(255, 80, 80);

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let filename = if args.len() >= 2 {
        args[1].clone()
    } else {
        let result = nfd::open_file_dialog(None, None)
            .unwrap_or_else(|e| panic!("{}", e));

        match result {
            Response::Okay(filename) => filename,
            Response::OkayMultiple(files) => files[0].clone(),
            Response::Cancel => return,
        }
    };

    if let Err(e) = run(&filename) {
        println!("{} {}", "Error:", e.to_string());
    }
}

fn run(filename: &str) -> Result<(), Error> {
    let program = fs::read(filename).map_err(|e| Error::from(e.to_string()))?;

    let mut rng = rand::thread_rng();
    let mut emulator = Emulator::new(|| rng.gen_range(0..=u8::MAX));

    emulator.load_program(&program);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window(WINDOW_TITLE, WIDTH * WINDOW_SIZE, HEIGHT * WINDOW_SIZE)
        .position_centered()
        .build().unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();


    let audio_device = create_audio_device(&audio_subsystem).unwrap();

    canvas.set_scale(WINDOW_SIZE as f32, WINDOW_SIZE as f32).unwrap();
    canvas.present();

    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_target(PixelFormatEnum::RGB24, WIDTH, HEIGHT).unwrap();

    let mut paused = false;
    let mut speed = 1.0;
    let mut color = Color::WHITE;
    let mut pixel_data = [0; (WIDTH * HEIGHT * 3) as usize];

    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'main,
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    paused = !paused;
                    let title = if paused {
                        format!("Paused · {}", WINDOW_TITLE)
                    } else {
                        WINDOW_TITLE.into()
                    };
                    canvas.window_mut().set_title(&title).unwrap();
                }
                Event::KeyDown { keymod, keycode: Some(keycode), .. } => {
                    if keymod.contains(Mod::LCTRLMOD) || keymod.contains(Mod::RCTRLMOD) {
                        match keycode {
                            Keycode::W if speed > 0.2 => speed -= 0.1,
                            Keycode::E if speed < 4.0 => speed += 0.1,
                            Keycode::C => {
                                color = match color {
                                    Color::WHITE => LIGHT_GREEN,
                                    LIGHT_GREEN => LIGHT_YELLOW,
                                    LIGHT_YELLOW => LIGHT_RED,
                                    _ => Color::WHITE,
                                }
                            }
                            Keycode::R => {
                                emulator.reset();
                                emulator.load_program(&program);
                            }
                            Keycode::Q => break 'main,
                            _ => {}
                        }
                    } else if let Some(key) = keycode_to_key(keycode) {
                        emulator.keydown(key);
                    }
                }
                Event::KeyUp { keycode: Some(keycode), .. } => {
                    if let Some(key) = keycode_to_key(keycode) {
                        emulator.keyup(key);
                    }
                }
                _ => {}
            }
        }

        if !paused {
            emulator.time_step();
            emulator.cycle((CPF as f32 * speed) as u32)?;
        }

        if emulator.sound_timer() > 0 {
             audio_device.resume();
        } else {
            audio_device.pause();
        }

        if update_pixel_data(&emulator.display, &mut pixel_data, color) {
            texture.update(None, &pixel_data, WIDTH as usize * 3).unwrap();
            canvas.copy(&texture, None, None).unwrap();
            canvas.present();
        }

        spin_sleep::sleep(Duration::new(0, 1_000_000_000 / FPS));
    }

    Ok(())
}

fn keycode_to_key(keycode: Keycode) -> Option<u8> {
    match keycode {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0xC),
        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xD),
        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE),
        Keycode::Z => Some(0xA),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),
        _ => None,
    }
}

fn update_pixel_data(display: &Display, pixel_data: &mut [u8], color: Color) -> bool {
    let mut update = false;

    for (y, &row) in display.pixel_rows().iter().enumerate() {
        for x in 0..64 {
            let mask = 1 << (63 - x);
            let pixel_color = if row & mask == 0 { Color::BLACK } else { color };
            let (r, g, b) = pixel_color.rgb();

            let index = (y * WIDTH as usize + x) * 3;

            if pixel_data[index] != r || pixel_data[index + 1] != g || pixel_data[index + 2] != b {
                update = true;
                pixel_data[index + 0] = r;
                pixel_data[index + 1] = g;
                pixel_data[index + 2] = b;
            }
        }
    }

    update
}
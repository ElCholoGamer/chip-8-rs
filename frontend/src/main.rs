use std::time::Duration;
use std::fs;
use rand::Rng;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Mod};

use core::Emulator;
use frontend::audio;

const WINDOW_TITLE: &str = "CHIP-8 Emulator";
const WINDOW_SIZE: u32 = 15;
const WIDTH: u32 = 64;
const HEIGHT: u32 = 32;
const FPS: u32 = 60;
const CPF: u32 = 700 / FPS;

const LIGHT_GREEN: Color = Color::RGB(80, 255, 80);
const LIGHT_YELLOW: Color = Color::RGB(255, 255, 80);
const LIGHT_RED: Color = Color::RGB(255, 80, 80);

fn main() {
    let startup_program = include_bytes!("../startup.ch8").to_vec();
    let args: Vec<String> = std::env::args().collect();

    let initial_program = args.get(1).map_or(
        startup_program,
        |file| fs::read(file).expect("could not read file"));

    if let Err(e) = run(initial_program) {
        println!("{} {}", "Error:", e);
    }
}

fn run(mut program: Vec<u8>) -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window(WINDOW_TITLE, WIDTH * WINDOW_SIZE, HEIGHT * WINDOW_SIZE)
        .position_centered()
        .build().map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().present_vsync().build().map_err(|e| e.to_string())?;
    let mut event_pump = sdl_context.event_pump()?;
    let audio_subsystem = sdl_context.audio()?;

    let audio_device = audio::create_audio_device(&audio_subsystem)?;

    canvas.set_scale(WINDOW_SIZE as f32, WINDOW_SIZE as f32)?;
    canvas.present();

    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_target(PixelFormatEnum::RGB24, WIDTH, HEIGHT).map_err(|e| e.to_string())?;

    let mut paused = false;
    let mut speed = 1.0;
    let mut color = Color::WHITE;
    let mut pixel_data = [0; (WIDTH * HEIGHT * 3) as usize];

    let mut rng = rand::thread_rng();
    let mut emulator = Emulator::new(|| rng.gen_range(0..=u8::MAX));

    emulator.load_program(&program);

    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'main,
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    paused = !paused;

                    if paused {
                        canvas.window_mut().set_title(&format!("Paused · {}", WINDOW_TITLE)).unwrap();
                    } else {
                        canvas.window_mut().set_title(WINDOW_TITLE).unwrap();
                    }
                }
                Event::KeyDown { keymod, keycode: Some(keycode), .. } => {
                    if keymod.contains(Mod::LCTRLMOD) || keymod.contains(Mod::RCTRLMOD) {
                        match keycode {
                            Keycode::W if speed > 0.2 => speed -= 0.1,
                            Keycode::E if speed < 4.0 => speed += 0.1,
                            Keycode::C => color = match color {
                                Color::WHITE => LIGHT_GREEN,
                                LIGHT_GREEN => LIGHT_YELLOW,
                                LIGHT_YELLOW => LIGHT_RED,
                                _ => Color::WHITE,
                            },
                            Keycode::R => {
                                emulator.reset();
                                emulator.load_program(&program);
                            }
                            Keycode::O => {
                                if let Ok(Some(filename)) = frontend::prompt_file() {
                                    program = fs::read(filename).map_err(|e| e.to_string())?;
                                    emulator.reset();
                                    emulator.load_program(&program);
                                    paused = false;
                                }
                            }
                            Keycode::Q => break 'main,
                            _ => {}
                        }
                    } else if let Some(key) = frontend::keycode_to_key(keycode) {
                        emulator.keydown(key);
                    }
                }
                Event::KeyUp { keycode: Some(keycode), .. } => {
                    if let Some(key) = frontend::keycode_to_key(keycode) {
                        emulator.keyup(key);
                    }
                }
                _ => {}
            }
        }

        if !paused {
            emulator.time_step();
            emulator.cycle((CPF as f32 * speed) as u32).map_err(|e| e.to_string())?;
        }

        if emulator.sound_timer() > 0 && !paused {
            audio_device.resume();
        } else {
            audio_device.pause();
        }

        if frontend::update_pixel_data(&emulator.display, &mut pixel_data, color) {
            texture.update(None, &pixel_data, WIDTH as usize * 3).unwrap();
            canvas.copy(&texture, None, None).unwrap();
            canvas.present();
        }

        spin_sleep::sleep(Duration::new(0, 1_000_000_000 / FPS));
    }

    Ok(())
}

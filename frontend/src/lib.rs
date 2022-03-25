use nfd::{Response, Result as NFDResult};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;

use core::Display;

pub mod audio;

pub fn prompt_file() -> NFDResult<Option<String>> {
    let result = nfd::open_file_dialog(None, None)?;

    Ok(match result {
        Response::Okay(filename) => Some(filename),
        Response::OkayMultiple(files) => Some(files[0].clone()),
        Response::Cancel => None,
    })
}

pub fn update_pixel_data(display: &Display, pixel_data: &mut [u8], color: Color) -> bool {
    let mut update = false;

    for (y, &row) in display.pixel_rows().iter().enumerate() {
        for x in 0..64 {
            let mask = 1 << (63 - x);
            let pixel_color = if row & mask == 0 { Color::BLACK } else { color };
            let (r, g, b) = pixel_color.rgb();

            let i = (y * 64 + x) * 3;

            if pixel_data[i] != r || pixel_data[i + 1] != g || pixel_data[i + 2] != b {
                update = true;
                pixel_data[i + 0] = r;
                pixel_data[i + 1] = g;
                pixel_data[i + 2] = b;
            }
        }
    }

    update
}

pub fn keycode_to_key(keycode: Keycode) -> Option<u8> {
    Some(match keycode {
        Keycode::Num1 => 0x1,
        Keycode::Num2 => 0x2,
        Keycode::Num3 => 0x3,
        Keycode::Num4 => 0xC,
        Keycode::Q => 0x4,
        Keycode::W => 0x5,
        Keycode::E => 0x6,
        Keycode::R => 0xD,
        Keycode::A => 0x7,
        Keycode::S => 0x8,
        Keycode::D => 0x9,
        Keycode::F => 0xE,
        Keycode::Z => 0xA,
        Keycode::X => 0x0,
        Keycode::C => 0xB,
        Keycode::V => 0xF,
        _ => return None,
    })
}

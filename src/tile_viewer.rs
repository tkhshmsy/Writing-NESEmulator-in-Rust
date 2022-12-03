pub mod bus;
pub mod rom;
pub mod cpu;
pub mod opcodes;
pub mod trace;
pub mod ppu;
pub mod renderer;

// use bus::Bus;
use rom::Rom;
// use cpu::CPU;
// use bus::Memory;
// use trace::trace;
use renderer::Frame;
use renderer::SYSTEM_PALETTE;

use std::env;
use getopts::Options;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
// use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
// use sdl2::EventPump;

#[macro_use]
extern crate lazy_static;
// #[macro_use]
// extern crate bitflags;

fn show_tile(chr_rom: &Vec<u8>, bank: usize, tile_n: usize) ->Frame {
    assert!(bank <= 1);

    let mut frame = Frame::new();
    let bank = (bank * 0x1000) as usize;

    let tile = &chr_rom[(bank + tile_n * 16)..=(bank + tile_n * 16 + 15)];

    for y in 0..=7 {
        let mut upper = tile[y];
        let mut lower = tile[y + 8];

        for x in (0..=7).rev() {
            let value = (1 & upper) << 1 | (1 & lower);
            upper = upper >> 1;
            lower = lower >> 1;
            let rgb = match value {
                0 => SYSTEM_PALETTE[0x01],
                1 => SYSTEM_PALETTE[0x23],
                2 => SYSTEM_PALETTE[0x27],
                3 => SYSTEM_PALETTE[0x30],
                _ => panic!("out of palette"),
            };
            frame.set_pixel(x, y, rgb)
        }
    }

    frame
}


fn show_tile_bank(chr_rom: &Vec<u8>, bank: usize) ->Frame {
    assert!(bank <= 1);

    let mut frame = Frame::new();
    let mut tile_y = 0;
    let mut tile_x = 0;
    let bank = (bank * 0x1000) as usize;

    for tile_n in 0..255 {
        if tile_n != 0 && tile_n % 20 == 0 {
            tile_y += 10;
            tile_x = 0;
        }
        let tile = &chr_rom[(bank + tile_n * 16)..=(bank + tile_n * 16 + 15)];

        for y in 0..=7 {
            let mut upper = tile[y];
            let mut lower = tile[y + 8];

            for x in (0..=7).rev() {
                let value = (1 & upper) << 1 | (1 & lower);
                upper = upper >> 1;
                lower = lower >> 1;
                let rgb = match value {
                    0 => SYSTEM_PALETTE[0x01],
                    1 => SYSTEM_PALETTE[0x23],
                    2 => SYSTEM_PALETTE[0x27],
                    3 => SYSTEM_PALETTE[0x30],
                    _ => panic!("out of palette"),
                };
                frame.set_pixel(tile_x + x, tile_y + y, rgb)
            }
        }

        tile_x += 10;
    }
    frame
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} <iNes 1.0 ROM>", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m },
        Err(f) => { panic!("{}",f.to_string()); },
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }
    if matches.free.is_empty() {
        print_usage(&program, opts);
        return;
    }
    let rom_filename = matches.free[0].clone().to_string();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Tile Viewer", (256.0 * 3.0) as u32, (240.0 * 3.0) as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    canvas.set_scale(3.0, 3.0).unwrap();

    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_target(PixelFormatEnum::RGB24, 256, 240)
        .unwrap();

    let bytes: Vec<u8> = std::fs::read(&rom_filename).unwrap();
    let rom = Rom::new(&bytes).unwrap();

    let right_bank = show_tile_bank(&rom.chr_rom, 1);

    loop {
        texture.update(None, &right_bank.data, 256 * 3).unwrap();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => std::process::exit(0),
                _ => {
                    // none
                }
            }
        }
    }
}
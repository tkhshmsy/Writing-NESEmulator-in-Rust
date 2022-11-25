pub mod bus;
pub mod cpu;
pub mod opcodes;
pub mod rom;
pub mod trace;
// use bus::Memory;

#[macro_use]
extern crate lazy_static;

use std::env;
use std::fs::{File, metadata};
use std::io::Read;
// use rand::Rng;
// use sdl2::event::Event;
// use sdl2::EventPump;
// use sdl2::keyboard::Keycode;
// use sdl2::pixels::Color;
// use sdl2::pixels::PixelFormatEnum;

// fn handle_user_input(cpu: &mut cpu::CPU, event_pump: &mut EventPump) {
//     for event in event_pump.poll_iter() {
//         match event {
//             Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
//                 std::process::exit(0);
//             },
//             Event::KeyDown { keycode: Some(Keycode::W), .. } => {
//                 cpu.memory_write_u8(0xff, 0x77);
//             },
//             Event::KeyDown { keycode: Some(Keycode::S), .. } => {
//                 cpu.memory_write_u8(0xff, 0x73);
//             },
//             Event::KeyDown { keycode: Some(Keycode::A), .. } => {
//                 cpu.memory_write_u8(0xff, 0x61);
//             },
//             Event::KeyDown { keycode: Some(Keycode::D), .. } => {
//                 cpu.memory_write_u8(0xff, 0x64);
//             },
//             _ => {}
//         }
//     }
// }

// fn color(byte: u8) -> Color {
//     match byte {
//         0 => sdl2::pixels::Color::BLACK,
//         1 => sdl2::pixels::Color::WHITE,
//         2 | 9 => sdl2::pixels::Color::GRAY,
//         3 | 10 => sdl2::pixels::Color::RED,
//         4 | 11 => sdl2::pixels::Color::GREEN,
//         5 | 12 => sdl2::pixels::Color::BLUE,
//         6 | 13 => sdl2::pixels::Color::MAGENTA,
//         7 | 14 => sdl2::pixels::Color::YELLOW,
//         _ => sdl2::pixels::Color::CYAN,
//     }
// }

// fn read_screen_state(cpu: &cpu::CPU, frame: &mut [u8; 32 * 3 * 32]) -> bool {
//     let mut frame_idx = 0;
//     let mut update = false;
//     for i in 0x0200..0x0600 {
//         let color_idx = cpu.memory_read_u8(i as u16);
//         let (b1, b2, b3) = color(color_idx).rgb();
//         if frame[frame_idx] != b1 || frame[frame_idx + 1] != b2 || frame[frame_idx + 2] != b3 {
//             frame[frame_idx] = b1;
//             frame[frame_idx + 1] = b2;
//             frame[frame_idx + 2] = b3;
//             update = true;
//         }
//         frame_idx += 3;
//     }
//     return update;
// }

fn read_rom_file(filename: &String) -> Result<rom::Rom, String> {
    let mut fp = File::open(filename).expect("unable to open.");
    let fp_metadata = metadata(&filename).expect("unable to get metadata.");
    let mut buffer = vec![0; fp_metadata.len() as usize];
    fp.read(&mut buffer).expect("unable to read.");
    return rom::Rom::new(&buffer);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <iNES 1.0 ROM> [-s <start address>]",args[0]);
        std::process::exit(1);
    }
    let rom_filename = &args[1];

    // init SDL2
    // let sdl_context = sdl2::init().unwrap();
    // let video_subsystem = sdl_context.video().unwrap();
    // let window = video_subsystem
    //                 .window("Snake game", (32.0 * 10.0) as u32, (32.0 * 10.0) as u32)
    //                 .position_centered()
    //                 .build().unwrap();
    // let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    // let mut event_pump = sdl_context.event_pump().unwrap();
    // canvas.set_scale(10.0, 10.0).unwrap();

    // let creator = canvas.texture_creator();
    // let mut texture = creator
    //                     .create_texture_target(PixelFormatEnum::RGB24, 32 ,32).unwrap();

    let rom = read_rom_file(rom_filename).expect("failed to read ROM.");
    let bus = bus::Bus::new_with_rom(rom);
    let mut cpu = cpu::CPU::new(bus);
    cpu.reset();
    cpu.reg_pc = 0xc000;

    // let mut screen_state = [ 0 as u8; 32 * 3 *32];
    // let mut rng = rand::thread_rng();

    cpu.run_with_callback(move |cpu| {
        println!("{}", trace::trace(cpu));
        // handle_user_input(cpu, &mut event_pump);
        // cpu.memory_write_u8(0xfe, rng.gen_range(1..16));

        // if read_screen_state(cpu, &mut screen_state) {
        //     texture.update(None, &screen_state, 32 *3).unwrap();
        //     canvas.copy(&texture, None, None).unwrap();
        //     canvas.present();
        // }

        // ::std::thread::sleep(std::time::Duration::new(0, 70_000));
    });
}

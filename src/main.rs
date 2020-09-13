mod cpu;
mod ppu;
mod apu;
mod memory;
mod utils;

struct DMG01 {
    cpu: cpu::CPU,
}

use memory::cartridge::Cartridge;

impl DMG01 {
    fn new(cart: Option<Cartridge>) -> DMG01 {
        DMG01 {
            cpu: cpu::CPU::new(cart),
        }
    }
}

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str), long)]
    rom: Option<std::path::PathBuf>,
}

fn main() {
    let args = Cli::from_args();

    use std::fs;
    let cart = if let Some(rom_path) = args.rom {
        Some(Cartridge {
            rom: fs::read(rom_path).expect("Could not open rom file!"),
        })
    } else {
        None
    };

    use ppu::{LCD_WIDTH, LCD_HEIGHT};
    use minifb::{Window, WindowOptions};
    let mut window = match Window::new("DMG-01", LCD_WIDTH as usize * 3, LCD_HEIGHT as usize * 3, WindowOptions::default()) {
        Ok(win) => win,
        Err(_) => panic!("Could not create window!"),
    };
    window.limit_update_rate(Some(std::time::Duration::from_millis(16)));

    let gameboy = DMG01::new(cart);
    let _audio_player = apu::cpal_audio_output::CpalAudioLoop::new(gameboy.cpu).ok();

    while window.is_open() {

        //window
        //    .update_with_buffer(gameboy.cpu.bus.ppu.framebuffer.as_slice(), LCD_WIDTH as usize, LCD_HEIGHT as usize)
        //    .unwrap();
        window.update();
    }
}

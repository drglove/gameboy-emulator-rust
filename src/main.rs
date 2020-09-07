mod cpu;
mod ppu;
mod apu;
mod memory;
mod utils;

struct DMG01 {
    cpu: cpu::CPU,
}

impl DMG01 {
    fn new(cart: Option<Cartridge>) -> DMG01 {
        use std::cmp::min;

        let mut memory: [u8; 0x10000] = [0; 0x10000];
        let size_to_copy = min(0x10000, cart.as_ref().unwrap().rom.len());
        memory[0..size_to_copy].copy_from_slice(&cart.unwrap().rom.as_slice());

        DMG01 {
            cpu: cpu::CPU::new(memory),
        }
    }
}

struct Cartridge {
    rom: Vec<u8>,
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
    let mut window = match Window::new("DMG-01", LCD_WIDTH as usize, LCD_HEIGHT as usize, WindowOptions::default()) {
        Ok(win) => win,
        Err(_) => panic!("Could not create window!"),
    };
    window.limit_update_rate(Some(std::time::Duration::from_millis(16)));

    let mut gameboy = DMG01::new(cart);
    while window.is_open() {
        gameboy.cpu.step_frame();

        window
            .update_with_buffer(gameboy.cpu.bus.ppu.framebuffer.as_slice(), LCD_WIDTH as usize, LCD_HEIGHT as usize)
            .unwrap();
    }
}

mod cpu;
mod ppu;

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

struct MemoryBus {
    memory: [u8; 0x10000],
    boot_rom: [u8; BOOTROM_SIZE],
    finished_boot: bool,
    ppu: PPU,
}

impl MemoryBus {
    fn read_byte(&self, address: u16) -> u8 {
        let address = address as usize;
        match address {
            BOOTROM_BEGIN..=BOOTROM_END if !self.finished_boot => self.boot_rom[address],
            VRAM_BEGIN..=VRAM_END => self.ppu.read_vram(address - VRAM_BEGIN),
            IO_REGISTER_BEGIN..=IO_REGISTER_END => self.read_io_register(address),
            _ => self.memory[address],
        }
    }

    fn read_byte_from_offset(&self, address_offset: u8) -> u8 {
        self.read_byte(address_offset as u16 + 0xFF00)
    }

    fn write_byte(&mut self, value: u8, address: u16) {
        let address = address as usize;
        match address {
            BOOTROM_BEGIN..=BOOTROM_END if !self.finished_boot => panic!("Cannot write into bootrom territory!"),
            VRAM_BEGIN..=VRAM_END => self.ppu.write_vram(value, address - VRAM_BEGIN),
            IO_REGISTER_BEGIN..=IO_REGISTER_END => self.write_io_register(value, address),
            _ => self.memory[address] = value,
        }
    }

    fn write_byte_to_offset(&mut self, value: u8, address_offset: u8) {
        self.write_byte(value, address_offset as u16 + 0xFF00)
    }

    fn read_io_register(&self, address: usize) -> u8 {
        match address {
            _ if self.ppu.supports_io_register(address) => self.ppu.read_io_register(address),
            _ => self.memory[address],
        }
    }

    fn write_io_register(&mut self, value: u8, address: usize) {
        match address {
            0xFF50 if !self.finished_boot => self.finished_boot = true,
            _ if self.ppu.supports_io_register(address) => self.ppu.write_io_register(value, address),
            _ => self.memory[address] = value,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum Interrupt {
    VBlank,
    LCDStat,
    Timer,
    Serial,
    Joypad,
}

impl Interrupt {
    const INTERRUPT_ENABLE_ADDRESS: u16 = 0xFFFF;
    const INTERRUPT_FLAG_ADDRESS: u16 = 0xFF0F;

    fn get_interrupts_to_process(bus: &MemoryBus) -> Vec<Self> {
        let mut interrupts: Vec<Interrupt> = vec![];
        let all_interrupts = vec![
            Interrupt::VBlank,
            Interrupt::LCDStat,
            Interrupt::Timer,
            Interrupt::Serial,
            Interrupt::Joypad,
        ];
        for interrupt in all_interrupts {
            if interrupt.is_interrupt_enabled(bus) && interrupt.is_interrupt_flag_set(bus) {
                interrupts.push(interrupt);
            }
        }
        interrupts
    }

    fn is_interrupt_enabled(&self, bus: &MemoryBus) -> bool {
        let interrupt_enabled_byte = bus.read_byte(Interrupt::INTERRUPT_ENABLE_ADDRESS);
        let mask: u8 = self.interrupt_byte_mask();
        (interrupt_enabled_byte & mask) == mask
    }

    fn is_interrupt_flag_set(&self, bus: &MemoryBus) -> bool {
        let interrupt_flag_byte = bus.read_byte(Interrupt::INTERRUPT_FLAG_ADDRESS);
        let mask = self.interrupt_byte_mask();
        (interrupt_flag_byte & mask) == mask
    }

    fn set_interrupt_flag(&self, bus: &mut MemoryBus) {
        self.set_interrupt_flag_to_value(true, bus);
    }

    fn clear_interrupt_flag(&self, bus: &mut MemoryBus) {
        self.set_interrupt_flag_to_value(false, bus);
    }

    fn set_interrupt_flag_to_value(&self, value: bool, bus: &mut MemoryBus) {
        let interrupt_flag_byte = bus.read_byte(Interrupt::INTERRUPT_FLAG_ADDRESS);
        let mask = self.interrupt_byte_mask();
        let new_flag_byte = if value {
            interrupt_flag_byte.bitor(mask)
        } else {
            interrupt_flag_byte.bitand(mask.not())
        };
        bus.write_byte(new_flag_byte, Interrupt::INTERRUPT_FLAG_ADDRESS);
    }

    fn interrupt_byte_mask(&self) -> u8 {
        (match self {
            Interrupt::VBlank => 1 << 0,
            Interrupt::LCDStat => 1 << 1,
            Interrupt::Timer => 1 << 2,
            Interrupt::Serial => 1 << 3,
            Interrupt::Joypad => 1 << 4,
        }) as u8
    }
}

const BOOTROM_BEGIN: usize = 0x0000;
const BOOTROM_END: usize = 0x00FF;
const BOOTROM_SIZE: usize = BOOTROM_END - BOOTROM_BEGIN + 1;
const VRAM_BEGIN: usize = 0x8000;
const VRAM_END: usize = 0x9FFF;
const VRAM_SIZE: usize = VRAM_END - VRAM_BEGIN + 1;
const IO_REGISTER_BEGIN: usize = 0xFF00;
const IO_REGISTER_END: usize = 0xFF7F;
const BG_MAP_START: usize = 0x9800;
const BG_MAP_END: usize = 0x9BFF;
const LCD_WIDTH: u8 = 160;
const LCD_HEIGHT: u8 = 144;

fn dump_bytes(bytes: &[u8], filename: &str) {
    std::fs::write(filename, bytes).unwrap();
}

struct Cartridge {
    rom: Vec<u8>,
}

use std::ops::{BitAnd, BitOr, Not};
use structopt::StructOpt;
use ppu::PPU;
use std::hash::Hash;

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

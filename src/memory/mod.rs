pub mod cartridge;

use crate::apu::APU;
use crate::cpu::timers::Timers;
use crate::input::InputState;
use crate::ppu::PPU;
use cartridge::Cartridge;

pub struct MemoryBus {
    memory: Vec<u8>,
    boot_rom: [u8; BOOTROM_SIZE],
    cart_rom: Option<Cartridge>,
    finished_boot: bool,
    pub ppu: PPU,
    pub apu: APU,
    pub input: InputState,
    pub timer: Timers,
}

impl MemoryBus {
    pub fn new(cart: Option<Cartridge>) -> Self {
        MemoryBus {
            memory: vec![0; 0x10000],
            boot_rom: [
                0x31, 0xFE, 0xFF, 0xAF, 0x21, 0xFF, 0x9F, 0x32, 0xCB, 0x7C, 0x20, 0xFB, 0x21, 0x26,
                0xFF, 0x0E, 0x11, 0x3E, 0x80, 0x32, 0xE2, 0x0C, 0x3E, 0xF3, 0xE2, 0x32, 0x3E, 0x77,
                0x77, 0x3E, 0xFC, 0xE0, 0x47, 0x11, 0x04, 0x01, 0x21, 0x10, 0x80, 0x1A, 0xCD, 0x95,
                0x00, 0xCD, 0x96, 0x00, 0x13, 0x7B, 0xFE, 0x34, 0x20, 0xF3, 0x11, 0xD8, 0x00, 0x06,
                0x08, 0x1A, 0x13, 0x22, 0x23, 0x05, 0x20, 0xF9, 0x3E, 0x19, 0xEA, 0x10, 0x99, 0x21,
                0x2F, 0x99, 0x0E, 0x0C, 0x3D, 0x28, 0x08, 0x32, 0x0D, 0x20, 0xF9, 0x2E, 0x0F, 0x18,
                0xF3, 0x67, 0x3E, 0x64, 0x57, 0xE0, 0x42, 0x3E, 0x91, 0xE0, 0x40, 0x04, 0x1E, 0x02,
                0x0E, 0x0C, 0xF0, 0x44, 0xFE, 0x90, 0x20, 0xFA, 0x0D, 0x20, 0xF7, 0x1D, 0x20, 0xF2,
                0x0E, 0x13, 0x24, 0x7C, 0x1E, 0x83, 0xFE, 0x62, 0x28, 0x06, 0x1E, 0xC1, 0xFE, 0x64,
                0x20, 0x06, 0x7B, 0xE2, 0x0C, 0x3E, 0x87, 0xE2, 0xF0, 0x42, 0x90, 0xE0, 0x42, 0x15,
                0x20, 0xD2, 0x05, 0x20, 0x4F, 0x16, 0x20, 0x18, 0xCB, 0x4F, 0x06, 0x04, 0xC5, 0xCB,
                0x11, 0x17, 0xC1, 0xCB, 0x11, 0x17, 0x05, 0x20, 0xF5, 0x22, 0x23, 0x22, 0x23, 0xC9,
                0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C,
                0x00, 0x0D, 0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6,
                0xDD, 0xDD, 0xD9, 0x99, 0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC,
                0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E, 0x3C, 0x42, 0xB9, 0xA5, 0xB9, 0xA5, 0x42, 0x3C,
                0x21, 0x04, 0x01, 0x11, 0xA8, 0x00, 0x1A, 0x13, 0xBE, 0x20, 0xFE, 0x23, 0x7D, 0xFE,
                0x34, 0x20, 0xF5, 0x06, 0x19, 0x78, 0x86, 0x23, 0x05, 0x20, 0xFB, 0x86, 0x20, 0xFE,
                0x3E, 0x01, 0xE0, 0x50,
            ],
            cart_rom: cart,
            finished_boot: false,
            ppu: PPU::new(),
            apu: APU::new(),
            input: Default::default(),
            timer: Default::default(),
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        let address = address as usize;
        match address {
            BOOTROM_BEGIN..=BOOTROM_END if !self.finished_boot => self.boot_rom[address],
            CARTRIDGE_ROM_BANK_0_START..=CARTRIDGE_ROM_BANK_0_END
            | CARTRIDGE_ROM_BANK_REST_START..=CARTRIDGE_ROM_BANK_REST_END => {
                if self.cart_rom.as_ref().is_some()
                    && address < self.cart_rom.as_ref().unwrap().rom.len()
                {
                    self.cart_rom.as_ref().unwrap().rom[address]
                } else {
                    0
                }
            }
            VRAM_BEGIN..=VRAM_END => self.ppu.read_vram(address - VRAM_BEGIN),
            IO_REGISTER_BEGIN..=IO_REGISTER_END => self.read_io_register(address),
            _ => self.memory[address],
        }
    }

    pub fn read_byte_from_offset(&self, address_offset: u8) -> u8 {
        self.read_byte(address_offset as u16 + 0xFF00)
    }

    pub fn write_byte(&mut self, value: u8, address: u16) {
        let address = address as usize;
        match address {
            BOOTROM_BEGIN..=BOOTROM_END if !self.finished_boot => {}
            CARTRIDGE_ROM_BANK_0_START..=CARTRIDGE_ROM_BANK_0_START
            | CARTRIDGE_ROM_BANK_REST_START..=CARTRIDGE_ROM_BANK_REST_END => {}
            VRAM_BEGIN..=VRAM_END => self.ppu.write_vram(value, address - VRAM_BEGIN),
            IO_REGISTER_BEGIN..=IO_REGISTER_END => self.write_io_register(value, address),
            _ => self.memory[address] = value,
        }
    }

    pub fn write_byte_to_offset(&mut self, value: u8, address_offset: u8) {
        self.write_byte(value, address_offset as u16 + 0xFF00)
    }

    fn read_io_register(&self, address: usize) -> u8 {
        match address {
            _ if self.input.supports_io_register(address) => self.input.read_io_register(address),
            _ if self.ppu.supports_io_register(address) => self.ppu.read_io_register(address),
            _ if APU::supports_io_register(address) => self.apu.read_io_register(address),
            _ if self.timer.supports_io_register(address) => self.timer.read_io_register(address),
            _ => self.memory[address],
        }
    }

    fn write_io_register(&mut self, value: u8, address: usize) {
        match address {
            0xFF50 if !self.finished_boot => self.finished_boot = true,
            _ if self.input.supports_io_register(address) => {
                self.input.write_io_register(value, address)
            }
            _ if self.ppu.supports_io_register(address) => {
                self.ppu.write_io_register(value, address)
            }
            _ if APU::supports_io_register(address) => self.apu.write_io_register(value, address),
            _ if self.timer.supports_io_register(address) => {
                self.timer.write_io_register(value, address)
            }
            _ => self.memory[address] = value,
        }
    }
}

const BOOTROM_BEGIN: usize = 0x0000;
const BOOTROM_END: usize = 0x00FF;
const BOOTROM_SIZE: usize = BOOTROM_END - BOOTROM_BEGIN + 1;
const CARTRIDGE_ROM_BANK_0_START: usize = 0x0000;
const CARTRIDGE_ROM_BANK_0_END: usize = 0x3FFF;
const CARTRIDGE_ROM_BANK_REST_START: usize = 0x4000;
const CARTRIDGE_ROM_BANK_REST_END: usize = 0x7FFF;
pub const VRAM_BEGIN: usize = 0x8000;
pub const VRAM_END: usize = 0x9FFF;
pub const VRAM_SIZE: usize = VRAM_END - VRAM_BEGIN + 1;
const IO_REGISTER_BEGIN: usize = 0xFF00;
const IO_REGISTER_END: usize = 0xFF7F;

mod cpu;

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
            0xFF42 => self.ppu.scroll.1,
            0xFF43 => self.ppu.scroll.0,
            0xFF44 => self.ppu.line,
            0xFF47 => u8::from(&self.ppu.palette),
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
            0xFF42 => self.ppu.scroll.1 = value,
            0xFF43 => self.ppu.scroll.0 = value,
            0xFF47 => self.ppu.palette = Palette::from(value),
            0xFF50 if !self.finished_boot => self.finished_boot = true,
            _ => self.memory[address] = value,
        }
    }

    fn write_byte_to_offset(&mut self, value: u8, address_offset: u8) {
        self.write_byte(value, address_offset as u16 + 0xFF00)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
enum Interrupt {
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
const BG_MAP_START: usize = 0x9800;
const BG_MAP_END: usize = 0x9BFF;
const LCD_WIDTH: u8 = 160;
const LCD_HEIGHT: u8 = 144;

struct PPU {
    vram: [u8; VRAM_SIZE],
    tile_set: [Tile; 384],
    mode: PPUMode,
    cycles: u16,
    line: u8,
    palette: Palette,
    scroll: (u8, u8),
    framebuffer: Vec<u32>,
}

enum PPUMode {
    HBlank,
    VBlank,
    OAMAccess,
    VRAMAccess,
}

impl PPU {
    fn read_vram(&self, address: usize) -> u8 {
        self.vram[address]
    }

    fn write_vram(&mut self, value: u8, address: usize) {
        self.vram[address] = value;

        // Addresses outside this range are not in the tile set.
        if address >= 0x1800 {
            return;
        }

        // Determine the even address corresponding to this address.
        let even_address = address & 0xFFFE;
        let byte1 = self.vram[even_address];
        let byte2 = self.vram[even_address + 1];

        // Each row is 16 bytes, and every 2 bytes is a new row.
        let tile_index = address / 16;
        let row_index = (address % 16) / 2;

        for pixel_index in 0..8 {
            let mask = (1 << (7 - pixel_index)) as u8;
            let lsb = byte1 & mask;
            let msb = byte2 & mask;
            let pixel_value = match (msb != 0, lsb != 0) {
                (false, false) => TilePixelValue::Zero,
                (false, true) => TilePixelValue::One,
                (true, false) => TilePixelValue::Two,
                (true, true) => TilePixelValue::Three,
            };
            self.tile_set[tile_index].pixels[row_index][pixel_index] = pixel_value;
        }
    }

    fn step(&mut self, cycles: u8) -> HashSet<Interrupt> {
        self.cycles += cycles as u16;

        let mut interrupts = HashSet::new();
        match self.mode {
            PPUMode::HBlank => {
                if self.cycles >= 200 {
                    self.cycles = self.cycles % 200;
                    self.line += 1;

                    if self.line >= LCD_HEIGHT {
                        self.mode = PPUMode::VBlank;
                        interrupts.insert(Interrupt::VBlank);
                    } else {
                        self.mode = PPUMode::OAMAccess;
                    }
                }
            }
            PPUMode::VBlank => {
                if self.cycles >= 456 {
                    self.cycles = self.cycles % 456;
                    self.line += 1;

                    if self.line == 154 {
                        self.mode = PPUMode::OAMAccess;
                        self.line = 0;
                    }
                }
            }
            PPUMode::OAMAccess => {
                if self.cycles >= 80 {
                    self.cycles = self.cycles % 80;
                    self.mode = PPUMode::VRAMAccess;
                }
            }
            PPUMode::VRAMAccess => {
                if self.cycles >= 172 {
                    self.cycles = self.cycles % 172;
                    self.mode = PPUMode::HBlank;
                    self.render_line();
                }
            }
        }
        interrupts
    }

    fn render_line(&mut self) {
        const BG_OFFSET: usize = BG_MAP_START - VRAM_BEGIN;
        const PIXEL_DIMENSION_PER_TILE: usize = 8;
        const TILES_PER_ROW: usize = 0x20;

        let pixel_row = self.line.wrapping_add(self.scroll.1);

        for column in 0..=255 as u8 {
            let pixel_column = column.wrapping_add(self.scroll.0);
            let tile_row = (pixel_row as usize) / PIXEL_DIMENSION_PER_TILE;
            let tile_column = (pixel_column as usize) / PIXEL_DIMENSION_PER_TILE;
            let tile_address = BG_OFFSET + tile_row * TILES_PER_ROW + tile_column;
            let tile_byte = self.vram[tile_address];
            let onscreen = self.line < LCD_HEIGHT && column < LCD_WIDTH;
            if onscreen {
                let pixel_index = (self.line as usize) * (LCD_WIDTH as usize) + column as usize;
                self.framebuffer[pixel_index] = self.get_pixel_colour_from_tile(tile_byte, pixel_row % 8, (pixel_column % 8) as u8);
            }
        }
    }

    fn get_pixel_colour_from_tile(&self, tile: u8, row: u8, col: u8) -> u32 {
        let tile = self.tile_set[tile as usize];
        let tile_pixel= tile.pixels[row as usize][col as usize];
        self.palette.get_colour(&tile_pixel)
    }

    fn render_entire_framebuffer(&self) -> Vec<u32> {
        let mut framebuffer = vec![0; 256 * 256];
        let mut current_pixel = 0;
        for tile_row in 0..=31 {
            for pixel_row in 0..=7 {
                for tile_column in 0..=31 {
                    let bg_start = BG_MAP_START - VRAM_BEGIN;
                    let tiles_per_row = 0x20;
                    let tile_address = bg_start + tile_row * tiles_per_row + tile_column;
                    let tile_byte = self.vram[tile_address];
                    for pixel_column in 0..=7 {
                        framebuffer[current_pixel] = self.get_pixel_colour_from_tile(tile_byte, pixel_row, pixel_column);
                        current_pixel += 1;
                    }
                }
            }
        }
        framebuffer
    }
}

fn dump_bytes(bytes: &[u8], filename: &str) {
    std::fs::write(filename, bytes).unwrap();
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
enum TilePixelValue {
    Zero,
    One,
    Two,
    Three,
}

impl TilePixelValue {
    fn bit_position(&self) -> u8 {
        match self {
            TilePixelValue::Zero => 0,
            TilePixelValue::One => 2,
            TilePixelValue::Two => 4,
            TilePixelValue::Three => 6,
        }
    }
}

#[derive(Copy, Clone)]
enum Shade {
    White,
    LightGray,
    DarkGray,
    Black,
}

impl Shade {
    fn colour(&self) -> u32 {
        match self {
            Shade::White => 0xF8F8F8,
            Shade::LightGray => 0xA8A8A8,
            Shade::DarkGray => 0x505050,
            Shade::Black => 0x000000,
        }
    }
}

impl std::convert::From<&Shade> for u8 {
    fn from(shade: &Shade) -> Self {
        match *shade {
            Shade::White => 0b00,
            Shade::LightGray => 0b01,
            Shade::DarkGray => 0b10,
            Shade::Black => 0b11,
        }
    }
}

impl std::convert::From<u8> for Shade {
    fn from(value: u8) -> Self {
        match value & 0b11 {
            0b00 => Shade::White,
            0b01 => Shade::LightGray,
            0b10 => Shade::DarkGray,
            0b11 => Shade::Black,
            _ => unreachable!(),
        }
    }
}

struct Palette {
    shades: HashMap<TilePixelValue, Shade>,
}

impl Palette {
    fn get_colour(&self, tile_pixel: &TilePixelValue) -> u32 {
        self.shades.get(tile_pixel).expect("Unknown palette colour for tile pixel!").colour()
    }
}

impl Default for Palette {
    fn default() -> Self {
        Palette {
            shades: [(TilePixelValue::Zero, Shade::White), (TilePixelValue::One, Shade::LightGray), (TilePixelValue::Two, Shade::DarkGray), (TilePixelValue::Three, Shade::Black)].iter().cloned().collect(),
        }
    }
}

impl std::convert::From<&Palette> for u8 {
    fn from(palette: &Palette) -> u8 {
        let mut byte = 0;
        for (tile_pixel, shade) in &palette.shades {
            let shade_bits = u8::from(shade);
            byte = byte | (shade_bits << tile_pixel.bit_position());
        }
        byte
    }
}

impl std::convert::From<u8> for Palette {
    fn from(value: u8) -> Self {
        let mut shades: HashMap<TilePixelValue, Shade> = Default::default();
        for tile_pixel in [TilePixelValue::Zero, TilePixelValue::One, TilePixelValue::Two, TilePixelValue::Three].iter() {
            let bit_position = tile_pixel.bit_position();
            let mask = 0b11 << bit_position;
            let shade_bits = (value & mask) >> bit_position;
            let shade = Shade::from(shade_bits);
            shades.insert(*tile_pixel, shade);
        }
        Palette { shades }
    }
}

#[derive(Copy, Clone)]
struct Tile {
    pixels: [[TilePixelValue; 8]; 8],
}

impl Tile {
    fn empty_tile() -> Tile {
        Tile {
            pixels: [[TilePixelValue::Zero; 8]; 8],
        }
    }
}

struct Cartridge {
    rom: Vec<u8>,
}

use std::collections::{HashSet, HashMap};
use std::ops::{BitAnd, BitOr, Not};
use structopt::StructOpt;
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

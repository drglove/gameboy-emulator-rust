mod palette;
mod tile;

use crate::{BG_MAP_START, LCD_HEIGHT, LCD_WIDTH, VRAM_BEGIN, VRAM_SIZE};
use palette::*;
use std::collections::HashSet;
use tile::Tile;
use crate::cpu::interrupts::Interrupt;

pub struct PPU {
    vram: [u8; VRAM_SIZE],
    tile_set: [Tile; 384],
    mode: PPUMode,
    cycles: u16,
    line: u8,
    palette: Palette,
    scroll: (u8, u8),
    pub framebuffer: Vec<u32>,
}

enum PPUMode {
    HBlank,
    VBlank,
    OAMAccess,
    VRAMAccess,
}

impl PPU {
    pub fn new() -> Self {
        PPU {
            vram: [0; VRAM_SIZE],
            tile_set: [Tile::empty_tile(); 384],
            mode: PPUMode::HBlank,
            cycles: 0,
            line: 0,
            palette: Palette::default(),
            scroll: (0, 0),
            framebuffer: vec![0; LCD_WIDTH as usize * LCD_HEIGHT as usize],
        }
    }

    pub fn supports_io_register(&self, address: usize) -> bool {
        match address {
            0xFF42 | 0xFF43 | 0xFF44 | 0xFF47 => true,
            _ => false,
        }
    }

    pub fn read_io_register(&self, address: usize) -> u8 {
        match address {
            0xFF42 => self.scroll.1,
            0xFF43 => self.scroll.0,
            0xFF44 => self.line,
            0xFF47 => u8::from(&self.palette),
            _ => panic!("Trying to read unknown IO register related to PPU!"),
        }
    }

    pub fn write_io_register(&mut self, value: u8, address: usize) {
        match address {
            0xFF42 => self.scroll.1 = value,
            0xFF43 => self.scroll.0 = value,
            0xFF47 => self.palette = Palette::from(value),
            _ => panic!("Trying to write to unknown IO register in PPU!"),
        }
    }

    pub fn read_vram(&self, address: usize) -> u8 {
        self.vram[address]
    }

    pub fn write_vram(&mut self, value: u8, address: usize) {
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
                (false, false) => PixelValue::Zero,
                (false, true) => PixelValue::One,
                (true, false) => PixelValue::Two,
                (true, true) => PixelValue::Three,
            };
            self.tile_set[tile_index].pixels[row_index][pixel_index] = pixel_value;
        }
    }

    pub fn step(&mut self, cycles: u8) -> HashSet<Interrupt> {
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
                self.framebuffer[pixel_index] = self.get_pixel_colour_from_tile(
                    tile_byte,
                    pixel_row % 8,
                    (pixel_column % 8) as u8,
                );
            }
        }
    }

    fn get_pixel_colour_from_tile(&self, tile: u8, row: u8, col: u8) -> u32 {
        let tile = self.tile_set[tile as usize];
        let tile_pixel = tile.pixels[row as usize][col as usize];
        self.palette.get_colour(&tile_pixel)
    }

    #[allow(dead_code)]
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
                        framebuffer[current_pixel] =
                            self.get_pixel_colour_from_tile(tile_byte, pixel_row, pixel_column);
                        current_pixel += 1;
                    }
                }
            }
        }
        framebuffer
    }
}

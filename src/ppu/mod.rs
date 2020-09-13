mod palette;
mod tile;

use crate::cpu::interrupts::{Interrupt, InterruptsToSet};
use crate::memory::{VRAM_BEGIN, VRAM_SIZE};
use palette::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tile::Tile;

pub struct PPU {
    vram: [u8; VRAM_SIZE],
    tile_set: [Tile; 384],
    mode: PPUMode,
    cycles: u16,
    line: Line,
    scroll: Scroll,
    palette: Palette,
    lines_to_render: LinesToRender,
    framebuffer: Framebuffer,
    pub displayable_framebuffer: Arc<Mutex<Framebuffer>>,
}

enum PPUMode {
    HBlank,
    VBlank,
    OAMAccess,
    VRAMAccess,
}

const BG_MAP_START: usize = 0x9800;
pub const LCD_WIDTH: u8 = 160;
pub const LCD_HEIGHT: u8 = 144;

type Line = u8;
type Framebuffer = Vec<u32>;

#[derive(Copy, Clone)]
struct Scroll {
    horiz: u8,
    vert: u8,
}

struct LinesToRender {
    pub jobs: HashMap<Line, Scroll>,
}

impl PPU {
    pub fn new() -> Self {
        PPU {
            vram: [0; VRAM_SIZE],
            tile_set: [Tile::empty_tile(); 384],
            mode: PPUMode::HBlank,
            cycles: 0,
            line: 0,
            scroll: Scroll { horiz: 0, vert: 0 },
            palette: Palette::default(),
            lines_to_render: LinesToRender {
                jobs: Default::default(),
            },
            framebuffer: vec![0; LCD_WIDTH as usize * LCD_HEIGHT as usize],
            displayable_framebuffer: Arc::new(Mutex::new(vec![
                0;
                LCD_WIDTH as usize
                    * LCD_HEIGHT as usize
            ])),
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
            0xFF42 => self.scroll.vert,
            0xFF43 => self.scroll.horiz,
            0xFF44 => self.line,
            0xFF47 => u8::from(&self.palette),
            _ => panic!("Trying to read unknown IO register related to PPU!"),
        }
    }

    pub fn write_io_register(&mut self, value: u8, address: usize) {
        match address {
            0xFF42 => self.scroll.vert = value,
            0xFF43 => self.scroll.horiz = value,
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

    pub fn step(&mut self, cycles: u8) -> InterruptsToSet {
        self.cycles += cycles as u16;

        let mut interrupts: InterruptsToSet = Default::default();
        match self.mode {
            PPUMode::HBlank => {
                if self.cycles >= 200 {
                    self.cycles = self.cycles % 200;
                    self.line += 1;

                    if self.line >= LCD_HEIGHT {
                        self.mode = PPUMode::VBlank;
                        interrupts.set_interrupt(Interrupt::VBlank);
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
                    self.queue_current_line_to_render();
                }
            }
        }

        interrupts
    }

    fn queue_current_line_to_render(&mut self) {
        if self.line < LCD_HEIGHT {
            self.lines_to_render.jobs.insert(self.line, self.scroll);
        }
    }

    pub fn render(&mut self) {
        let mut current_framebuffer = self.framebuffer.clone();
        for (line, scroll) in self.lines_to_render.jobs.iter() {
            let rendered_line = self.render_line(*line, *scroll);
            let pixel_offset = (*line as usize) * (LCD_WIDTH as usize);
            let current_rendered_line = &mut current_framebuffer.as_mut_slice()
                [pixel_offset..(pixel_offset + LCD_WIDTH as usize)];
            current_rendered_line.clone_from_slice(&rendered_line);
        }
        self.framebuffer = current_framebuffer;
        self.lines_to_render.jobs.clear();

        // Only try to lock here. If we fail, we will drop the frame as opposed to messing up our audio stream.
        let displayable_framebuffer = self.displayable_framebuffer.try_lock();
        if displayable_framebuffer.is_ok() {
            let mut displayable_framebuffer = displayable_framebuffer.unwrap();
            *displayable_framebuffer = self.framebuffer.clone();
        }
    }

    fn render_line(&self, line: Line, scroll: Scroll) -> [u32; LCD_WIDTH as usize] {
        const BG_OFFSET: usize = BG_MAP_START - VRAM_BEGIN;
        const PIXEL_DIMENSION_PER_TILE: usize = 8;
        const TILES_PER_ROW: usize = 0x20;

        let pixel_row = line.wrapping_add(scroll.vert);
        let mut rendered_line = [0; LCD_WIDTH as usize];

        for column in 0..LCD_WIDTH as u8 {
            let pixel_column = column.wrapping_add(scroll.horiz);
            let tile_row = (pixel_row as usize) / PIXEL_DIMENSION_PER_TILE;
            let tile_column = (pixel_column as usize) / PIXEL_DIMENSION_PER_TILE;
            let tile_address = BG_OFFSET + tile_row * TILES_PER_ROW + tile_column;
            let tile_byte = self.vram[tile_address];
            rendered_line[column as usize] =
                self.get_pixel_colour_from_tile(tile_byte, pixel_row % 8, (pixel_column % 8) as u8);
            rendered_line[column as usize] = 0;
        }
        rendered_line
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

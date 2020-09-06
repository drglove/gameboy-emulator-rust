use std::collections::HashMap;

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

pub(super) struct Palette {
    shades: HashMap<PixelValue, Shade>,
}

impl Palette {
    pub fn get_colour(&self, tile_pixel: &PixelValue) -> u32 {
        self.shades.get(tile_pixel).expect("Unknown palette colour for tile pixel!").colour()
    }
}

impl Default for Palette {
    fn default() -> Self {
        Palette {
            shades: [(PixelValue::Zero, Shade::White), (PixelValue::One, Shade::LightGray), (PixelValue::Two, Shade::DarkGray), (PixelValue::Three, Shade::Black)].iter().cloned().collect(),
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
        let mut shades: HashMap<PixelValue, Shade> = Default::default();
        for tile_pixel in [PixelValue::Zero, PixelValue::One, PixelValue::Two, PixelValue::Three].iter() {
            let bit_position = tile_pixel.bit_position();
            let mask = 0b11 << bit_position;
            let shade_bits = (value & mask) >> bit_position;
            let shade = Shade::from(shade_bits);
            shades.insert(*tile_pixel, shade);
        }
        Palette { shades }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum PixelValue {
    Zero,
    One,
    Two,
    Three,
}

impl PixelValue {
    fn bit_position(&self) -> u8 {
        match self {
            PixelValue::Zero => 0,
            PixelValue::One => 2,
            PixelValue::Two => 4,
            PixelValue::Three => 6,
        }
    }
}
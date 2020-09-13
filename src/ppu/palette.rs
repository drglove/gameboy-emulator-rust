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
    shades: [Shade; 4],
}

impl Palette {
    pub fn get_colour(&self, tile_pixel: &PixelValue) -> u32 {
        match tile_pixel {
            PixelValue::Zero => self.shades[0].colour(),
            PixelValue::One => self.shades[1].colour(),
            PixelValue::Two => self.shades[2].colour(),
            PixelValue::Three => self.shades[3].colour(),
        }
    }

    fn set_shade(&mut self, tile_pixel: &PixelValue, shade: Shade) {
        match tile_pixel {
            PixelValue::Zero => self.shades[0] = shade,
            PixelValue::One => self.shades[1] = shade,
            PixelValue::Two => self.shades[2] = shade,
            PixelValue::Three => self.shades[3] = shade,
        }
    }
}

impl Default for Palette {
    fn default() -> Self {
        Palette {
            shades: [
                Shade::White,
                Shade::LightGray,
                Shade::DarkGray,
                Shade::Black,
            ],
        }
    }
}

impl std::convert::From<&Palette> for u8 {
    fn from(palette: &Palette) -> u8 {
        let mut byte = 0;
        for (tile_pixel_index, shade) in palette.shades.iter().enumerate() {
            let shade_bits = u8::from(shade);
            let tile_pixel = match tile_pixel_index {
                0 => PixelValue::Zero,
                1 => PixelValue::One,
                2 => PixelValue::Two,
                3 => PixelValue::Three,
                _ => unreachable!(),
            };
            byte = byte | (shade_bits << tile_pixel.bit_position());
        }
        byte
    }
}

impl std::convert::From<u8> for Palette {
    fn from(value: u8) -> Self {
        let mut palette = Palette {
            shades: [Shade::White; 4],
        };
        for tile_pixel in [
            PixelValue::Zero,
            PixelValue::One,
            PixelValue::Two,
            PixelValue::Three,
        ]
        .iter()
        {
            let bit_position = tile_pixel.bit_position();
            let mask = 0b11 << bit_position;
            let shade_bits = (value & mask) >> bit_position;
            let shade = Shade::from(shade_bits);
            palette.set_shade(tile_pixel, shade);
        }
        palette
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

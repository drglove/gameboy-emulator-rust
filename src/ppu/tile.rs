use super::palette::PixelValue;

#[derive(Copy, Clone)]
pub(super) struct Tile {
    pub pixels: [[PixelValue; 8]; 8],
}

impl Tile {
    pub fn empty_tile() -> Tile {
        Tile {
            pixels: [[PixelValue::Zero; 8]; 8],
        }
    }
}

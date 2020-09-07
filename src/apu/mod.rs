use self::channels::SquareChannel;

mod channels;

pub struct APU {
    square_with_sweep: SquareChannel,
    square_without_sweep: SquareChannel,
}

impl APU {
    pub fn new() -> Self {
        APU {
            square_with_sweep: SquareChannel::new(),
            square_without_sweep: SquareChannel::new(),
        }
    }

    pub fn supports_io_register(address: usize) -> bool {
        match address {
            0xFF10..=0xFF19 => true,
            _ => false,
        }
    }

    pub fn read_io_register(&self, address: usize) -> u8 {
    }

    pub fn write_io_register(&self, value: u8, address: usize) {
    }
}

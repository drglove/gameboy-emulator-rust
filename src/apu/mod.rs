use self::channels::{SquareChannel, Sweep, Duty};

mod channels;

pub struct APU {
    square_with_sweep: SquareChannel,
    square_without_sweep: SquareChannel,
}

impl APU {
    pub fn new() -> Self {
        APU {
            square_with_sweep: SquareChannel::new_with_sweep(),
            square_without_sweep: SquareChannel::new_without_sweep(),
        }
    }

    pub fn supports_io_register(address: usize) -> bool {
        match address {
            0xFF10..=0xFF19 => true,
            _ => false,
        }
    }

    pub fn read_io_register(&self, address: usize) -> u8 {
        match address {
            0xFF10 => u8::from(self.square_with_sweep.sweep.as_ref().unwrap()),
            0xFF11 => u8::from(&self.square_with_sweep.duty),
            _ => panic!("Unknown command when reading from APU IO register!"),
        }
    }

    pub fn write_io_register(&mut self, value: u8, address: usize) {
        match address {
            0xFF10 => self.square_with_sweep.sweep = Some(Sweep::from(value)),
            0xFF11 => self.square_with_sweep.duty = Duty::from(value),
            _ => panic!("Unknown command when writing to APU IO register!"),
        }
    }
}

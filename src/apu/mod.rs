use self::channels::{NoiseRegister, SquareChannel};

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
            0xFF10 => NoiseRegister::read_nr10(&self.square_with_sweep),
            0xFF11 => NoiseRegister::read_nr11(&self.square_with_sweep),
            0xFF12 => NoiseRegister::read_nr12(&self.square_with_sweep),
            _ => panic!("Unknown command when reading from APU IO register!"),
        }
    }

    pub fn write_io_register(&mut self, value: u8, address: usize) {
        match address {
            0xFF10 => NoiseRegister::write_nr10(value, &mut self.square_with_sweep),
            0xFF11 => NoiseRegister::write_nr11(value, &mut self.square_with_sweep),
            0xFF12 => NoiseRegister::write_nr12(value, &mut self.square_with_sweep),
            _ => panic!("Unknown command when writing to APU IO register!"),
        }
    }
}

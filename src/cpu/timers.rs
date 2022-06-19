use crate::utils::frame_sequencer::FrameSequencer;

pub struct Timers {
    divider: u8,
    divider_sequencer: FrameSequencer,
}

impl Default for Timers {
    fn default() -> Self {
        Self {
            divider: 0,
            divider_sequencer: FrameSequencer::new(256 - 1, 256),
        }
    }
}

impl Timers {
    pub fn step(&mut self, cycles: u8) {
        if self.divider_sequencer.step_multiple(cycles) {
            self.divider = self.divider.wrapping_add(1);
        }
    }

    pub fn supports_io_register(&self, address: usize) -> bool {
        matches!(address, 0xFF04)
    }

    pub fn read_io_register(&self, address: usize) -> u8 {
        match address {
            0xFF04 => self.divider,
            _ => panic!("Reading from unsupported timer register!"),
        }
    }

    pub fn write_io_register(&mut self, _value: u8, address: usize) {
        match address {
            0xFF04 => self.divider = 0,
            _ => panic!("Writing to unsupported timer register!"),
        }
    }
}

#[derive(Copy, Clone)]
pub struct FrameSequencer {
    pub timer: u32,
    pub period: u32,
}

impl FrameSequencer {
    pub fn new(initial_delay: u32, period: u32) -> Self {
        FrameSequencer {
            timer: initial_delay,
            period,
        }
    }

    pub fn step_multiple(&mut self, cycles: u8) -> bool {
        if self.period > 0 {
            let underflow = self.timer < cycles as u32;
            self.timer =
                ((self.timer as i32) - (cycles as i32)).rem_euclid(self.period as i32) as u32;
            underflow
        } else {
            false
        }
    }

    pub fn step(&mut self) -> bool {
        self.step_multiple(1)
    }
}

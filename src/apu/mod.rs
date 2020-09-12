use self::channels::{Channel, NoiseRegister, SquareChannel, StereoOutput};

mod channels;
pub mod cpal_audio_output;
pub mod rodio_audio_output;
pub mod sdl_audio_output;

pub struct APU {
    square_with_sweep: SquareChannel,
    square_without_sweep: SquareChannel,
    frame_sequencer: FrameSequencer,
    cycles: u32,
}

pub trait AudioPlayer {
    fn play(&mut self, stereo_output: StereoOutput);
    fn sample_rate(&self) -> u32;
}

const MASTER_FRAME_SEQUENCER_CLOCK_RATE_HZ: u32 = 512;
const MASTER_FRAME_SEQUENCER_CLOCKS: u32 =
    crate::cpu::CPU_CLOCK_RATE_HZ / MASTER_FRAME_SEQUENCER_CLOCK_RATE_HZ;

impl APU {
    pub fn new() -> Self {
        APU {
            square_with_sweep: SquareChannel::new_with_sweep(),
            square_without_sweep: SquareChannel::new_without_sweep(),
            frame_sequencer: FrameSequencer {
                timer: 0, // We want the first tick to fire
                initial: MASTER_FRAME_SEQUENCER_CLOCKS,
            },
            cycles: 0,
        }
    }

    pub fn initialize_buffers(&mut self, sample_rate: u32, clock_rate: u32) {
        self.square_with_sweep
            .initialize_buffer(sample_rate, clock_rate);
        self.square_without_sweep
            .initialize_buffer(sample_rate, clock_rate);
    }

    pub fn step(&mut self, cycles: u8) {
        self.cycles += cycles as u32;
        self.square_with_sweep.step(cycles);
        //self.square_without_sweep.step(cycles);
    }

    pub fn end_frame(&mut self) {
        self.square_with_sweep.end_frame(self.cycles);
        //self.square_without_sweep.end_frame(end_frame_cycles);

        self.cycles = 0;
    }

    fn gather_samples(&mut self) -> StereoOutput {
        self.square_with_sweep.gather_samples()
    }

    pub fn play(&mut self, audio_player: Option<&mut impl AudioPlayer>) {
        let stereo_output = self.gather_samples();
        if let Some(audio_player) = audio_player {
            audio_player.play(stereo_output);
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
            0xFF13 => NoiseRegister::read_nr13(&self.square_with_sweep),
            0xFF14 => NoiseRegister::read_nr14(&self.square_with_sweep),
            _ => panic!("Unknown command when reading from APU IO register!"),
        }
    }

    pub fn write_io_register(&mut self, value: u8, address: usize) {
        match address {
            0xFF10 => NoiseRegister::write_nr10(value, &mut self.square_with_sweep),
            0xFF11 => NoiseRegister::write_nr11(value, &mut self.square_with_sweep),
            0xFF12 => NoiseRegister::write_nr12(value, &mut self.square_with_sweep),
            0xFF13 => NoiseRegister::write_nr13(value, &mut self.square_with_sweep),
            0xFF14 => NoiseRegister::write_nr14(value, &mut self.square_with_sweep),
            _ => panic!("Unknown command when writing to APU IO register!"),
        }
    }
}

struct FrameSequencer {
    timer: u32,
    initial: u32,
}

impl FrameSequencer {
    fn step(&mut self) -> bool {
        let (decremented_timer, underflow) = self.timer.overflowing_sub(1);
        self.timer = if underflow {
            self.initial
        } else {
            decremented_timer
        };
        underflow
    }
}

use self::channels::{Channel, NoiseRegister, SquareChannel, StereoOutput};
use crate::cpu::{CPU, CPU_CLOCK_RATE_HZ};

mod channels;
pub mod cpal_audio_output;

pub struct APU {
    square_with_sweep: SquareChannel,
    square_without_sweep: SquareChannel,
    sequencers: FrameSequencers,
    cycles: u32,
}

pub trait AudioLoop {}

impl dyn AudioLoop {
    pub fn run_cycles_for_desired_samples(samples_needed: u32, cpu: &mut CPU) {
        let cycles_to_run = cpu
            .bus
            .apu
            .cycles_needed_to_generate_samples(samples_needed);
        let mut cycles_ran = 0;
        while cycles_ran < cycles_to_run {
            let instruction_cycles = cpu.step_single_instruction();
            cycles_ran += instruction_cycles as u32;
        }
        cpu.end_frame();
    }
}

impl APU {
    pub fn new() -> Self {
        APU {
            square_with_sweep: SquareChannel::new_with_sweep(),
            square_without_sweep: SquareChannel::new_without_sweep(),
            sequencers: FrameSequencers::new(),
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

        let sequencers_to_fire = self.sequencers.step(cycles);
        self.square_with_sweep.fire_sequences(&sequencers_to_fire);
        self.square_without_sweep
            .fire_sequences(&sequencers_to_fire);

        self.square_with_sweep.step(cycles);
        self.square_without_sweep.step(cycles);
    }

    pub fn end_frame(&mut self) {
        self.square_with_sweep.end_frame(self.cycles);
        self.square_without_sweep.end_frame(self.cycles);
        self.cycles = 0;
    }

    pub fn cycles_needed_to_generate_samples(&self, samples_needed: u32) -> u32 {
        let samples = self
            .square_with_sweep
            .cycles_needed_to_generate_samples(samples_needed);
        debug_assert_eq!(
            samples,
            self.square_without_sweep
                .cycles_needed_to_generate_samples(samples_needed)
        );
        samples
    }

    pub fn gather_samples(&mut self) -> StereoOutput {
        let channel1 = self.square_with_sweep.gather_samples();
        let channel2 = self.square_without_sweep.gather_samples();
        Self::mix_channels(channel1, channel2)
    }

    fn mix_channels(channel1: StereoOutput, channel2: StereoOutput) -> StereoOutput {
        let num_samples = channel1.length();
        debug_assert_eq!(channel2.length(), num_samples);
        let mut output = channel1;
        for idx in 0..output.length() {
            output.left[idx] += channel2.left[idx];
            output.right[idx] += channel2.right[idx];
        }
        output
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
            0xFF11 => NoiseRegister::read_nrx1(&self.square_with_sweep),
            0xFF12 => NoiseRegister::read_nrx2(&self.square_with_sweep),
            0xFF13 => NoiseRegister::read_nrx3(&self.square_with_sweep),
            0xFF14 => NoiseRegister::read_nrx4(&self.square_with_sweep),
            0xFF16 => NoiseRegister::read_nrx1(&self.square_without_sweep),
            0xFF17 => NoiseRegister::read_nrx2(&self.square_without_sweep),
            0xFF18 => NoiseRegister::read_nrx3(&self.square_without_sweep),
            0xFF19 => NoiseRegister::read_nrx4(&self.square_without_sweep),
            _ => panic!("Unknown command when reading from APU IO register!"),
        }
    }

    pub fn write_io_register(&mut self, value: u8, address: usize) {
        match address {
            0xFF10 => NoiseRegister::write_nr10(value, &mut self.square_with_sweep),
            0xFF11 => NoiseRegister::write_nrx1(value, &mut self.square_with_sweep),
            0xFF12 => NoiseRegister::write_nrx2(value, &mut self.square_with_sweep),
            0xFF13 => NoiseRegister::write_nrx3(value, &mut self.square_with_sweep),
            0xFF14 => NoiseRegister::write_nrx4(value, &mut self.square_with_sweep),
            0xFF16 => NoiseRegister::write_nrx1(value, &mut self.square_without_sweep),
            0xFF17 => NoiseRegister::write_nrx2(value, &mut self.square_without_sweep),
            0xFF18 => NoiseRegister::write_nrx3(value, &mut self.square_without_sweep),
            0xFF19 => NoiseRegister::write_nrx4(value, &mut self.square_without_sweep),
            _ => panic!("Unknown command when writing to APU IO register!"),
        }
    }
}

#[derive(Copy, Clone)]
struct FrameSequencer {
    timer: u32,
    period: u32,
}

impl FrameSequencer {
    pub fn new(initial_delay: u32, period: u32) -> Self {
        FrameSequencer {
            timer: initial_delay,
            period,
        }
    }

    fn step(&mut self) -> bool {
        let (decremented_timer, underflow) = self.timer.overflowing_sub(1);
        self.timer = if underflow {
            self.period
        } else {
            decremented_timer
        };
        underflow
    }
}

#[derive(Copy, Clone)]
struct FrameSequencers {
    frame_sequencer: FrameSequencer,
    length_sequencer: FrameSequencer,
    volume_sequencer: FrameSequencer,
    sweep_sequencer: FrameSequencer,
}

struct SequencesToFire {
    fire: [bool; 3],
}

impl SequencesToFire {
    pub fn should_length_sequence_fire(&self) -> bool {
        self.fire[0]
    }

    pub fn should_volume_sequence_fire(&self) -> bool {
        self.fire[1]
    }

    pub fn should_sweep_sequence_fire(&self) -> bool {
        self.fire[2]
    }

    pub fn fire_length_sequence(&mut self) {
        self.fire[0] = true;
    }

    pub fn fire_volume_sequence(&mut self) {
        self.fire[1] = true;
    }

    pub fn fire_sweep_sequence(&mut self) {
        self.fire[2] = true;
    }
}

impl Default for SequencesToFire {
    fn default() -> Self {
        Self { fire: [false; 3] }
    }
}

impl FrameSequencers {
    pub fn new() -> Self {
        // Step   Length Ctr  Vol Env     Sweep
        // ---------------------------------------
        // 0      Clock       -           -
        // 1      -           -           -
        // 2      Clock       -           Clock
        // 3      -           -           -
        // 4      Clock       -           -
        // 5      -           -           -
        // 6      Clock       -           Clock
        // 7      -           Clock       -
        // ---------------------------------------
        // Rate   256 Hz      64 Hz       128 Hz
        Self {
            frame_sequencer: FrameSequencer::new(0, CPU_CLOCK_RATE_HZ / 512),
            length_sequencer: FrameSequencer::new(0, 2),
            volume_sequencer: FrameSequencer::new(7, 8),
            sweep_sequencer: FrameSequencer::new(2, 4),
        }
    }

    pub fn step(&mut self, cycles: u8) -> SequencesToFire {
        let mut sequences_to_fire = SequencesToFire::default();
        for _ in 0..cycles {
            if self.frame_sequencer.step() {
                if self.length_sequencer.step() {
                    sequences_to_fire.fire_length_sequence();
                }
                if self.volume_sequencer.step() {
                    sequences_to_fire.fire_volume_sequence();
                }
                if self.volume_sequencer.step() {
                    sequences_to_fire.fire_sweep_sequence();
                }
            }
        }
        sequences_to_fire
    }
}

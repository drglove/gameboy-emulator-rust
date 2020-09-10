use blip_buf::BlipBuf;

struct Sweep {
    period: u8,
    decrease: bool,
    shift: u8,
}

impl std::convert::From<&Sweep> for u8 {
    fn from(sweep: &Sweep) -> Self {
        let sweep_time = (sweep.period & 0b111) << 4;
        let decrease = (sweep.decrease as u8) << 3;
        let shift = sweep.shift & 0b111;
        sweep_time | decrease | shift
    }
}

impl std::convert::From<u8> for Sweep {
    fn from(value: u8) -> Self {
        Sweep {
            period: (value & 0b1110000) >> 4,
            decrease: (value & 0b1000) != 0,
            shift: (value & 0b111),
        }
    }
}

enum DutyType {
    Eighth,
    Quarter,
    Half,
    ThreeQuarters,
}

struct Duty {
    duty_type: DutyType,
    length: u8,
    phase: u8,
}

impl Duty {
    fn sequence(&self) -> i32 {
        let duty_wave_form: u8 = match self.duty_type {
            DutyType::Eighth => 0b01111111,
            DutyType::Quarter => 0b00111111,
            DutyType::Half => 0b00001111,
            DutyType::ThreeQuarters => 0b00000011,
        };
        let mask = 1 << (7 - self.phase);
        ((duty_wave_form & mask) == mask) as i32
    }

    fn step(&mut self) {
        self.phase = (self.phase + 1) % 8;
    }
}

impl std::convert::From<&Duty> for u8 {
    fn from(duty: &Duty) -> Self {
        let duty_type: u8 = match duty.duty_type {
            DutyType::Eighth => 0b00,
            DutyType::Quarter => 0b01,
            DutyType::Half => 0b10,
            DutyType::ThreeQuarters => 0b11,
        } << 6;
        // Length of the duty cycle is unreadable
        duty_type
    }
}

impl std::convert::From<u8> for Duty {
    fn from(value: u8) -> Self {
        let duty_bits = (value & 0b11000000) >> 6;
        let duty_type = match duty_bits {
            0b00 => DutyType::Eighth,
            0b01 => DutyType::Quarter,
            0b10 => DutyType::Half,
            0b11 => DutyType::ThreeQuarters,
            _ => unreachable!(),
        };
        let length = value & 0b111111;
        Duty {
            duty_type,
            length,
            phase: 0,
        }
    }
}

struct VolumeEnvelope {
    initial_volume: u8,
    increase: bool,
    period: u8,
}

impl std::convert::From<&VolumeEnvelope> for u8 {
    fn from(volume_envelope: &VolumeEnvelope) -> Self {
        let initial_volume = (volume_envelope.initial_volume & 0b1111) << 4;
        let envelope_direction = (volume_envelope.increase as u8) << 3;
        let period = volume_envelope.period & 0b111;
        initial_volume | envelope_direction | period
    }
}

impl std::convert::From<u8> for VolumeEnvelope {
    fn from(value: u8) -> Self {
        let initial_volume = (value & 0b11110000) >> 4;
        let envelope_direction = ((value & 0b1000) >> 3) != 0;
        let period = value & 0b111;
        VolumeEnvelope {
            initial_volume,
            increase: envelope_direction,
            period,
        }
    }
}

enum Trigger {
    Stopped,
    Playing,
    Restart,
}

enum PlayMode {
    Counter,
    Consecutive,
}

struct Frequency {
    frequency: u16,
}

pub struct StereoOutput {
    pub left: Vec<f32>,
    pub right: Vec<f32>,
}

impl Default for StereoOutput {
    fn default() -> Self {
        StereoOutput {
            left: vec![],
            right: vec![],
        }
    }
}

pub(super) trait Channel {
    fn initialize_buffer(&mut self, sample_rate: u32, clock_rate: u32);
    fn step(&mut self, cycles: u8);
    fn end_frame(&mut self, cycles: u32);
    fn gather_samples(&mut self) -> StereoOutput;
}

pub(super) struct SquareChannel {
    sweep: Option<Sweep>,
    duty: Duty,
    volume_envelope: VolumeEnvelope,
    frequency: Frequency,
    trigger: Trigger,
    play_mode: PlayMode,
    buffer: Option<BlipBuf>,
    current_sampling_cycle: u32,
    next_sample_cycle: u32,
    last_sample: i32,
}

fn gather_samples_for_buffer(buffer: Option<&mut BlipBuf>) -> StereoOutput {
    if buffer.is_none() {
        return StereoOutput::default();
    }

    let buffer = buffer.unwrap();
    let samples_available = buffer.samples_avail();

    let mut samples = vec![0; samples_available as usize];
    buffer.read_samples(samples.as_mut_slice(), false);

    let mut left_samples = vec![0f32; samples_available as usize];
    let mut right_samples = vec![0f32; samples_available as usize];
    for (idx, sample) in samples.iter().enumerate() {
        left_samples[idx] = *sample as f32;
        right_samples[idx] = *sample as f32;
    }

    StereoOutput {
        left: left_samples,
        right: right_samples,
    }
}

impl SquareChannel {
    pub fn new_with_sweep() -> Self {
        let default_sweep = Some(Sweep::from(0));
        Self::new(default_sweep)
    }

    pub fn new_without_sweep() -> Self {
        Self::new(None)
    }

    fn new(sweep: Option<Sweep>) -> Self {
        SquareChannel {
            sweep,
            duty: Duty {
                duty_type: DutyType::Eighth,
                length: 0,
                phase: 0,
            },
            volume_envelope: VolumeEnvelope {
                initial_volume: 0,
                increase: false,
                period: 0,
            },
            frequency: Frequency { frequency: 0 },
            trigger: Trigger::Stopped,
            play_mode: PlayMode::Consecutive,
            buffer: None,
            current_sampling_cycle: 0,
            next_sample_cycle: 0,
            last_sample: 0,
        }
    }
}

impl Channel for SquareChannel {
    fn initialize_buffer(&mut self, sample_rate: u32, clock_rate: u32) {
        let mut new_buffer = BlipBuf::new(sample_rate / 10);
        new_buffer.set_rates(clock_rate as f64, sample_rate as f64);
        self.buffer = Some(new_buffer);
    }

    fn step(&mut self, cycles: u8) {
        let end_cycle = self.current_sampling_cycle + cycles as u32;
        while self.next_sample_cycle < end_cycle {
            let sample = match self.trigger {
                Trigger::Stopped => 0,
                _ => {
                    let duty_sample = self.duty.sequence();
                    self.duty.step();
                    duty_sample
                }
            };

            let delta = sample - self.last_sample;
            if delta != 0 {
                if let Some(buffer) = self.buffer.as_mut() {
                    buffer.add_delta(self.next_sample_cycle, delta);
                }
            }

            self.last_sample = sample;
            // Square period = (2048 - F) / 131072 = (2048 - F) / (CPUClockRate * 32) = (2048 - F) * 8 * 4 / CPUClockRate
            // 8 duty entries per wave form => duty entry period = (2048 - F) * 4 / CPUClockRate
            // In CPUClockRate units => period = (2048 - F) * 4
            let period = (2048 - self.frequency.frequency) * 4;
            self.next_sample_cycle += period as u32;
        }
        self.current_sampling_cycle = end_cycle;
    }

    fn end_frame(&mut self, cycles: u32) {
        self.current_sampling_cycle -= cycles;
        self.next_sample_cycle -= cycles;
        if let Some(buffer) = &mut self.buffer {
            buffer.end_frame(cycles);
        }
    }

    fn gather_samples(&mut self) -> StereoOutput {
        gather_samples_for_buffer(self.buffer.as_mut())
    }
}

pub(super) struct NoiseRegister {}

impl NoiseRegister {
    pub fn read_nr10(channel: &SquareChannel) -> u8 {
        u8::from(channel.sweep.as_ref().unwrap())
    }

    pub fn write_nr10(value: u8, channel: &mut SquareChannel) {
        channel.sweep = Some(Sweep::from(value))
    }

    pub fn read_nr11(channel: &SquareChannel) -> u8 {
        u8::from(&channel.duty)
    }

    pub fn write_nr11(value: u8, channel: &mut SquareChannel) {
        channel.duty = Duty::from(value)
    }

    pub fn read_nr12(channel: &SquareChannel) -> u8 {
        u8::from(&channel.volume_envelope)
    }

    pub fn write_nr12(value: u8, channel: &mut SquareChannel) {
        channel.volume_envelope = VolumeEnvelope::from(value)
    }

    pub fn read_nr13(_channel: &SquareChannel) -> u8 {
        // Frequencies are unreadable
        0
    }

    pub fn write_nr13(value: u8, channel: &mut SquareChannel) {
        let msb = channel.frequency.frequency & 0xFF00;
        let lsb = value as u16;
        channel.frequency.frequency = msb | lsb;
    }

    pub fn read_nr14(channel: &SquareChannel) -> u8 {
        // Only the play-mode is readable
        let play_mode: u8 = match channel.play_mode {
            PlayMode::Counter => 1,
            PlayMode::Consecutive => 0,
        } << 6;
        play_mode
    }

    pub fn write_nr14(value: u8, channel: &mut SquareChannel) {
        if (value & 0b10000000) != 0 {
            channel.trigger = Trigger::Restart;
        }
        channel.play_mode = if (value & 0b01000000) != 0 {
            PlayMode::Counter
        } else {
            PlayMode::Consecutive
        };
        let freq_msb = ((value & 0b0111) as u16) << 8;
        let freq_lsb = channel.frequency.frequency & 0x00FF;
        channel.frequency.frequency = freq_msb | freq_lsb;
    }
}

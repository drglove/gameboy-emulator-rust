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
        Duty { duty_type, length }
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
        let period = (volume_envelope.period & 0b111);
        initial_volume | envelope_direction | period
    }
}

impl std::convert::From<u8> for VolumeEnvelope {
    fn from(value: u8) -> Self {
        let initial_volume = (value & 0b11110000) >> 4;
        let envelope_direction = ((value & 0b1000) >> 3) != 0;
        let period = (value & 0b111);
        VolumeEnvelope {
            initial_volume,
            increase: envelope_direction,
            period,
        }
    }
}

enum Trigger {
    Disabled,
    EnabledLooping,
    EnabledOneShot,
}

struct Frequency {
    frequency: u16,
}

pub(super) struct SquareChannel {
    sweep: Option<Sweep>,
    duty: Duty,
    volume_envelope: VolumeEnvelope,
    frequency: Frequency,
    trigger: Trigger,
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
            },
            volume_envelope: VolumeEnvelope {
                initial_volume: 0,
                increase: false,
                period: 0,
            },
            frequency: Frequency { frequency: 0 },
            trigger: Trigger::Disabled,
        }
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

    pub fn read_nr13(channel: &SquareChannel) -> u8 {
        (channel.frequency.frequency & 0xFF) as u8
    }

    pub fn write_nr13(value: u8, channel: &mut SquareChannel) {
        let msb = channel.frequency.frequency & 0xFF00;
        let lsb = value as u16;
        channel.frequency.frequency = msb | lsb;
    }
}

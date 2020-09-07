pub(super) struct Sweep {
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
    length: u16,
}

struct VolumeEnvelope {
    initial_volume: u8,
    decrease: bool,
    period: u8,
}

enum Trigger {
    Disabled,
    EnabledLooping,
    EnabledOneShot,
}

struct Frequency {
    frequency: u32,
}

pub(super) struct SquareChannel {
    pub sweep: Option<Sweep>,
    duty: Duty,
    volume_envelope: VolumeEnvelope,
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
                decrease: false,
                period: 0,
            },
            trigger: Trigger::Disabled,
        }
    }
}

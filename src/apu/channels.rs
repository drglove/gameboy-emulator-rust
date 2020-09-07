struct Sweep {
    period: u8,
    negate: bool,
    shift: u8,
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
    sweep: Option<Sweep>,
    duty: Duty,
    volume_envelope: VolumeEnvelope,
    trigger: Trigger,
}

impl SquareChannel {
    pub fn new() -> Self {
        SquareChannel {
            sweep: None,
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

    pub fn set_sweep(&mut self, value: u8) {
    }
}

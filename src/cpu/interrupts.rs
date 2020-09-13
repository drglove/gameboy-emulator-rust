use crate::memory::MemoryBus;
use std::ops::{BitAnd, BitOr, Not};

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum Interrupt {
    VBlank,
    LCDStat,
    Timer,
    Serial,
    Joypad,
}

impl Interrupt {
    const INTERRUPT_ENABLE_ADDRESS: u16 = 0xFFFF;
    const INTERRUPT_FLAG_ADDRESS: u16 = 0xFF0F;

    pub(super) fn should_process_interrupt(&self, bus: &MemoryBus) -> bool {
        self.is_interrupt_enabled(bus) && self.is_interrupt_flag_set(bus)
    }

    pub(super) fn is_interrupt_enabled(&self, bus: &MemoryBus) -> bool {
        let interrupt_enabled_byte = bus.read_byte(Interrupt::INTERRUPT_ENABLE_ADDRESS);
        let mask: u8 = self.interrupt_byte_mask();
        (interrupt_enabled_byte & mask) == mask
    }

    pub(super) fn is_interrupt_flag_set(&self, bus: &MemoryBus) -> bool {
        let interrupt_flag_byte = bus.read_byte(Interrupt::INTERRUPT_FLAG_ADDRESS);
        let mask = self.interrupt_byte_mask();
        (interrupt_flag_byte & mask) == mask
    }

    pub fn set_interrupt_flag(&self, bus: &mut MemoryBus) {
        self.set_interrupt_flag_to_value(true, bus);
    }

    pub(super) fn clear_interrupt_flag(&self, bus: &mut MemoryBus) {
        self.set_interrupt_flag_to_value(false, bus);
    }

    fn set_interrupt_flag_to_value(&self, value: bool, bus: &mut MemoryBus) {
        let interrupt_flag_byte = bus.read_byte(Interrupt::INTERRUPT_FLAG_ADDRESS);
        let mask = self.interrupt_byte_mask();
        let new_flag_byte = if value {
            interrupt_flag_byte.bitor(mask)
        } else {
            interrupt_flag_byte.bitand(mask.not())
        };
        bus.write_byte(new_flag_byte, Interrupt::INTERRUPT_FLAG_ADDRESS);
    }

    fn interrupt_byte_mask(&self) -> u8 {
        (match self {
            Interrupt::VBlank => 1 << 0,
            Interrupt::LCDStat => 1 << 1,
            Interrupt::Timer => 1 << 2,
            Interrupt::Serial => 1 << 3,
            Interrupt::Joypad => 1 << 4,
        }) as u8
    }
}

pub struct InterruptsToSet {
    interrupts: [bool; 5],
}

impl InterruptsToSet {
    pub fn is_interrupt_set(&self, interrupt: Interrupt) -> bool {
        match interrupt {
            Interrupt::VBlank => self.interrupts[0],
            Interrupt::LCDStat => self.interrupts[1],
            Interrupt::Timer => self.interrupts[2],
            Interrupt::Serial => self.interrupts[3],
            Interrupt::Joypad => self.interrupts[4],
        }
    }

    pub fn set_interrupt(&mut self, interrupt: Interrupt) {
        match interrupt {
            Interrupt::VBlank => self.interrupts[0] = true,
            Interrupt::LCDStat => self.interrupts[1] = true,
            Interrupt::Timer => self.interrupts[2] = true,
            Interrupt::Serial => self.interrupts[3] = true,
            Interrupt::Joypad => self.interrupts[4] = true,
        }
    }
}

impl Default for InterruptsToSet {
    fn default() -> Self {
        Self {
            interrupts: [false; 5],
        }
    }
}

use crate::memory::MemoryBus;
use std::ops::{BitOr, BitAnd, Not};

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

    pub(super) fn get_interrupts_to_process(bus: &MemoryBus) -> Vec<Self> {
        let mut interrupts: Vec<Interrupt> = vec![];
        let all_interrupts = vec![
            Interrupt::VBlank,
            Interrupt::LCDStat,
            Interrupt::Timer,
            Interrupt::Serial,
            Interrupt::Joypad,
        ];
        for interrupt in all_interrupts {
            if interrupt.is_interrupt_enabled(bus) && interrupt.is_interrupt_flag_set(bus) {
                interrupts.push(interrupt);
            }
        }
        interrupts
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

    pub(super) fn set_interrupt_flag(&self, bus: &mut MemoryBus) {
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

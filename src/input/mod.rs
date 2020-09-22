use crate::cpu::interrupts::{Interrupt, InterruptsToSet};
use std::sync::{Arc, Mutex};
use std::ops::Not;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct JoypadInput {
    pub start: bool,
    pub select: bool,
    pub a: bool,
    pub b: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

impl Default for JoypadInput {
    fn default() -> Self {
        Self {
            start: false,
            select: false,
            a: false,
            b: false,
            up: false,
            down: false,
            left: false,
            right: false,
        }
    }
}

pub struct InputState {
    pub select_buttons: bool,
    pub select_directions: bool,
    pub current_joypad: JoypadInput,
    pub next_joypad: Arc<Mutex<JoypadInput>>,
}

impl InputState {
    pub fn swap_to_next_joypad_state(&mut self) -> InterruptsToSet {
        let next_joypad_state_mutex = self.next_joypad.lock().unwrap();
        let next_joypad_state = *next_joypad_state_mutex;
        drop(next_joypad_state_mutex);

        let fire_interrupt = self.current_joypad != next_joypad_state;
        self.current_joypad = next_joypad_state;

        let mut interrupts = InterruptsToSet::default();
        if fire_interrupt {
            interrupts.set_interrupt(Interrupt::Joypad);
        }
        interrupts
    }

    pub fn step(&mut self) -> InterruptsToSet {
        self.swap_to_next_joypad_state()
    }

    pub fn supports_io_register(&self, address: usize) -> bool {
        address == 0xFF00
    }

    pub fn read_io_register(&self, _address: usize) -> u8 {
        let mut value = 0x00 as u8;
        if self.select_buttons {
            value = value | (1 << 5);
            if self.current_joypad.start {
                value = value | (1 << 3);
            }
            if self.current_joypad.select {
                value = value | (1 << 2);
            }
            if self.current_joypad.b {
                value = value | (1 << 1);
            }
            if self.current_joypad.a {
                value = value | (1 << 0);
            }
        }
        if self.select_directions {
            value = value & !(1 << 4);
            if self.current_joypad.down {
                value = value | (1 << 3);
            }
            if self.current_joypad.up {
                value = value | (1 << 2);
            }
            if self.current_joypad.left {
                value = value | (1 << 1);
            }
            if self.current_joypad.right {
                value = value | (1 << 0);
            }
        }
        value.not()
    }

    pub fn write_io_register(&mut self, value: u8, _address: usize) {
        self.select_buttons = (value.not() & (1 << 5)) != 0;
        self.select_directions = (value.not() & (1 << 4)) != 0;
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            select_buttons: false,
            select_directions: false,
            current_joypad: Default::default(),
            next_joypad: Arc::new(Mutex::new(Default::default())),
        }
    }
}

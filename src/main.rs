struct Registers {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: FlagsRegister,
    h: u8,
    l: u8,
    pc: u16,
    sp: u16,
}

enum Instruction {
    ADD(ArithmeticTarget),
    LD(LoadTarget),
    XOR(ArithmeticTarget),
}

impl Instruction {
    fn from_byte(byte: u8) -> Option<Instruction> {
        match byte {
            0x80 | 0x88 => Some(Instruction::ADD(ArithmeticTarget::B)),
            0x81 | 0x89 => Some(Instruction::ADD(ArithmeticTarget::C)),
            0x82 | 0x8A => Some(Instruction::ADD(ArithmeticTarget::D)),
            0x83 | 0x8B => Some(Instruction::ADD(ArithmeticTarget::E)),
            0x84 | 0x8C => Some(Instruction::ADD(ArithmeticTarget::H)),
            0x85 | 0x8D => Some(Instruction::ADD(ArithmeticTarget::L)),
            0x87 | 0x8F => Some(Instruction::ADD(ArithmeticTarget::A)),
            0x01 => Some(Instruction::LD(LoadTarget::BC)),
            0x11 => Some(Instruction::LD(LoadTarget::DE)),
            0x21 => Some(Instruction::LD(LoadTarget::HL)),
            0x31 => Some(Instruction::LD(LoadTarget::SP)),
            0xA8 => Some(Instruction::XOR(ArithmeticTarget::B)),
            0xA9 => Some(Instruction::XOR(ArithmeticTarget::C)),
            0xAA => Some(Instruction::XOR(ArithmeticTarget::D)),
            0xAB => Some(Instruction::XOR(ArithmeticTarget::E)),
            0xAC => Some(Instruction::XOR(ArithmeticTarget::H)),
            0xAD => Some(Instruction::XOR(ArithmeticTarget::L)),
            0xAF => Some(Instruction::XOR(ArithmeticTarget::A)),
            _ => None,
        }
    }
}

enum ArithmeticTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

enum LoadTarget {
    BC,
    DE,
    HL,
    SP,
}

impl Registers {
    fn get_bc(&self) -> u16 {
        ((self.b as u16) << 8) | (self.c as u16)
    }

    fn set_bc(&mut self, value: u16) {
        self.b = ((value & 0xFF00) >> 8) as u8;
        self.c = (value & 0x00FF) as u8;
    }

    fn get_de(&self) -> u16 {
        ((self.d as u16) << 8) | (self.e as u16)
    }

    fn set_de(&mut self, value: u16) {
        self.d = ((value & 0xFF00) >> 8) as u8;
        self.e = (value & 0x00FF) as u8;
    }

    fn get_hl(&self) -> u16 {
        ((self.h as u16) << 8) | (self.l as u16)
    }

    fn set_hl(&mut self, value: u16) {
        self.h = ((value & 0xFF00) >> 8) as u8;
        self.l = (value & 0x00FF) as u8;
    }
}

struct FlagsRegister {
    zero: bool,
    subtract: bool,
    half_carry: bool,
    carry: bool,
}

const ZERO_FLAG_BYTE_POSITION: u8 = 7;
const SUBTRACT_FLAG_BYTE_POSITION: u8 = 6;
const HALF_CARRY_FLAG_BYTE_POSITION: u8 = 5;
const CARRY_FLAG_BYTE_POSITION: u8 = 4;

impl std::convert::From<FlagsRegister> for u8 {
    fn from(flag: FlagsRegister) -> u8 {
        (if flag.zero { 1 } else { 0 }) << ZERO_FLAG_BYTE_POSITION
            | (if flag.subtract { 1 } else { 0 }) << SUBTRACT_FLAG_BYTE_POSITION
            | (if flag.half_carry { 1 } else { 0 }) << HALF_CARRY_FLAG_BYTE_POSITION
            | (if flag.carry { 1 } else { 0 }) << CARRY_FLAG_BYTE_POSITION
    }
}

impl std::convert::From<u8> for FlagsRegister {
    fn from(value: u8) -> Self {
        let zero = ((value >> ZERO_FLAG_BYTE_POSITION) & 0b1) != 0;
        let subtract = ((value >> SUBTRACT_FLAG_BYTE_POSITION) & 0b1) != 0;
        let half_carry = ((value >> HALF_CARRY_FLAG_BYTE_POSITION) & 0b1) != 0;
        let carry = ((value >> CARRY_FLAG_BYTE_POSITION) & 0b1) != 0;

        FlagsRegister {
            zero,
            subtract,
            half_carry,
            carry,
        }
    }
}

struct DMG01 {
    cpu: CPU,
}

impl DMG01 {
    fn new(cart: Option<Cartridge>) -> DMG01 {
        use std::cmp::min;

        let mut memory: [u8; 0xFFFF] = [0; 0xFFFF];
        let size_to_copy = min(0xFFFF, cart.as_ref().unwrap().rom.len());
        memory[0..size_to_copy].copy_from_slice(&cart.unwrap().rom.as_slice());

        DMG01 {
            cpu: CPU {
                registers: Registers {
                    a: 0,
                    b: 0,
                    c: 0,
                    d: 0,
                    e: 0,
                    f: FlagsRegister {
                        zero: false,
                        subtract: false,
                        half_carry: false,
                        carry: false,
                    },
                    h: 0,
                    l: 0,
                    pc: 0,
                    sp: 0,
                },
                bus: MemoryBus { memory },
            },
        }
    }
}

struct CPU {
    registers: Registers,
    bus: MemoryBus,
}

struct MemoryBus {
    memory: [u8; 0xFFFF],
}

impl MemoryBus {
    fn read_byte(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }
}

impl CPU {
    fn step(&mut self) {
        let instruction_byte = self.bus.read_byte(self.registers.pc);
        let next_pc = if let Some(instruction) = Instruction::from_byte(instruction_byte) {
            self.execute(instruction)
        } else {
            panic!("Unknown instruction found for 0x{:x}", instruction_byte);
        };

        self.registers.pc = next_pc;
    }

    fn execute(&mut self, instruction: Instruction) -> u16 {
        match instruction {
            Instruction::ADD(target) => {
                let value = match target {
                    ArithmeticTarget::A => self.registers.a,
                    ArithmeticTarget::B => self.registers.b,
                    ArithmeticTarget::C => self.registers.c,
                    ArithmeticTarget::D => self.registers.d,
                    ArithmeticTarget::E => self.registers.e,
                    ArithmeticTarget::H => self.registers.h,
                    ArithmeticTarget::L => self.registers.l,
                };
                let new_value = self.add(value);
                self.registers.a = new_value;
                self.registers.pc.wrapping_add(1)
            }
            Instruction::LD(load_target) => {
                let immediate_value = self.read_next_word();
                match load_target {
                    LoadTarget::BC => self.registers.set_bc(immediate_value),
                    LoadTarget::DE => self.registers.set_de(immediate_value),
                    LoadTarget::HL => self.registers.set_hl(immediate_value),
                    LoadTarget::SP => self.registers.sp = immediate_value,
                }
                self.registers.pc.wrapping_add(3)
            }
            Instruction::XOR(target) => {
                let value = match target {
                    ArithmeticTarget::A => self.registers.a,
                    ArithmeticTarget::B => self.registers.b,
                    ArithmeticTarget::C => self.registers.c,
                    ArithmeticTarget::D => self.registers.d,
                    ArithmeticTarget::E => self.registers.e,
                    ArithmeticTarget::H => self.registers.h,
                    ArithmeticTarget::L => self.registers.l,
                };
                let new_value = self.xor(value);
                self.registers.a = new_value;
                self.registers.pc.wrapping_add(1)
            }
        }
    }

    fn read_next_word(&self) -> u16 {
        (self.bus.read_byte(self.registers.pc + 2) as u16) << 8
            | self.bus.read_byte(self.registers.pc + 1) as u16
    }

    fn add(&mut self, value: u8) -> u8 {
        let (new_value, did_overflow) = self.registers.a.overflowing_add(value);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = did_overflow;
        // Half-carry is true if adding the values of the lower nibbles of A and the value
        // results in a carry into the upper nibble.
        self.registers.f.half_carry = (self.registers.a & 0x0F) + (value & 0x0F) > 0x0F;
        new_value
    }

    fn xor(&mut self, value: u8) -> u8 {
        let new_value = self.registers.a.bitxor(value);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = false;
        self.registers.f.half_carry = false;
        new_value
    }
}

struct Cartridge {
    rom: Vec<u8>,
}

use std::ops::BitXor;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str), long)]
    rom: Option<std::path::PathBuf>,
}

fn main() {
    let args = Cli::from_args();

    use std::fs;
    let cart = if let Some(rom_path) = args.rom {
        Some(Cartridge {
            rom: fs::read(rom_path).expect("Could not open rom file!"),
        })
    } else {
        None
    };

    let mut gameboy = DMG01::new(cart);
    loop {
        gameboy.cpu.step();
    }
}

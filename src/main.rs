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
    XOR(ArithmeticTarget),
    LD(LoadType),
}

enum LoadType {
    ReadWordNumericLiteral(WordRegister, WordNumericLiteral),
    ReadByteNumericLiteral(ByteRegister, ByteNumericLiteral),
    ReadByteFromAddressOffset(ByteRegister, AddressOffsetContainingRegister),
    ReadByteFromAddressLiteral(ByteRegister, AddressLiteral),
    ReadByteFromAddressOffsetLiteral(ByteRegister, AddressOffsetLiteral),
    ReadByteFromAddress(ByteRegister, AddressContainingRegister),
    WriteByteFromRegisterToAddressOffsetLiteral(AddressOffsetLiteral, ByteRegister),
    WriteByteFromRegisterToAddressLiteral(AddressLiteral, ByteRegister),
    WriteByteFromRegisterToAddressOffsetRegister(AddressOffsetContainingRegister, ByteRegister),
    WriteByteFromRegisterToAddressContainedInRegister(AddressContainingRegister, ByteRegister),
    WriteByteLiteralToAddressContainedInRegister(AddressContainingRegister, ByteNumericLiteral),
    WriteWordInRegisterToAddressContainedInLiteral(AddressLiteral, WordRegister),
    CopyByteFromRegisterToRegister(ByteRegister, ByteRegister),
    CopyWordFromRegisterToRegister(WordRegister, WordRegister),
    CopyStackOffsetToRegister(WordRegister, StackOffset),
}

enum ByteRegister {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

impl ByteRegister {
    fn get_byte(&self, registers: &Registers) -> u8 {
        match self {
            ByteRegister::A => registers.a,
            ByteRegister::B => registers.b,
            ByteRegister::C => registers.c,
            ByteRegister::D => registers.d,
            ByteRegister::E => registers.e,
            ByteRegister::H => registers.h,
            ByteRegister::L => registers.l,
        }
    }

    fn set_byte(&self, value: u8, registers: &mut Registers) {
        match self {
            ByteRegister::A => registers.a = value,
            ByteRegister::B => registers.b = value,
            ByteRegister::C => registers.c = value,
            ByteRegister::D => registers.d = value,
            ByteRegister::E => registers.e = value,
            ByteRegister::H => registers.h = value,
            ByteRegister::L => registers.l = value,
        }
    }
}

enum WordRegister {
    BC,
    DE,
    HL,
    SP,
}

impl WordRegister {
    fn get_word(&self, registers: &Registers) -> u16 {
        match self {
            WordRegister::BC => registers.get_bc(),
            WordRegister::DE => registers.get_de(),
            WordRegister::HL => registers.get_hl(),
            WordRegister::SP => registers.sp,
        }
    }

    fn set_word(&self, value: u16, registers: &mut Registers) {
        match self {
            WordRegister::BC => registers.set_bc(value),
            WordRegister::DE => registers.set_de(value),
            WordRegister::HL => registers.set_hl(value),
            WordRegister::SP => registers.sp = value,
        }
    }
}

enum AddressContainingRegister {
    BC,
    DE,
    HL,
    HLI,
    HLD,
}

impl AddressContainingRegister {
    fn get_address(&self, registers: &Registers) -> u16 {
        match self {
            AddressContainingRegister::BC => registers.get_bc(),
            AddressContainingRegister::DE => registers.get_de(),
            AddressContainingRegister::HL
            | AddressContainingRegister::HLI
            | AddressContainingRegister::HLD => registers.get_hl(),
        }
    }
}

enum AddressOffsetContainingRegister {
    C,
}

impl AddressOffsetContainingRegister {
    fn get_address_offset(&self, registers: &Registers) -> u8 {
        match self {
            AddressOffsetContainingRegister::C => registers.c,
        }
    }
}

enum AddressLiteral {
    A16,
}

enum AddressOffsetLiteral {
    A8,
}

enum ByteNumericLiteral {
    D8,
}

enum WordNumericLiteral {
    D16,
}

enum StackOffset {
    SPOffset,
}

impl Instruction {
    #[rustfmt::skip]
    fn from_byte(byte: u8, prefix_instruction: bool) -> Option<Instruction> {
        if !prefix_instruction {
            match byte {
                0x80 => Some(Instruction::ADD(ArithmeticTarget::B)),
                0x81 => Some(Instruction::ADD(ArithmeticTarget::C)),
                0x82 => Some(Instruction::ADD(ArithmeticTarget::D)),
                0x83 => Some(Instruction::ADD(ArithmeticTarget::E)),
                0x84 => Some(Instruction::ADD(ArithmeticTarget::H)),
                0x85 => Some(Instruction::ADD(ArithmeticTarget::L)),
                0x87 => Some(Instruction::ADD(ArithmeticTarget::A)),
                0xA8 => Some(Instruction::XOR(ArithmeticTarget::B)),
                0xA9 => Some(Instruction::XOR(ArithmeticTarget::C)),
                0xAA => Some(Instruction::XOR(ArithmeticTarget::D)),
                0xAB => Some(Instruction::XOR(ArithmeticTarget::E)),
                0xAC => Some(Instruction::XOR(ArithmeticTarget::H)),
                0xAD => Some(Instruction::XOR(ArithmeticTarget::L)),
                0xAF => Some(Instruction::XOR(ArithmeticTarget::A)),
                0x01 => Some(Instruction::LD(LoadType::ReadWordNumericLiteral(WordRegister::BC, WordNumericLiteral::D16))),
                0x11 => Some(Instruction::LD(LoadType::ReadWordNumericLiteral(WordRegister::DE, WordNumericLiteral::D16))),
                0x21 => Some(Instruction::LD(LoadType::ReadWordNumericLiteral(WordRegister::HL, WordNumericLiteral::D16))),
                0x31 => Some(Instruction::LD(LoadType::ReadWordNumericLiteral(WordRegister::SP, WordNumericLiteral::D16))),
                0x02 => Some(Instruction::LD(LoadType::WriteByteFromRegisterToAddressContainedInRegister(AddressContainingRegister::BC, ByteRegister::A))),
                0x12 => Some(Instruction::LD(LoadType::WriteByteFromRegisterToAddressContainedInRegister(AddressContainingRegister::DE, ByteRegister::A))),
                0x22 => Some(Instruction::LD(LoadType::WriteByteFromRegisterToAddressContainedInRegister(AddressContainingRegister::HLI, ByteRegister::A))),
                0x32 => Some(Instruction::LD(LoadType::WriteByteFromRegisterToAddressContainedInRegister(AddressContainingRegister::HLD, ByteRegister::A))),
                0x06 => Some(Instruction::LD(LoadType::ReadByteNumericLiteral(ByteRegister::B, ByteNumericLiteral::D8))),
                0x0E => Some(Instruction::LD(LoadType::ReadByteNumericLiteral(ByteRegister::C, ByteNumericLiteral::D8))),
                0x16 => Some(Instruction::LD(LoadType::ReadByteNumericLiteral(ByteRegister::D, ByteNumericLiteral::D8))),
                0x1E => Some(Instruction::LD(LoadType::ReadByteNumericLiteral(ByteRegister::E, ByteNumericLiteral::D8))),
                0x26 => Some(Instruction::LD(LoadType::ReadByteNumericLiteral(ByteRegister::H, ByteNumericLiteral::D8))),
                0x2E => Some(Instruction::LD(LoadType::ReadByteNumericLiteral(ByteRegister::L, ByteNumericLiteral::D8))),
                0x36 => Some(Instruction::LD(LoadType::WriteByteLiteralToAddressContainedInRegister(AddressContainingRegister::HL, ByteNumericLiteral::D8))),
                0x3E => Some(Instruction::LD(LoadType::ReadByteNumericLiteral(ByteRegister::A, ByteNumericLiteral::D8))),
                0x08 => Some(Instruction::LD(LoadType::WriteWordInRegisterToAddressContainedInLiteral(AddressLiteral::A16, WordRegister::SP))),
                0x0A => Some(Instruction::LD(LoadType::ReadByteFromAddress(ByteRegister::A, AddressContainingRegister::BC))),
                0x1A => Some(Instruction::LD(LoadType::ReadByteFromAddress(ByteRegister::A, AddressContainingRegister::DE))),
                0x2A => Some(Instruction::LD(LoadType::ReadByteFromAddress(ByteRegister::A, AddressContainingRegister::HLI))),
                0x3A => Some(Instruction::LD(LoadType::ReadByteFromAddress(ByteRegister::A, AddressContainingRegister::HLD))),
                0x40 => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::B, ByteRegister::B))),
                0x41 => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::B, ByteRegister::C))),
                0x42 => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::B, ByteRegister::D))),
                0x43 => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::B, ByteRegister::E))),
                0x44 => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::B, ByteRegister::H))),
                0x45 => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::B, ByteRegister::L))),
                0x46 => Some(Instruction::LD(LoadType::ReadByteFromAddress(ByteRegister::B, AddressContainingRegister::HL))),
                0x47 => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::B, ByteRegister::A))),
                0x48 => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::C, ByteRegister::B))),
                0x49 => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::C, ByteRegister::C))),
                0x4A => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::C, ByteRegister::D))),
                0x4B => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::C, ByteRegister::E))),
                0x4C => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::C, ByteRegister::H))),
                0x4D => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::C, ByteRegister::L))),
                0x4E => Some(Instruction::LD(LoadType::ReadByteFromAddress(ByteRegister::C, AddressContainingRegister::HL))),
                0x4F => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::C, ByteRegister::A))),
                0x50 => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::D, ByteRegister::B))),
                0x51 => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::D, ByteRegister::C))),
                0x52 => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::D, ByteRegister::D))),
                0x53 => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::D, ByteRegister::E))),
                0x54 => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::D, ByteRegister::H))),
                0x55 => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::D, ByteRegister::L))),
                0x56 => Some(Instruction::LD(LoadType::ReadByteFromAddress(ByteRegister::D, AddressContainingRegister::HL))),
                0x57 => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::D, ByteRegister::A))),
                0x58 => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::E, ByteRegister::B))),
                0x59 => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::E, ByteRegister::C))),
                0x5A => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::E, ByteRegister::D))),
                0x5B => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::E, ByteRegister::E))),
                0x5C => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::E, ByteRegister::H))),
                0x5D => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::E, ByteRegister::L))),
                0x5E => Some(Instruction::LD(LoadType::ReadByteFromAddress(ByteRegister::E, AddressContainingRegister::HL))),
                0x5F => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::E, ByteRegister::A))),
                0x60 => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::H, ByteRegister::B))),
                0x61 => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::H, ByteRegister::C))),
                0x62 => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::H, ByteRegister::D))),
                0x63 => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::H, ByteRegister::E))),
                0x64 => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::H, ByteRegister::H))),
                0x65 => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::H, ByteRegister::L))),
                0x66 => Some(Instruction::LD(LoadType::ReadByteFromAddress(ByteRegister::H, AddressContainingRegister::HL))),
                0x67 => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::H, ByteRegister::A))),
                0x68 => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::L, ByteRegister::B))),
                0x69 => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::L, ByteRegister::C))),
                0x6A => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::L, ByteRegister::D))),
                0x6B => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::L, ByteRegister::E))),
                0x6C => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::L, ByteRegister::H))),
                0x6D => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::L, ByteRegister::L))),
                0x6E => Some(Instruction::LD(LoadType::ReadByteFromAddress(ByteRegister::L, AddressContainingRegister::HL))),
                0x6F => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::L, ByteRegister::A))),
                0x70 => Some(Instruction::LD(LoadType::WriteByteFromRegisterToAddressContainedInRegister(AddressContainingRegister::HL, ByteRegister::B))),
                0x71 => Some(Instruction::LD(LoadType::WriteByteFromRegisterToAddressContainedInRegister(AddressContainingRegister::HL, ByteRegister::C))),
                0x72 => Some(Instruction::LD(LoadType::WriteByteFromRegisterToAddressContainedInRegister(AddressContainingRegister::HL, ByteRegister::D))),
                0x73 => Some(Instruction::LD(LoadType::WriteByteFromRegisterToAddressContainedInRegister(AddressContainingRegister::HL, ByteRegister::E))),
                0x74 => Some(Instruction::LD(LoadType::WriteByteFromRegisterToAddressContainedInRegister(AddressContainingRegister::HL, ByteRegister::H))),
                0x75 => Some(Instruction::LD(LoadType::WriteByteFromRegisterToAddressContainedInRegister(AddressContainingRegister::HL, ByteRegister::L))),
                0x77 => Some(Instruction::LD(LoadType::WriteByteFromRegisterToAddressContainedInRegister(AddressContainingRegister::HL, ByteRegister::A))),
                0x78 => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::A, ByteRegister::B))),
                0x79 => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::A, ByteRegister::C))),
                0x7A => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::A, ByteRegister::D))),
                0x7B => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::A, ByteRegister::E))),
                0x7C => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::A, ByteRegister::H))),
                0x7D => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::A, ByteRegister::L))),
                0x7E => Some(Instruction::LD(LoadType::ReadByteFromAddress(ByteRegister::A, AddressContainingRegister::HL))),
                0x7F => Some(Instruction::LD(LoadType::CopyByteFromRegisterToRegister(ByteRegister::A, ByteRegister::A))),
                0xE0 => Some(Instruction::LD(LoadType::WriteByteFromRegisterToAddressOffsetLiteral(AddressOffsetLiteral::A8, ByteRegister::A))),
                0xE2 => Some(Instruction::LD(LoadType::WriteByteFromRegisterToAddressOffsetRegister(AddressOffsetContainingRegister::C, ByteRegister::A))),
                0xEA => Some(Instruction::LD(LoadType::WriteByteFromRegisterToAddressLiteral(AddressLiteral::A16, ByteRegister::A))),
                0xF0 => Some(Instruction::LD(LoadType::ReadByteFromAddressOffsetLiteral(ByteRegister::A, AddressOffsetLiteral::A8))),
                0xF2 => Some(Instruction::LD(LoadType::ReadByteFromAddressOffset(ByteRegister::A, AddressOffsetContainingRegister::C))),
                0xFA => Some(Instruction::LD(LoadType::ReadByteFromAddressLiteral(ByteRegister::A, AddressLiteral::A16))),
                0xF8 => Some(Instruction::LD(LoadType::CopyStackOffsetToRegister(WordRegister::HL, StackOffset::SPOffset))),
                0xF9 => Some(Instruction::LD(LoadType::CopyWordFromRegisterToRegister(WordRegister::SP, WordRegister::HL))),
                _ => None,
            }
        }
        else {
            None
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

    fn read_byte_from_offset(&self, address_offset: u8) -> u8 {
        self.memory[address_offset as usize + 0xFF00]
    }

    fn write_byte(&mut self, value: u8, address: u16) {
        self.memory[address as usize] = value;
    }

    fn write_byte_to_offset(&mut self, value: u8, address_offset: u8) {
        self.memory[address_offset as usize + 0xFF00] = value;
    }
}

impl CPU {
    fn step(&mut self) {
        let instruction = self.next_instruction().unwrap();
        let next_pc = self.execute(instruction);
        self.registers.pc = next_pc;
    }

    fn next_instruction(&self) -> Result<Instruction, String> {
        let mut instruction_byte = self.bus.read_byte(self.registers.pc);
        let prefix_instruction = instruction_byte == 0xCB;
        if prefix_instruction {
            instruction_byte = self.bus.read_byte(self.registers.pc.wrapping_add(1));
        }

        Instruction::from_byte(instruction_byte, prefix_instruction).ok_or(format!(
            "Unknown instruction found for 0x{:x}",
            instruction_byte
        ))
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
            Instruction::LD(load_type) => {
                return match load_type {
                    LoadType::ReadWordNumericLiteral(target, _) => {
                        let value = self.read_next_word();
                        target.set_word(value, &mut self.registers);
                        self.registers.pc.wrapping_add(3)
                    }
                    LoadType::ReadByteNumericLiteral(target, _) => {
                        let value = self.read_next_byte();
                        target.set_byte(value, &mut self.registers);
                        self.registers.pc.wrapping_add(2)
                    }
                    LoadType::ReadByteFromAddressOffset(target, source) => {
                        let address_offset = source.get_address_offset(&self.registers);
                        let value = self.bus.read_byte_from_offset(address_offset);
                        target.set_byte(value, &mut self.registers);
                        self.registers.pc.wrapping_add(2)
                    }
                    LoadType::ReadByteFromAddressLiteral(target, _) => {
                        let address = self.read_next_word();
                        let value = self.bus.read_byte(address);
                        target.set_byte(value, &mut self.registers);
                        self.registers.pc.wrapping_add(3)
                    }
                    LoadType::ReadByteFromAddressOffsetLiteral(target, _) => {
                        let address_offset = self.read_next_byte();
                        let value = self.bus.read_byte_from_offset(address_offset);
                        target.set_byte(value, &mut self.registers);
                        self.registers.pc.wrapping_add(2)
                    }
                    LoadType::ReadByteFromAddress(target, source) => {
                        let address = source.get_address(&self.registers);
                        let value = self.bus.read_byte(address);
                        target.set_byte(value, &mut self.registers);
                        match source {
                            AddressContainingRegister::HLI => self
                                .registers
                                .set_hl(self.registers.get_hl().wrapping_add(1)),
                            AddressContainingRegister::HLD => self
                                .registers
                                .set_hl(self.registers.get_hl().wrapping_sub(1)),
                            _ => {}
                        }
                        self.registers.pc.wrapping_add(1)
                    }
                    LoadType::WriteByteFromRegisterToAddressContainedInRegister(target, source) => {
                        let address = target.get_address(&self.registers);
                        let value = source.get_byte(&self.registers);
                        self.bus.write_byte(value, address);
                        match target {
                            AddressContainingRegister::HLI => self
                                .registers
                                .set_hl(self.registers.get_hl().wrapping_add(1)),
                            AddressContainingRegister::HLD => self
                                .registers
                                .set_hl(self.registers.get_hl().wrapping_sub(1)),
                            _ => {}
                        }
                        self.registers.pc.wrapping_add(1)
                    }
                    LoadType::WriteByteFromRegisterToAddressOffsetLiteral(_, source) => {
                        let address_offset = self.read_next_byte();
                        let value = source.get_byte(&self.registers);
                        self.bus.write_byte_to_offset(value, address_offset);
                        self.registers.pc.wrapping_add(2)
                    }
                    LoadType::WriteByteFromRegisterToAddressLiteral(_, source) => {
                        let address = self.read_next_word();
                        let value = source.get_byte(&self.registers);
                        self.bus.write_byte(value, address);
                        self.registers.pc.wrapping_add(3)
                    }
                    LoadType::WriteByteFromRegisterToAddressOffsetRegister(target, source) => {
                        let address_offset = target.get_address_offset(&self.registers);
                        let value = source.get_byte(&self.registers);
                        self.bus.write_byte_to_offset(value, address_offset);
                        self.registers.pc.wrapping_add(2)
                    }
                    LoadType::WriteByteLiteralToAddressContainedInRegister(target, _) => {
                        let address = target.get_address(&self.registers);
                        let value = self.read_next_byte();
                        self.bus.write_byte(value, address);
                        self.registers.pc.wrapping_add(2)
                    }
                    LoadType::WriteWordInRegisterToAddressContainedInLiteral(_, source) => {
                        let address = self.read_next_word();
                        let value = source.get_word(&self.registers);
                        self.bus.write_byte((value & 0x00FF) as u8, address);
                        self.bus
                            .write_byte(((value & 0xFF00) >> 8) as u8, address.wrapping_add(1));
                        self.registers.pc.wrapping_add(3)
                    }
                    LoadType::CopyByteFromRegisterToRegister(target, source) => {
                        let value = source.get_byte(&self.registers);
                        target.set_byte(value, &mut self.registers);
                        self.registers.pc.wrapping_add(1)
                    }
                    LoadType::CopyWordFromRegisterToRegister(target, source) => {
                        let value = source.get_word(&self.registers);
                        target.set_word(value, &mut self.registers);
                        self.registers.pc.wrapping_add(1)
                    }
                    LoadType::CopyStackOffsetToRegister(target, _) => {
                        let offset = self.read_next_byte() as i8 as u16;
                        let value = self.registers.sp.wrapping_add(offset);
                        target.set_word(value, &mut self.registers);
                        self.registers.f.zero = false;
                        self.registers.f.subtract = false;
                        self.registers.f.half_carry =
                            (self.registers.sp & 0x0F) + (value & 0x0F) > 0x0F;
                        self.registers.f.carry = (self.registers.sp & 0xFF) + (value & 0xFF) > 0xFF;
                        self.registers.pc.wrapping_add(2)
                    }
                }
            }
        }
    }

    fn read_next_word(&self) -> u16 {
        (self.bus.read_byte(self.registers.pc + 2) as u16) << 8
            | self.bus.read_byte(self.registers.pc + 1) as u16
    }

    fn read_next_byte(&self) -> u8 {
        self.bus.read_byte(self.registers.pc + 1)
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

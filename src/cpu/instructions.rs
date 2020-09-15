use crate::cpu::registers::Registers;
use crate::cpu::CPU;
use core::option::Option::{None, Some};

#[allow(non_camel_case_types)]
pub(super) enum Instruction {
    NOP,
    ADD(ArithmeticSource),
    ADD_HL(WordRegister),
    ADD_SP(),
    SUB(ArithmeticSource),
    CP(ArithmeticSource),
    XOR(ArithmeticSource),
    AND(ArithmeticSource),
    OR(ArithmeticSource),
    LD(LoadType),
    JR(JumpCondition),
    JP(JumpCondition, JumpTarget),
    CALL(JumpCondition),
    RET(JumpCondition),
    PUSH(WordRegister),
    POP(WordRegister),
    RST(RestartTarget),
    INC(IncrementDecrementTarget),
    DEC(IncrementDecrementTarget),
    RL(ArithmeticSource),
    RLA(),
    RR(ArithmeticSource),
    SCF,
    CPL,
    SWAP(ArithmeticSource),
    BIT(u8, ArithmeticSource),
    DI,
    EI,
    RETI,
}

pub(super) enum LoadType {
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

pub(super) enum JumpCondition {
    Always,
    Zero,
    NotZero,
    Carry,
    NoCarry,
}

impl JumpCondition {
    pub(super) fn take_jump(&self, registers: &Registers) -> bool {
        match self {
            JumpCondition::Always => true,
            JumpCondition::Zero => registers.f.zero,
            JumpCondition::NotZero => !registers.f.zero,
            JumpCondition::Carry => registers.f.carry,
            JumpCondition::NoCarry => !registers.f.carry,
        }
    }
}

#[allow(non_camel_case_types)]
pub(super) enum JumpTarget {
    A16,
    HL_INDIRECT,
}

pub type RestartTarget = u8;

pub(super) enum IncrementDecrementTarget {
    Byte(ArithmeticSource),
    Word(WordRegister),
}

pub(super) enum ByteRegister {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

impl ByteRegister {
    pub(super) fn get_byte(&self, registers: &Registers) -> u8 {
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

    pub(super) fn set_byte(&self, value: u8, registers: &mut Registers) {
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

pub(super) enum WordRegister {
    BC,
    DE,
    HL,
    SP,
    AF,
}

impl WordRegister {
    pub(super) fn get_word(&self, registers: &Registers) -> u16 {
        match self {
            WordRegister::BC => registers.get_bc(),
            WordRegister::DE => registers.get_de(),
            WordRegister::HL => registers.get_hl(),
            WordRegister::AF => registers.get_af(),
            WordRegister::SP => registers.sp,
        }
    }

    pub(super) fn set_word(&self, value: u16, registers: &mut Registers) {
        match self {
            WordRegister::BC => registers.set_bc(value),
            WordRegister::DE => registers.set_de(value),
            WordRegister::HL => registers.set_hl(value),
            WordRegister::AF => registers.set_af(value),
            WordRegister::SP => registers.sp = value,
        }
    }
}

pub(super) enum AddressContainingRegister {
    BC,
    DE,
    HL,
    HLI,
    HLD,
}

impl AddressContainingRegister {
    pub(super) fn get_address(&self, registers: &Registers) -> u16 {
        match self {
            AddressContainingRegister::BC => registers.get_bc(),
            AddressContainingRegister::DE => registers.get_de(),
            AddressContainingRegister::HL
            | AddressContainingRegister::HLI
            | AddressContainingRegister::HLD => registers.get_hl(),
        }
    }
}

pub(super) enum AddressOffsetContainingRegister {
    C,
}

impl AddressOffsetContainingRegister {
    pub(super) fn get_address_offset(&self, registers: &Registers) -> u8 {
        match self {
            AddressOffsetContainingRegister::C => registers.c,
        }
    }
}

pub(super) enum AddressLiteral {
    A16,
}

pub(super) enum AddressOffsetLiteral {
    A8,
}

pub(super) enum ByteNumericLiteral {
    D8,
}

pub(super) enum WordNumericLiteral {
    D16,
}

pub(super) enum StackOffset {
    SPOffset,
}

impl Instruction {
    #[rustfmt::skip]
    pub fn from_byte(byte: u8, prefix_instruction: bool) -> Option<Instruction> {
        if !prefix_instruction {
            match byte {
                0x00 => Some(Instruction::NOP),
                0x80 => Some(Instruction::ADD(ArithmeticSource::B)),
                0x81 => Some(Instruction::ADD(ArithmeticSource::C)),
                0x82 => Some(Instruction::ADD(ArithmeticSource::D)),
                0x83 => Some(Instruction::ADD(ArithmeticSource::E)),
                0x84 => Some(Instruction::ADD(ArithmeticSource::H)),
                0x85 => Some(Instruction::ADD(ArithmeticSource::L)),
                0x86 => Some(Instruction::ADD(ArithmeticSource::HL_INDIRECT)),
                0x87 => Some(Instruction::ADD(ArithmeticSource::A)),
                0xC6 => Some(Instruction::ADD(ArithmeticSource::D8)),
                0x09 => Some(Instruction::ADD_HL(WordRegister::BC)),
                0x19 => Some(Instruction::ADD_HL(WordRegister::DE)),
                0x29 => Some(Instruction::ADD_HL(WordRegister::HL)),
                0x39 => Some(Instruction::ADD_HL(WordRegister::SP)),
                0xE8 => Some(Instruction::ADD_SP()),
                0x90 => Some(Instruction::SUB(ArithmeticSource::B)),
                0x91 => Some(Instruction::SUB(ArithmeticSource::C)),
                0x92 => Some(Instruction::SUB(ArithmeticSource::D)),
                0x93 => Some(Instruction::SUB(ArithmeticSource::E)),
                0x94 => Some(Instruction::SUB(ArithmeticSource::H)),
                0x95 => Some(Instruction::SUB(ArithmeticSource::L)),
                0x96 => Some(Instruction::SUB(ArithmeticSource::HL_INDIRECT)),
                0x97 => Some(Instruction::SUB(ArithmeticSource::A)),
                0xD6 => Some(Instruction::SUB(ArithmeticSource::D8)),
                0xB8 => Some(Instruction::CP(ArithmeticSource::B)),
                0xB9 => Some(Instruction::CP(ArithmeticSource::C)),
                0xBA => Some(Instruction::CP(ArithmeticSource::D)),
                0xBB => Some(Instruction::CP(ArithmeticSource::E)),
                0xBC => Some(Instruction::CP(ArithmeticSource::H)),
                0xBD => Some(Instruction::CP(ArithmeticSource::L)),
                0xBE => Some(Instruction::CP(ArithmeticSource::HL_INDIRECT)),
                0xBF => Some(Instruction::CP(ArithmeticSource::A)),
                0xFE => Some(Instruction::CP(ArithmeticSource::D8)),
                0x17 => Some(Instruction::RLA()),
                0xA8 => Some(Instruction::XOR(ArithmeticSource::B)),
                0xA9 => Some(Instruction::XOR(ArithmeticSource::C)),
                0xAA => Some(Instruction::XOR(ArithmeticSource::D)),
                0xAB => Some(Instruction::XOR(ArithmeticSource::E)),
                0xAC => Some(Instruction::XOR(ArithmeticSource::H)),
                0xAD => Some(Instruction::XOR(ArithmeticSource::L)),
                0xAE => Some(Instruction::XOR(ArithmeticSource::HL_INDIRECT)),
                0xAF => Some(Instruction::XOR(ArithmeticSource::A)),
                0xEE => Some(Instruction::XOR(ArithmeticSource::D8)),
                0xA0 => Some(Instruction::AND(ArithmeticSource::B)),
                0xA1 => Some(Instruction::AND(ArithmeticSource::C)),
                0xA2 => Some(Instruction::AND(ArithmeticSource::D)),
                0xA3 => Some(Instruction::AND(ArithmeticSource::E)),
                0xA4 => Some(Instruction::AND(ArithmeticSource::H)),
                0xA5 => Some(Instruction::AND(ArithmeticSource::L)),
                0xA6 => Some(Instruction::AND(ArithmeticSource::HL_INDIRECT)),
                0xA7 => Some(Instruction::AND(ArithmeticSource::A)),
                0xE6 => Some(Instruction::AND(ArithmeticSource::D8)),
                0xB0 => Some(Instruction::OR(ArithmeticSource::B)),
                0xB1 => Some(Instruction::OR(ArithmeticSource::C)),
                0xB2 => Some(Instruction::OR(ArithmeticSource::D)),
                0xB3 => Some(Instruction::OR(ArithmeticSource::E)),
                0xB4 => Some(Instruction::OR(ArithmeticSource::H)),
                0xB5 => Some(Instruction::OR(ArithmeticSource::L)),
                0xB6 => Some(Instruction::OR(ArithmeticSource::HL_INDIRECT)),
                0xB7 => Some(Instruction::OR(ArithmeticSource::A)),
                0xF6 => Some(Instruction::OR(ArithmeticSource::D8)),
                0x2F => Some(Instruction::CPL),
                0x37 => Some(Instruction::SCF),
                0x03 => Some(Instruction::INC(IncrementDecrementTarget::Word(WordRegister::BC))),
                0x13 => Some(Instruction::INC(IncrementDecrementTarget::Word(WordRegister::DE))),
                0x23 => Some(Instruction::INC(IncrementDecrementTarget::Word(WordRegister::HL))),
                0x33 => Some(Instruction::INC(IncrementDecrementTarget::Word(WordRegister::SP))),
                0x04 => Some(Instruction::INC(IncrementDecrementTarget::Byte(ArithmeticSource::B))),
                0x0C => Some(Instruction::INC(IncrementDecrementTarget::Byte(ArithmeticSource::C))),
                0x14 => Some(Instruction::INC(IncrementDecrementTarget::Byte(ArithmeticSource::D))),
                0x1C => Some(Instruction::INC(IncrementDecrementTarget::Byte(ArithmeticSource::E))),
                0x24 => Some(Instruction::INC(IncrementDecrementTarget::Byte(ArithmeticSource::H))),
                0x2C => Some(Instruction::INC(IncrementDecrementTarget::Byte(ArithmeticSource::L))),
                0x34 => Some(Instruction::INC(IncrementDecrementTarget::Byte(ArithmeticSource::HL_INDIRECT))),
                0x3C => Some(Instruction::INC(IncrementDecrementTarget::Byte(ArithmeticSource::A))),
                0x0B => Some(Instruction::DEC(IncrementDecrementTarget::Word(WordRegister::BC))),
                0x1B => Some(Instruction::DEC(IncrementDecrementTarget::Word(WordRegister::DE))),
                0x2B => Some(Instruction::DEC(IncrementDecrementTarget::Word(WordRegister::HL))),
                0x3B => Some(Instruction::DEC(IncrementDecrementTarget::Word(WordRegister::SP))),
                0x05 => Some(Instruction::DEC(IncrementDecrementTarget::Byte(ArithmeticSource::B))),
                0x0D => Some(Instruction::DEC(IncrementDecrementTarget::Byte(ArithmeticSource::C))),
                0x15 => Some(Instruction::DEC(IncrementDecrementTarget::Byte(ArithmeticSource::D))),
                0x1D => Some(Instruction::DEC(IncrementDecrementTarget::Byte(ArithmeticSource::E))),
                0x25 => Some(Instruction::DEC(IncrementDecrementTarget::Byte(ArithmeticSource::H))),
                0x2D => Some(Instruction::DEC(IncrementDecrementTarget::Byte(ArithmeticSource::L))),
                0x35 => Some(Instruction::DEC(IncrementDecrementTarget::Byte(ArithmeticSource::HL_INDIRECT))),
                0x3D => Some(Instruction::DEC(IncrementDecrementTarget::Byte(ArithmeticSource::A))),
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
                0x18 => Some(Instruction::JR(JumpCondition::Always)),
                0x20 => Some(Instruction::JR(JumpCondition::NotZero)),
                0x28 => Some(Instruction::JR(JumpCondition::Zero)),
                0x30 => Some(Instruction::JR(JumpCondition::NoCarry)),
                0x38 => Some(Instruction::JR(JumpCondition::Carry)),
                0xC2 => Some(Instruction::JP(JumpCondition::NotZero, JumpTarget::A16)),
                0xC3 => Some(Instruction::JP(JumpCondition::Always, JumpTarget::A16)),
                0xCA => Some(Instruction::JP(JumpCondition::Zero, JumpTarget::A16)),
                0xD2 => Some(Instruction::JP(JumpCondition::NoCarry, JumpTarget::A16)),
                0xDA => Some(Instruction::JP(JumpCondition::Carry, JumpTarget::A16)),
                0xE9 => Some(Instruction::JP(JumpCondition::Always, JumpTarget::HL_INDIRECT)),
                0xC4 => Some(Instruction::CALL(JumpCondition::NotZero)),
                0xCC => Some(Instruction::CALL(JumpCondition::Zero)),
                0xCD => Some(Instruction::CALL(JumpCondition::Always)),
                0xD4 => Some(Instruction::CALL(JumpCondition::NoCarry)),
                0xDC => Some(Instruction::CALL(JumpCondition::Carry)),
                0xC0 => Some(Instruction::RET(JumpCondition::NotZero)),
                0xC8 => Some(Instruction::RET(JumpCondition::Zero)),
                0xC9 => Some(Instruction::RET(JumpCondition::Always)),
                0xD0 => Some(Instruction::RET(JumpCondition::NoCarry)),
                0xD8 => Some(Instruction::RET(JumpCondition::Carry)),
                0xC7 => Some(Instruction::RST(0x00)),
                0xCF => Some(Instruction::RST(0x08)),
                0xD7 => Some(Instruction::RST(0x10)),
                0xDF => Some(Instruction::RST(0x18)),
                0xE7 => Some(Instruction::RST(0x20)),
                0xEF => Some(Instruction::RST(0x28)),
                0xF7 => Some(Instruction::RST(0x30)),
                0xFF => Some(Instruction::RST(0x38)),
                0xC5 => Some(Instruction::PUSH(WordRegister::BC)),
                0xD5 => Some(Instruction::PUSH(WordRegister::DE)),
                0xE5 => Some(Instruction::PUSH(WordRegister::HL)),
                0xF5 => Some(Instruction::PUSH(WordRegister::AF)),
                0xC1 => Some(Instruction::POP(WordRegister::BC)),
                0xD1 => Some(Instruction::POP(WordRegister::DE)),
                0xE1 => Some(Instruction::POP(WordRegister::HL)),
                0xF1 => Some(Instruction::POP(WordRegister::AF)),
                0xF3 => Some(Instruction::DI),
                0xFB => Some(Instruction::EI),
                0xD9 => Some(Instruction::RETI),
                _ => None,
            }
        }
        else {
            match byte {
                0x10 => Some(Instruction::RL(ArithmeticSource::B)),
                0x11 => Some(Instruction::RL(ArithmeticSource::C)),
                0x12 => Some(Instruction::RL(ArithmeticSource::D)),
                0x13 => Some(Instruction::RL(ArithmeticSource::E)),
                0x14 => Some(Instruction::RL(ArithmeticSource::H)),
                0x15 => Some(Instruction::RL(ArithmeticSource::L)),
                0x16 => Some(Instruction::RL(ArithmeticSource::HL_INDIRECT)),
                0x17 => Some(Instruction::RL(ArithmeticSource::A)),
                0x18 => Some(Instruction::RR(ArithmeticSource::B)),
                0x19 => Some(Instruction::RR(ArithmeticSource::C)),
                0x1A => Some(Instruction::RR(ArithmeticSource::D)),
                0x1B => Some(Instruction::RR(ArithmeticSource::E)),
                0x1C => Some(Instruction::RR(ArithmeticSource::H)),
                0x1D => Some(Instruction::RR(ArithmeticSource::L)),
                0x1E => Some(Instruction::RR(ArithmeticSource::HL_INDIRECT)),
                0x1F => Some(Instruction::RR(ArithmeticSource::A)),
                0x30 => Some(Instruction::SWAP(ArithmeticSource::B)),
                0x31 => Some(Instruction::SWAP(ArithmeticSource::C)),
                0x32 => Some(Instruction::SWAP(ArithmeticSource::D)),
                0x33 => Some(Instruction::SWAP(ArithmeticSource::E)),
                0x34 => Some(Instruction::SWAP(ArithmeticSource::H)),
                0x35 => Some(Instruction::SWAP(ArithmeticSource::L)),
                0x36 => Some(Instruction::SWAP(ArithmeticSource::HL_INDIRECT)),
                0x37 => Some(Instruction::SWAP(ArithmeticSource::A)),
                0x40 => Some(Instruction::BIT(0, ArithmeticSource::B)),
                0x41 => Some(Instruction::BIT(0, ArithmeticSource::C)),
                0x42 => Some(Instruction::BIT(0, ArithmeticSource::D)),
                0x43 => Some(Instruction::BIT(0, ArithmeticSource::E)),
                0x44 => Some(Instruction::BIT(0, ArithmeticSource::H)),
                0x45 => Some(Instruction::BIT(0, ArithmeticSource::L)),
                0x46 => Some(Instruction::BIT(0, ArithmeticSource::HL_INDIRECT)),
                0x47 => Some(Instruction::BIT(0, ArithmeticSource::A)),
                0x48 => Some(Instruction::BIT(1, ArithmeticSource::B)),
                0x49 => Some(Instruction::BIT(1, ArithmeticSource::C)),
                0x4A => Some(Instruction::BIT(1, ArithmeticSource::D)),
                0x4B => Some(Instruction::BIT(1, ArithmeticSource::E)),
                0x4C => Some(Instruction::BIT(1, ArithmeticSource::H)),
                0x4D => Some(Instruction::BIT(1, ArithmeticSource::L)),
                0x4E => Some(Instruction::BIT(1, ArithmeticSource::HL_INDIRECT)),
                0x4F => Some(Instruction::BIT(1, ArithmeticSource::A)),
                0x50 => Some(Instruction::BIT(2, ArithmeticSource::B)),
                0x51 => Some(Instruction::BIT(2, ArithmeticSource::C)),
                0x52 => Some(Instruction::BIT(2, ArithmeticSource::D)),
                0x53 => Some(Instruction::BIT(2, ArithmeticSource::E)),
                0x54 => Some(Instruction::BIT(2, ArithmeticSource::H)),
                0x55 => Some(Instruction::BIT(2, ArithmeticSource::L)),
                0x56 => Some(Instruction::BIT(2, ArithmeticSource::HL_INDIRECT)),
                0x57 => Some(Instruction::BIT(2, ArithmeticSource::A)),
                0x58 => Some(Instruction::BIT(3, ArithmeticSource::B)),
                0x59 => Some(Instruction::BIT(3, ArithmeticSource::C)),
                0x5A => Some(Instruction::BIT(3, ArithmeticSource::D)),
                0x5B => Some(Instruction::BIT(3, ArithmeticSource::E)),
                0x5C => Some(Instruction::BIT(3, ArithmeticSource::H)),
                0x5D => Some(Instruction::BIT(3, ArithmeticSource::L)),
                0x5E => Some(Instruction::BIT(3, ArithmeticSource::HL_INDIRECT)),
                0x5F => Some(Instruction::BIT(3, ArithmeticSource::A)),
                0x60 => Some(Instruction::BIT(4, ArithmeticSource::B)),
                0x61 => Some(Instruction::BIT(4, ArithmeticSource::C)),
                0x62 => Some(Instruction::BIT(4, ArithmeticSource::D)),
                0x63 => Some(Instruction::BIT(4, ArithmeticSource::E)),
                0x64 => Some(Instruction::BIT(4, ArithmeticSource::H)),
                0x65 => Some(Instruction::BIT(4, ArithmeticSource::L)),
                0x66 => Some(Instruction::BIT(4, ArithmeticSource::HL_INDIRECT)),
                0x67 => Some(Instruction::BIT(4, ArithmeticSource::A)),
                0x68 => Some(Instruction::BIT(5, ArithmeticSource::B)),
                0x69 => Some(Instruction::BIT(5, ArithmeticSource::C)),
                0x6A => Some(Instruction::BIT(5, ArithmeticSource::D)),
                0x6B => Some(Instruction::BIT(5, ArithmeticSource::E)),
                0x6C => Some(Instruction::BIT(5, ArithmeticSource::H)),
                0x6D => Some(Instruction::BIT(5, ArithmeticSource::L)),
                0x6E => Some(Instruction::BIT(5, ArithmeticSource::HL_INDIRECT)),
                0x6F => Some(Instruction::BIT(5, ArithmeticSource::A)),
                0x70 => Some(Instruction::BIT(6, ArithmeticSource::B)),
                0x71 => Some(Instruction::BIT(6, ArithmeticSource::C)),
                0x72 => Some(Instruction::BIT(6, ArithmeticSource::D)),
                0x73 => Some(Instruction::BIT(6, ArithmeticSource::E)),
                0x74 => Some(Instruction::BIT(6, ArithmeticSource::H)),
                0x75 => Some(Instruction::BIT(6, ArithmeticSource::L)),
                0x76 => Some(Instruction::BIT(6, ArithmeticSource::HL_INDIRECT)),
                0x77 => Some(Instruction::BIT(6, ArithmeticSource::A)),
                0x78 => Some(Instruction::BIT(7, ArithmeticSource::B)),
                0x79 => Some(Instruction::BIT(7, ArithmeticSource::C)),
                0x7A => Some(Instruction::BIT(7, ArithmeticSource::D)),
                0x7B => Some(Instruction::BIT(7, ArithmeticSource::E)),
                0x7C => Some(Instruction::BIT(7, ArithmeticSource::H)),
                0x7D => Some(Instruction::BIT(7, ArithmeticSource::L)),
                0x7E => Some(Instruction::BIT(7, ArithmeticSource::HL_INDIRECT)),
                0x7F => Some(Instruction::BIT(7, ArithmeticSource::A)),
                _ => None,
            }
        }
    }
}

#[allow(non_camel_case_types)]
pub(super) enum ArithmeticSource {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HL_INDIRECT,
    D8,
}

impl ArithmeticSource {
    pub(super) fn get_byte_and_pc_offset(&self, cpu: &CPU) -> (u8, u16) {
        match self {
            ArithmeticSource::A => (cpu.registers.a, 1),
            ArithmeticSource::B => (cpu.registers.b, 1),
            ArithmeticSource::C => (cpu.registers.c, 1),
            ArithmeticSource::D => (cpu.registers.d, 1),
            ArithmeticSource::E => (cpu.registers.e, 1),
            ArithmeticSource::H => (cpu.registers.h, 1),
            ArithmeticSource::L => (cpu.registers.l, 1),
            ArithmeticSource::HL_INDIRECT => (cpu.bus.read_byte(cpu.registers.get_hl()), 1),
            ArithmeticSource::D8 => (cpu.read_next_byte(), 2),
        }
    }

    pub(super) fn set_byte(&self, value: u8, cpu: &mut CPU) {
        match self {
            ArithmeticSource::A => cpu.registers.a = value,
            ArithmeticSource::B => cpu.registers.b = value,
            ArithmeticSource::C => cpu.registers.c = value,
            ArithmeticSource::D => cpu.registers.d = value,
            ArithmeticSource::E => cpu.registers.e = value,
            ArithmeticSource::H => cpu.registers.h = value,
            ArithmeticSource::L => cpu.registers.l = value,
            ArithmeticSource::HL_INDIRECT => {
                cpu.bus.write_byte(value, cpu.registers.get_hl());
            }
            ArithmeticSource::D8 => panic!("Trying to set the byte for a literal d8!"),
        };
    }
}

pub(super) enum RotateDirection {
    Left,
    Right,
}

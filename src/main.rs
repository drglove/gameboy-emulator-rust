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

#[allow(non_camel_case_types)]
enum Instruction {
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
    INC(IncrementDecrementTarget),
    DEC(IncrementDecrementTarget),
    RL(ArithmeticSource),
    RLA(),
    RR(ArithmeticSource),
    SCF,
    CPL,
    BIT(u8, ArithmeticSource),
    DI,
    EI,
    RETI,
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

enum JumpCondition {
    Always,
    Zero,
    NotZero,
    Carry,
    NoCarry,
}

impl JumpCondition {
    fn take_jump(&self, registers: &Registers) -> bool {
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
enum JumpTarget {
    A16,
    HL_INDIRECT,
}

enum IncrementDecrementTarget {
    Byte(ArithmeticSource),
    Word(WordRegister),
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
    AF,
}

impl WordRegister {
    fn get_word(&self, registers: &Registers) -> u16 {
        match self {
            WordRegister::BC => registers.get_bc(),
            WordRegister::DE => registers.get_de(),
            WordRegister::HL => registers.get_hl(),
            WordRegister::AF => registers.get_af(),
            WordRegister::SP => registers.sp,
        }
    }

    fn set_word(&self, value: u16, registers: &mut Registers) {
        match self {
            WordRegister::BC => registers.set_bc(value),
            WordRegister::DE => registers.set_de(value),
            WordRegister::HL => registers.set_hl(value),
            WordRegister::AF => registers.set_af(value),
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
enum ArithmeticSource {
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
    fn get_byte_and_pc_offset(&self, cpu: &CPU) -> (u8, u16) {
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

    fn set_byte(&self, value: u8, cpu: &mut CPU) {
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

    fn get_af(&self) -> u16 {
        ((self.a as u16) << 8) | (u8::from(self.f) as u16)
    }

    fn set_af(&mut self, value: u16) {
        self.a = ((value & 0xFF00) >> 8) as u8;
        self.f = FlagsRegister::from((value & 0x00FF) as u8);
    }
}

#[derive(Copy, Clone)]
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

enum RotateDirection {
    Left,
    Right,
}

struct DMG01 {
    cpu: CPU,
}

impl DMG01 {
    fn new(cart: Option<Cartridge>) -> DMG01 {
        use std::cmp::min;

        let mut memory: [u8; 0x10000] = [0; 0x10000];
        let size_to_copy = min(0x10000, cart.as_ref().unwrap().rom.len());
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
                bus: MemoryBus {
                    memory,
                    boot_rom: [
                        0x31, 0xFE, 0xFF, 0xAF, 0x21, 0xFF, 0x9F, 0x32, 0xCB, 0x7C, 0x20, 0xFB,
                        0x21, 0x26, 0xFF, 0x0E, 0x11, 0x3E, 0x80, 0x32, 0xE2, 0x0C, 0x3E, 0xF3,
                        0xE2, 0x32, 0x3E, 0x77, 0x77, 0x3E, 0xFC, 0xE0, 0x47, 0x11, 0x04, 0x01,
                        0x21, 0x10, 0x80, 0x1A, 0xCD, 0x95, 0x00, 0xCD, 0x96, 0x00, 0x13, 0x7B,
                        0xFE, 0x34, 0x20, 0xF3, 0x11, 0xD8, 0x00, 0x06, 0x08, 0x1A, 0x13, 0x22,
                        0x23, 0x05, 0x20, 0xF9, 0x3E, 0x19, 0xEA, 0x10, 0x99, 0x21, 0x2F, 0x99,
                        0x0E, 0x0C, 0x3D, 0x28, 0x08, 0x32, 0x0D, 0x20, 0xF9, 0x2E, 0x0F, 0x18,
                        0xF3, 0x67, 0x3E, 0x64, 0x57, 0xE0, 0x42, 0x3E, 0x91, 0xE0, 0x40, 0x04,
                        0x1E, 0x02, 0x0E, 0x0C, 0xF0, 0x44, 0xFE, 0x90, 0x20, 0xFA, 0x0D, 0x20,
                        0xF7, 0x1D, 0x20, 0xF2, 0x0E, 0x13, 0x24, 0x7C, 0x1E, 0x83, 0xFE, 0x62,
                        0x28, 0x06, 0x1E, 0xC1, 0xFE, 0x64, 0x20, 0x06, 0x7B, 0xE2, 0x0C, 0x3E,
                        0x87, 0xE2, 0xF0, 0x42, 0x90, 0xE0, 0x42, 0x15, 0x20, 0xD2, 0x05, 0x20,
                        0x4F, 0x16, 0x20, 0x18, 0xCB, 0x4F, 0x06, 0x04, 0xC5, 0xCB, 0x11, 0x17,
                        0xC1, 0xCB, 0x11, 0x17, 0x05, 0x20, 0xF5, 0x22, 0x23, 0x22, 0x23, 0xC9,
                        0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83,
                        0x00, 0x0C, 0x00, 0x0D, 0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E,
                        0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99, 0xBB, 0xBB, 0x67, 0x63,
                        0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E,
                        0x3C, 0x42, 0xB9, 0xA5, 0xB9, 0xA5, 0x42, 0x3C, 0x21, 0x04, 0x01, 0x11,
                        0xA8, 0x00, 0x1A, 0x13, 0xBE, 0x20, 0xFE, 0x23, 0x7D, 0xFE, 0x34, 0x20,
                        0xF5, 0x06, 0x19, 0x78, 0x86, 0x23, 0x05, 0x20, 0xFB, 0x86, 0x20, 0xFE,
                        0x3E, 0x01, 0xE0, 0x50,
                    ],
                    finished_boot: false,
                    ppu: PPU {
                        vram: [0; VRAM_SIZE],
                        tile_set: [Tile::empty_tile(); 384],
                        mode: PPUMode::HBlank,
                        cycles: 0,
                        line: 0,
                        framebuffer: vec![0; 256 * 256],
                    },
                },
                interrupt_master_enable: true,
            },
        }
    }
}

struct CPU {
    registers: Registers,
    bus: MemoryBus,
    interrupt_master_enable: bool,
}

struct MemoryBus {
    memory: [u8; 0x10000],
    boot_rom: [u8; BOOTROM_SIZE],
    finished_boot: bool,
    ppu: PPU,
}

impl MemoryBus {
    fn read_byte(&self, address: u16) -> u8 {
        let address = address as usize;
        match address {
            BOOTROM_BEGIN..=BOOTROM_END if !self.finished_boot => self.boot_rom[address],
            VRAM_BEGIN..=VRAM_END => self.ppu.read_vram(address - VRAM_BEGIN),
            0xFF44 => self.ppu.line,
            _ => self.memory[address],
        }
    }

    fn read_byte_from_offset(&self, address_offset: u8) -> u8 {
        self.read_byte(address_offset as u16 + 0xFF00)
    }

    fn write_byte(&mut self, value: u8, address: u16) {
        let address = address as usize;
        match address {
            BOOTROM_BEGIN..=BOOTROM_END if !self.finished_boot => panic!("Cannot write into bootrom territory!"),
            VRAM_BEGIN..=VRAM_END => self.ppu.write_vram(value, address - VRAM_BEGIN),
            0xFF50 if !self.finished_boot => self.finished_boot = true,
            _ => self.memory[address] = value,
        }
    }

    fn write_byte_to_offset(&mut self, value: u8, address_offset: u8) {
        self.write_byte(value, address_offset as u16 + 0xFF00)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
enum Interrupt {
    VBlank,
    LCDStat,
    Timer,
    Serial,
    Joypad,
}

impl Interrupt {
    const INTERRUPT_ENABLE_ADDRESS: u16 = 0xFFFF;
    const INTERRUPT_FLAG_ADDRESS: u16 = 0xFF0F;

    fn get_interrupts_to_process(bus: &MemoryBus) -> Vec<Self> {
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

    fn is_interrupt_enabled(&self, bus: &MemoryBus) -> bool {
        let interrupt_enabled_byte = bus.read_byte(Interrupt::INTERRUPT_ENABLE_ADDRESS);
        let mask: u8 = self.interrupt_byte_mask();
        (interrupt_enabled_byte & mask) == mask
    }

    fn is_interrupt_flag_set(&self, bus: &MemoryBus) -> bool {
        let interrupt_flag_byte = bus.read_byte(Interrupt::INTERRUPT_FLAG_ADDRESS);
        let mask = self.interrupt_byte_mask();
        (interrupt_flag_byte & mask) == mask
    }

    fn set_interrupt_flag(&self, bus: &mut MemoryBus) {
        self.set_interrupt_flag_to_value(true, bus);
    }

    fn clear_interrupt_flag(&self, bus: &mut MemoryBus) {
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

const BOOTROM_BEGIN: usize = 0x0000;
const BOOTROM_END: usize = 0x00FF;
const BOOTROM_SIZE: usize = BOOTROM_END - BOOTROM_BEGIN + 1;
const VRAM_BEGIN: usize = 0x8000;
const VRAM_END: usize = 0x9FFF;
const VRAM_SIZE: usize = VRAM_END - VRAM_BEGIN + 1;
const BG_MAP_START: usize = 0x9800;
const BG_MAP_END: usize = 0x9BFF;

struct PPU {
    vram: [u8; VRAM_SIZE],
    tile_set: [Tile; 384],
    mode: PPUMode,
    cycles: u16,
    line: u8,
    framebuffer: Vec<u32>,
}

enum PPUMode {
    HBlank,
    VBlank,
    OAMAccess,
    VRAMAccess,
}

impl PPU {
    fn read_vram(&self, address: usize) -> u8 {
        self.vram[address]
    }

    fn write_vram(&mut self, value: u8, address: usize) {
        self.vram[address] = value;

        // Addresses outside this range are not in the tile set.
        if address >= 0x1800 {
            return;
        }

        // Determine the even address corresponding to this address.
        let even_address = address & 0xFFFE;
        let byte1 = self.vram[even_address];
        let byte2 = self.vram[even_address + 1];

        // Each row is 16 bytes, and every 2 bytes is a new row.
        let tile_index = address / 16;
        let row_index = (address % 16) / 2;

        for pixel_index in 0..8 {
            let mask = (1 << (7 - pixel_index)) as u8;
            let lsb = byte1 & mask;
            let msb = byte2 & mask;
            let pixel_value = match (msb != 0, lsb != 0) {
                (false, false) => TilePixelValue::Zero,
                (false, true) => TilePixelValue::One,
                (true, false) => TilePixelValue::Two,
                (true, true) => TilePixelValue::Three,
            };
            self.tile_set[tile_index].pixels[row_index][pixel_index] = pixel_value;
        }
    }

    fn step(&mut self, cycles: u8) -> HashSet<Interrupt> {
        self.cycles += cycles as u16;

        let mut interrupts = HashSet::new();
        match self.mode {
            PPUMode::HBlank => {
                if self.cycles >= 200 {
                    self.cycles = self.cycles % 200;
                    self.line += 1;

                    if self.line >= 144 {
                        self.mode = PPUMode::VBlank;
                        interrupts.insert(Interrupt::VBlank);
                    } else {
                        self.mode = PPUMode::OAMAccess;
                    }
                }
            }
            PPUMode::VBlank => {
                if self.cycles >= 456 {
                    self.cycles = self.cycles % 456;
                    self.line += 1;

                    if self.line == 154 {
                        self.mode = PPUMode::OAMAccess;
                        self.line = 0;
                    }
                }
            }
            PPUMode::OAMAccess => {
                if self.cycles >= 80 {
                    self.cycles = self.cycles % 80;
                    self.mode = PPUMode::VRAMAccess;
                }
            }
            PPUMode::VRAMAccess => {
                if self.cycles >= 172 {
                    self.cycles = self.cycles % 172;
                    self.mode = PPUMode::HBlank;
                    self.render_line();
                }
            }
        }
        interrupts
    }

    fn render_line(&mut self) {
        const BG_OFFSET: usize = BG_MAP_START - VRAM_BEGIN;
        const PIXEL_DIMENSION_PER_TILE: usize = 8;
        const TILES_PER_ROW: usize = 0x20;

        let line = self.line.wrapping_add(0);
        let tile_row = (line as usize) / PIXEL_DIMENSION_PER_TILE;

        for pixel_column in 0..=255 {
            let tile_column = (pixel_column as usize) / PIXEL_DIMENSION_PER_TILE;
            let tile_address = BG_OFFSET + tile_row * TILES_PER_ROW + tile_column;
            let tile_byte = self.vram[tile_address];
            let pixel_index = (self.line as usize) * 256 + pixel_column;
            self.framebuffer[pixel_index] = self.get_pixel_colour_from_tile(tile_byte, line % 8, (pixel_column % 8) as u8);
        }
    }

    fn get_pixel_colour_from_tile(&self, tile: u8, row: u8, col: u8) -> u32 {
        let tile = self.tile_set[tile as usize];
        let tile_pixel= tile.pixels[row as usize][col as usize];
        PPU::color(tile_pixel)
    }

    fn render_entire_framebuffer(&self) -> Vec<u32> {
        let mut framebuffer = vec![0; 256 * 256];
        let mut current_pixel = 0;
        for tile_row in 0..=31 {
            for pixel_row in 0..=7 {
                for tile_column in 0..=31 {
                    let bg_start = BG_MAP_START - VRAM_BEGIN;
                    let tiles_per_row = 0x20;
                    let tile_address = bg_start + tile_row * tiles_per_row + tile_column;
                    let tile_byte = self.vram[tile_address];
                    for pixel_column in 0..=7 {
                        framebuffer[current_pixel] = self.get_pixel_colour_from_tile(tile_byte, pixel_row, pixel_column);
                        current_pixel += 1;
                    }
                }
            }
        }
        framebuffer
    }

    fn color(tile_pixel: TilePixelValue) -> u32 {
        match tile_pixel {
            TilePixelValue::Zero => 0xFFFFFF,
            TilePixelValue::One => 0xAAAAAA,
            TilePixelValue::Two => 0x555555,
            TilePixelValue::Three => 0x000000,
        }
    }
}

fn dump_bytes(bytes: &[u8], filename: &str) {
    std::fs::write(filename, bytes).unwrap();
}

#[derive(Copy, Clone)]
enum TilePixelValue {
    Zero,
    One,
    Two,
    Three,
}

#[derive(Copy, Clone)]
struct Tile {
    pixels: [[TilePixelValue; 8]; 8],
}

impl Tile {
    fn empty_tile() -> Tile {
        Tile {
            pixels: [[TilePixelValue::Zero; 8]; 8],
        }
    }
}

impl CPU {
    fn step_frame(&mut self) {
        const TARGET_FRAMERATE_HZ: u32 = 60;
        const CPU_CLOCK_RATE_HZ: u32 = 4194304;
        const CYCLES_PER_FRAME: u32 = CPU_CLOCK_RATE_HZ / TARGET_FRAMERATE_HZ;

        let mut cycles_executed = 0;
        while cycles_executed < CYCLES_PER_FRAME {
            let cycles_this_instruction = self.step_instruction();
            cycles_executed += cycles_this_instruction as u32;
        }
    }

    fn step_instruction(&mut self) -> u8 {
        let instruction = self.next_instruction().unwrap();
        let (next_pc, cycles) = self.execute(instruction);
        self.registers.pc = next_pc;

        let interrupts_to_flag = self.bus.ppu.step(4);
        for interrupt in interrupts_to_flag {
            if interrupt.is_interrupt_enabled(&self.bus) {
                interrupt.set_interrupt_flag(&mut self.bus);
            }
        }

        if self.interrupt_master_enable {
            let interrupts_to_process = Interrupt::get_interrupts_to_process(&self.bus);
            for interrupt in interrupts_to_process {
                interrupt.clear_interrupt_flag(&mut self.bus);
                self.interrupt(interrupt);
            }
        }

        cycles
    }

    fn next_instruction(&self) -> Result<Instruction, String> {
        let mut instruction_byte = self.bus.read_byte(self.registers.pc);
        let prefix_instruction = instruction_byte == 0xCB;
        if prefix_instruction {
            instruction_byte = self.bus.read_byte(self.registers.pc.wrapping_add(1));
        }

        Instruction::from_byte(instruction_byte, prefix_instruction).ok_or(format!(
            "Unknown instruction found for {:#04x} (prefixed = {})",
            instruction_byte, prefix_instruction,
        ))
    }

    fn execute(&mut self, instruction: Instruction) -> (u16, u8) {
        match instruction {
            Instruction::NOP => (self.registers.pc.wrapping_add(1), 4),
            Instruction::ADD(source) => {
                let (value, pc_offset) = source.get_byte_and_pc_offset(&self);
                let new_value = self.add(value);
                self.registers.a = new_value;
                let cycles = match source {
                    ArithmeticSource::HL_INDIRECT => 8,
                    ArithmeticSource::D8 => 8,
                    _ => 4,
                };
                (self.registers.pc.wrapping_add(pc_offset), cycles)
            }
            Instruction::ADD_HL(source) => {
                let value = source.get_word(&self.registers);
                let new_value = self.add_hl(value);
                self.registers.set_hl(new_value);
                (self.registers.pc.wrapping_add(1), 8)
            }
            Instruction::ADD_SP() => {
                let value = self.read_next_byte() as i8;
                let padded_value = value as i16 as u16; // Extend to 16 bits and drop the signed-ness
                let sp = self.registers.sp;
                let new_sp = sp.wrapping_add(padded_value);

                self.registers.f.zero = false;
                self.registers.f.subtract = false;
                self.registers.f.carry = (sp & 0x00FF) + (padded_value & 0x00FF) > 0x00FF;
                self.registers.f.half_carry = (sp & 0x000F) + (padded_value & 0x000F) > 0x000F;
                self.registers.sp = new_sp;

                (self.registers.pc.wrapping_add(2), 16)
            }
            Instruction::SUB(source) => {
                let (value, pc_offset) = source.get_byte_and_pc_offset(&self);
                let new_value = self.subtract(value);
                self.registers.a = new_value;
                let cycles = match source {
                    ArithmeticSource::HL_INDIRECT => 8,
                    ArithmeticSource::D8 => 8,
                    _ => 4,
                };
                (self.registers.pc.wrapping_add(pc_offset), cycles)
            }
            Instruction::CP(source) => {
                let (value, pc_offset) = source.get_byte_and_pc_offset(&self);
                self.compare(value);
                let cycles = match source {
                    ArithmeticSource::HL_INDIRECT => 8,
                    ArithmeticSource::D8 => 8,
                    _ => 4,
                };
                (self.registers.pc.wrapping_add(pc_offset), cycles)
            }
            Instruction::XOR(source) => {
                let (value, pc_offset) = source.get_byte_and_pc_offset(&self);
                let new_value = self.xor(value);
                self.registers.a = new_value;
                let cycles = match source {
                    ArithmeticSource::HL_INDIRECT => 8,
                    ArithmeticSource::D8 => 8,
                    _ => 4,
                };
                (self.registers.pc.wrapping_add(pc_offset), cycles)
            }
            Instruction::AND(source) => {
                let (value, pc_offset) = source.get_byte_and_pc_offset(&self);
                let new_value = self.and(value);
                self.registers.a = new_value;
                let cycles = match source {
                    ArithmeticSource::HL_INDIRECT => 8,
                    ArithmeticSource::D8 => 8,
                    _ => 4,
                };
                (self.registers.pc.wrapping_add(pc_offset), cycles)
            }
            Instruction::OR(source) => {
                let (value, pc_offset) = source.get_byte_and_pc_offset(&self);
                let new_value = self.or(value);
                self.registers.a = new_value;
                let cycles = match source {
                    ArithmeticSource::HL_INDIRECT => 8,
                    ArithmeticSource::D8 => 8,
                    _ => 4,
                };
                (self.registers.pc.wrapping_add(pc_offset), cycles)
            }
            Instruction::SCF => {
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = true;
                (self.registers.pc.wrapping_add(1), 4)
            }
            Instruction::CPL => {
                self.registers.a = self.cpl();
                (self.registers.pc.wrapping_add(1), 4)
            }
            Instruction::LD(load_type) => {
                return match load_type {
                    LoadType::ReadWordNumericLiteral(target, _) => {
                        let value = self.read_next_word();
                        target.set_word(value, &mut self.registers);
                        (self.registers.pc.wrapping_add(3), 12)
                    }
                    LoadType::ReadByteNumericLiteral(target, _) => {
                        let value = self.read_next_byte();
                        target.set_byte(value, &mut self.registers);
                        (self.registers.pc.wrapping_add(2), 8)
                    }
                    LoadType::ReadByteFromAddressOffset(target, source) => {
                        let address_offset = source.get_address_offset(&self.registers);
                        let value = self.bus.read_byte_from_offset(address_offset);
                        target.set_byte(value, &mut self.registers);
                        (self.registers.pc.wrapping_add(1), 8)
                    }
                    LoadType::ReadByteFromAddressLiteral(target, _) => {
                        let address = self.read_next_word();
                        let value = self.bus.read_byte(address);
                        target.set_byte(value, &mut self.registers);
                        (self.registers.pc.wrapping_add(3), 16)
                    }
                    LoadType::ReadByteFromAddressOffsetLiteral(target, _) => {
                        let address_offset = self.read_next_byte();
                        let value = self.bus.read_byte_from_offset(address_offset);
                        target.set_byte(value, &mut self.registers);
                        (self.registers.pc.wrapping_add(2), 12)
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
                        (self.registers.pc.wrapping_add(1), 8)
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
                        (self.registers.pc.wrapping_add(1), 8)
                    }
                    LoadType::WriteByteFromRegisterToAddressOffsetLiteral(_, source) => {
                        let address_offset = self.read_next_byte();
                        let value = source.get_byte(&self.registers);
                        self.bus.write_byte_to_offset(value, address_offset);
                        (self.registers.pc.wrapping_add(2), 12)
                    }
                    LoadType::WriteByteFromRegisterToAddressLiteral(_, source) => {
                        let address = self.read_next_word();
                        let value = source.get_byte(&self.registers);
                        self.bus.write_byte(value, address);
                        (self.registers.pc.wrapping_add(3), 16)
                    }
                    LoadType::WriteByteFromRegisterToAddressOffsetRegister(target, source) => {
                        let address_offset = target.get_address_offset(&self.registers);
                        let value = source.get_byte(&self.registers);
                        self.bus.write_byte_to_offset(value, address_offset);
                        (self.registers.pc.wrapping_add(1), 8)
                    }
                    LoadType::WriteByteLiteralToAddressContainedInRegister(target, _) => {
                        let address = target.get_address(&self.registers);
                        let value = self.read_next_byte();
                        self.bus.write_byte(value, address);
                        (self.registers.pc.wrapping_add(2), 12)
                    }
                    LoadType::WriteWordInRegisterToAddressContainedInLiteral(_, source) => {
                        let address = self.read_next_word();
                        let value = source.get_word(&self.registers);
                        self.bus.write_byte((value & 0x00FF) as u8, address);
                        self.bus
                            .write_byte(((value & 0xFF00) >> 8) as u8, address.wrapping_add(1));
                        (self.registers.pc.wrapping_add(3), 20)
                    }
                    LoadType::CopyByteFromRegisterToRegister(target, source) => {
                        let value = source.get_byte(&self.registers);
                        target.set_byte(value, &mut self.registers);
                        (self.registers.pc.wrapping_add(1), 4)
                    }
                    LoadType::CopyWordFromRegisterToRegister(target, source) => {
                        let value = source.get_word(&self.registers);
                        target.set_word(value, &mut self.registers);
                        (self.registers.pc.wrapping_add(1), 8)
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
                        (self.registers.pc.wrapping_add(2), 12)
                    }
                }
            }
            Instruction::JR(jump_condition) => {
                let take_jump = jump_condition.take_jump(&self.registers);
                let cycles = if take_jump { 12 } else { 8 };
                let next_pc = self.jump_relative(take_jump);
                (next_pc, cycles)
            }
            Instruction::JP(jump_condition, jump_target) => {
                let take_jump = jump_condition.take_jump(&self.registers);
                let cycles = match jump_target {
                    JumpTarget::A16 => {
                        if take_jump {
                            16
                        } else {
                            12
                        }
                    }
                    JumpTarget::HL_INDIRECT => 4,
                };
                let next_pc = self.jump(take_jump, jump_target);
                (next_pc, cycles)
            }
            Instruction::CALL(jump_condition) => {
                let take_jump = jump_condition.take_jump(&self.registers);
                let next_pc = self.call(take_jump);
                let cycles = if take_jump { 24 } else { 12 };
                (next_pc, cycles)
            }
            Instruction::RET(jump_condition) => {
                let take_jump = jump_condition.take_jump(&self.registers);
                let next_pc = self.ret(take_jump);
                let cycles = match jump_condition {
                    JumpCondition::Always => 16,
                    _ => {
                        if take_jump {
                            20
                        } else {
                            8
                        }
                    }
                };
                (next_pc, cycles)
            }
            Instruction::PUSH(source) => {
                let value = source.get_word(&self.registers);
                self.push(value);
                (self.registers.pc.wrapping_add(1), 16)
            }
            Instruction::POP(target) => {
                let value = self.pop();
                target.set_word(value, &mut self.registers);
                (self.registers.pc.wrapping_add(1), 12)
            }
            Instruction::INC(target) => match target {
                IncrementDecrementTarget::Byte(byte_target) => {
                    let (value, pc_offset) = byte_target.get_byte_and_pc_offset(&self);
                    let new_value = self.increment(value);
                    byte_target.set_byte(new_value, self);
                    let cycles = match byte_target {
                        ArithmeticSource::HL_INDIRECT => 12,
                        _ => 4,
                    };
                    (self.registers.pc.wrapping_add(pc_offset), cycles)
                }
                IncrementDecrementTarget::Word(word_register) => {
                    let value = word_register.get_word(&self.registers);
                    let new_value = self.increment_word(value);
                    word_register.set_word(new_value, &mut self.registers);
                    (self.registers.pc.wrapping_add(1), 8)
                }
            },
            Instruction::DEC(target) => match target {
                IncrementDecrementTarget::Byte(byte_target) => {
                    let (value, pc_offset) = byte_target.get_byte_and_pc_offset(&self);
                    let new_value = self.decrement(value);
                    byte_target.set_byte(new_value, self);
                    let cycles = match byte_target {
                        ArithmeticSource::HL_INDIRECT => 12,
                        _ => 4,
                    };
                    (self.registers.pc.wrapping_add(pc_offset), cycles)
                }
                IncrementDecrementTarget::Word(word_register) => {
                    let value = word_register.get_word(&self.registers);
                    let new_value = self.decrement_word(value);
                    word_register.set_word(new_value, &mut self.registers);
                    (self.registers.pc.wrapping_add(1), 8)
                }
            },
            Instruction::RL(source) => {
                let (value, pc_offset) = source.get_byte_and_pc_offset(&self);
                let new_value = self.rotate_through_carry(value, RotateDirection::Left, true);
                source.set_byte(new_value, self);
                let cycles = match source {
                    ArithmeticSource::HL_INDIRECT => 16,
                    _ => 8,
                };
                (self.registers.pc.wrapping_add(pc_offset + 1), cycles)
            }
            Instruction::RLA() => {
                let source = ArithmeticSource::A;
                let (value, pc_offset) = source.get_byte_and_pc_offset(&self);
                let new_value = self.rotate_through_carry(value, RotateDirection::Left, false);
                source.set_byte(new_value, self);
                (self.registers.pc.wrapping_add(pc_offset), 4)
            }
            Instruction::RR(source) => {
                let (value, pc_offset) = source.get_byte_and_pc_offset(&self);
                let new_value = self.rotate_through_carry(value, RotateDirection::Right, true);
                source.set_byte(new_value, self);
                let cycles = match source {
                    ArithmeticSource::HL_INDIRECT => 16,
                    _ => 8,
                };
                (self.registers.pc.wrapping_add(pc_offset + 1), cycles)
            }
            Instruction::BIT(bit_to_test, source) => {
                let (value, pc_offset) = source.get_byte_and_pc_offset(&self);
                self.bit_test(value, bit_to_test);
                let cycles = match source {
                    ArithmeticSource::HL_INDIRECT => 16,
                    _ => 8,
                };
                (self.registers.pc.wrapping_add(pc_offset + 1), cycles)
            }
            Instruction::DI => {
                self.interrupt_master_enable = false;
                (self.registers.pc.wrapping_add(1), 4)
            }
            Instruction::EI => {
                self.interrupt_master_enable = true;
                (self.registers.pc.wrapping_add(1), 4)
            }
            Instruction::RETI => {
                self.interrupt_master_enable = true;
                (self.ret(true), 16)
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

    fn add_hl(&mut self, value: u16) -> u16 {
        let hl = self.registers.get_hl();
        let (new_value, did_overflow) = hl.overflowing_add(value);
        self.registers.f.subtract = false;
        self.registers.f.carry = did_overflow;
        // Half-carry is true if the high bytes overflowed when added. This is when the 11th bit flips.
        let mask = 0b111_1111_1111;
        self.registers.f.half_carry = (value & mask) + (hl & mask) > mask;
        new_value
    }

    fn subtract(&mut self, value: u8) -> u8 {
        let (new_value, did_overflow) = self.registers.a.overflowing_sub(value);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = true;
        self.registers.f.carry = did_overflow;
        self.registers.f.half_carry = (self.registers.a & 0x0F) > (value & 0x0F);
        new_value
    }

    fn compare(&mut self, value: u8) {
        let a_value = self.registers.a;
        self.registers.f.zero = a_value == value;
        self.registers.f.subtract = true;
        self.registers.f.half_carry = (a_value & 0x0F) < (value & 0x0F);
        self.registers.f.carry = a_value < value;
    }

    fn cpl(&mut self) -> u8 {
        let new_value = self.registers.a.bitxor(0xFF);
        self.registers.f.subtract = true;
        self.registers.f.half_carry = true;
        new_value
    }

    fn increment(&mut self, value: u8) -> u8 {
        let new_value = value.wrapping_add(1);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = (value & 0x0F) + (1) > 0x0F;
        new_value
    }

    fn increment_word(&mut self, value: u16) -> u16 {
        value.wrapping_add(1)
    }

    fn decrement(&mut self, value: u8) -> u8 {
        let new_value = value.wrapping_sub(1);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = true;
        // There's a carry if we subtract one from 0 in the bottom nibble.
        self.registers.f.half_carry = (value & 0x0F) == 0x00;
        new_value
    }

    fn decrement_word(&mut self, value: u16) -> u16 {
        value.wrapping_sub(1)
    }

    fn xor(&mut self, value: u8) -> u8 {
        let new_value = self.registers.a.bitxor(value);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = false;
        self.registers.f.half_carry = false;
        new_value
    }

    fn and(&mut self, value: u8) -> u8 {
        let new_value = self.registers.a.bitand(value);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = true;
        self.registers.f.half_carry = false;
        new_value
    }

    fn or(&mut self, value: u8) -> u8 {
        let new_value = self.registers.a.bitor(value);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = false;
        self.registers.f.half_carry = false;
        new_value
    }

    fn jump_relative(&self, take_jump: bool) -> u16 {
        let next_pc = self.registers.pc.wrapping_add(2);
        if take_jump {
            let offset = self.read_next_byte() as i8;
            match offset.is_positive() {
                true => next_pc.wrapping_add(offset as u16),
                false => next_pc.wrapping_sub(offset.abs() as u16),
            }
        } else {
            next_pc
        }
    }

    fn jump(&self, take_jump: bool, jump_target: JumpTarget) -> u16 {
        let (address_if_taken, address_if_not_taken) = match jump_target {
            JumpTarget::A16 => (self.read_next_word(), self.registers.pc.wrapping_add(3)),
            JumpTarget::HL_INDIRECT => (self.registers.get_hl(), self.registers.pc.wrapping_add(1)),
        };
        if take_jump {
            address_if_taken
        } else {
            address_if_not_taken
        }
    }

    fn call(&mut self, take_jump: bool) -> u16 {
        let address_if_taken = self.read_next_word();
        let address_to_return_to = self.registers.pc.wrapping_add(3);
        if take_jump {
            self.push(address_to_return_to);
            address_if_taken
        } else {
            address_to_return_to
        }
    }

    fn ret(&mut self, take_jump: bool) -> u16 {
        if take_jump {
            self.pop()
        } else {
            self.registers.pc.wrapping_add(1)
        }
    }

    fn rotate_through_carry(
        &mut self,
        value: u8,
        direction: RotateDirection,
        set_zero: bool,
    ) -> u8 {
        let rotated_value = match direction {
            RotateDirection::Left => value << 1,
            RotateDirection::Right => value >> 1,
        };
        let carry_bit = if self.registers.f.carry { 1 } else { 0 };
        let new_value = rotated_value | carry_bit;
        self.registers.f.zero = set_zero && new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = (value & 0x80) == 0x80;
        new_value
    }

    fn bit_test(&mut self, value: u8, bit_to_test: u8) {
        let mask = (1 << bit_to_test) as u8;
        self.registers.f.zero = (mask & value) == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = true;
    }

    fn push(&mut self, value: u16) {
        self.registers.sp = self.registers.sp.wrapping_sub(1);
        self.bus
            .write_byte(((value & 0xFF00) >> 8) as u8, self.registers.sp);

        self.registers.sp = self.registers.sp.wrapping_sub(1);
        self.bus
            .write_byte((value & 0x00FF) as u8, self.registers.sp);
    }

    fn pop(&mut self) -> u16 {
        let low = self.bus.read_byte(self.registers.sp);
        self.registers.sp = self.registers.sp.wrapping_add(1);

        let hi = self.bus.read_byte(self.registers.sp);
        self.registers.sp = self.registers.sp.wrapping_add(1);

        ((hi as u16) << 8) | (low as u16)
    }

    fn interrupt(&mut self, interrupt: Interrupt) {
        self.push(self.registers.pc);
        self.registers.pc = match interrupt {
            Interrupt::VBlank => 0x40,
            Interrupt::LCDStat => 0x48,
            Interrupt::Timer => 0x50,
            Interrupt::Serial => 0x58,
            Interrupt::Joypad => 0x60,
        };
    }
}

struct Cartridge {
    rom: Vec<u8>,
}

use std::collections::HashSet;
use std::ops::{BitAnd, BitOr, BitXor, Not};
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

    use minifb::{Window, WindowOptions};
    let mut window = match Window::new("DMG-01", 256, 256, WindowOptions::default()) {
        Ok(win) => win,
        Err(_) => panic!("Could not create window!"),
    };
    window.limit_update_rate(Some(std::time::Duration::from_millis(16)));

    let mut gameboy = DMG01::new(cart);
    while window.is_open() {
        gameboy.cpu.step_frame();

        window
            .update_with_buffer(gameboy.cpu.bus.ppu.framebuffer.as_slice(), 256, 256)
            .unwrap();
    }
}

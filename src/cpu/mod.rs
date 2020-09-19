mod instructions;
pub mod interrupts;
mod registers;

use self::instructions::{
    AddressContainingRegister, ArithmeticSource, IncrementDecrementTarget, Instruction,
    JumpCondition, JumpTarget, LoadType, RestartTarget, RotateDirection,
};
use super::memory::cartridge::Cartridge;
use super::memory::MemoryBus;
use interrupts::Interrupt;
use registers::Registers;
use std::ops::{BitAnd, BitOr, BitXor, Not};

pub struct CPU {
    registers: Registers,
    pub bus: MemoryBus,
    interrupt_master_enable: bool,
    halted: bool,
}

pub const CPU_CLOCK_RATE_HZ: u32 = 4194304;

impl CPU {
    pub fn new(cart: Option<Cartridge>) -> Self {
        CPU {
            registers: Registers::new(),
            bus: MemoryBus::new(cart),
            interrupt_master_enable: true,
            halted: false,
        }
    }

    pub fn step_single_instruction(&mut self) -> u8 {
        let cycles_this_instruction = if self.halted {
            4
        } else {
            self.run_next_instruction()
        };

        self.bus.apu.step(cycles_this_instruction);
        cycles_this_instruction
    }

    pub fn end_frame(&mut self) {
        self.bus.apu.end_frame();
        self.bus.ppu.render();
    }

    fn run_next_instruction(&mut self) -> u8 {
        let instruction = self.next_instruction().unwrap();
        let (next_pc, cycles) = self.execute(instruction);
        self.registers.pc = next_pc;

        let all_interrupts = [
            Interrupt::VBlank,
            Interrupt::LCDStat,
            Interrupt::Timer,
            Interrupt::Serial,
            Interrupt::Joypad,
        ];

        let interrupts_to_flag = self.bus.ppu.step(cycles);
        for interrupt in all_interrupts.iter() {
            if interrupts_to_flag.is_interrupt_set(*interrupt) {
                interrupt.set_interrupt_flag(&mut self.bus);
            }
        }

        if self.interrupt_master_enable {
            'run_interrupts: for interrupt in all_interrupts.iter() {
                if interrupt.should_process_interrupt(&self.bus) {
                    interrupt.clear_interrupt_flag(&mut self.bus);
                    self.interrupt(*interrupt);
                    break 'run_interrupts;
                }
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
            Instruction::ADC(source) => {
                let (value, pc_offset) = source.get_byte_and_pc_offset(self);
                let new_value = self.add_with_carry(value);
                self.registers.a = new_value;
                let cycles = match source {
                    ArithmeticSource::HL_INDIRECT => 8,
                    ArithmeticSource::D8 => 8,
                    _ => 4,
                };
                (self.registers.pc.wrapping_add(pc_offset), cycles)
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
            Instruction::DAA => {
                self.registers.a = self.decimal_adjust_accumulator();
                (self.registers.pc.wrapping_add(1), 4)
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
            Instruction::RLA => {
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
            Instruction::RLCA => {
                let source = ArithmeticSource::A;
                let (value, pc_offset) = source.get_byte_and_pc_offset(self);
                let new_value = self.rotate(value, RotateDirection::Left, false);
                source.set_byte(new_value, self);
                (self.registers.pc.wrapping_add(pc_offset), 4)
            }
            Instruction::RLC(source) => {
                let (value, pc_offset) = source.get_byte_and_pc_offset(self);
                let new_value = self.rotate(value, RotateDirection::Left, true);
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
            Instruction::RES(bit_to_reset, target) => {
                let (value, pc_offset) = target.get_byte_and_pc_offset(&self);
                let new_value = self.reset_bit(value, bit_to_reset);
                target.set_byte(new_value, self);
                let cycles = match target {
                    ArithmeticSource::HL_INDIRECT => 16,
                    _ => 8,
                };
                (self.registers.pc.wrapping_add(pc_offset + 1), cycles)
            }
            Instruction::HALT => {
                self.halted = true;
                (self.registers.pc.wrapping_add(1), 4)
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
            Instruction::SLA(source) => {
                let (value, pc_offset) = source.get_byte_and_pc_offset(self);
                let new_value = self.bit_shift(value, RotateDirection::Left, false);
                source.set_byte(new_value, self);
                let cycles = match source {
                    ArithmeticSource::HL_INDIRECT => 16,
                    _ => 8,
                };
                (self.registers.pc.wrapping_add(pc_offset + 1), cycles)
            }
            Instruction::SRA(source) => {
                let (value, pc_offset) = source.get_byte_and_pc_offset(self);
                let new_value = self.bit_shift(value, RotateDirection::Right, true);
                source.set_byte(new_value, self);
                let cycles = match source {
                    ArithmeticSource::HL_INDIRECT => 16,
                    _ => 8,
                };
                (self.registers.pc.wrapping_add(pc_offset + 1), cycles)
            }
            Instruction::SRL(source) => {
                let (value, pc_offset) = source.get_byte_and_pc_offset(self);
                let new_value = self.bit_shift(value, RotateDirection::Right, false);
                source.set_byte(new_value, self);
                let cycles = match source {
                    ArithmeticSource::HL_INDIRECT => 16,
                    _ => 8,
                };
                (self.registers.pc.wrapping_add(pc_offset + 1), cycles)
            }
            Instruction::SWAP(source) => {
                let (value, pc_offset) = source.get_byte_and_pc_offset(&self);
                let new_value = self.swap_nibbles(value);
                source.set_byte(new_value, self);
                let cycles = match source {
                    ArithmeticSource::HL_INDIRECT => 16,
                    _ => 8,
                };
                (self.registers.pc.wrapping_add(pc_offset + 1), cycles)
            }
            Instruction::RST(address) => {
                let next_pc = self.restart(address);
                (next_pc, 16)
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

    fn add_with_carry(&mut self, value: u8) -> u8 {
        let carry_bit = if self.registers.f.carry { 1 } else { 0 };
        let (new_value_without_carry, first_carry) = self.registers.a.overflowing_add(value);
        let (new_value_with_carry, second_carry) =
            new_value_without_carry.overflowing_add(carry_bit);
        self.registers.f.zero = new_value_with_carry == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = first_carry || second_carry;
        self.registers.f.half_carry =
            ((self.registers.a & 0x0F) + (value & 0x0F) + carry_bit) > 0x0F;
        new_value_with_carry
    }

    fn subtract(&mut self, value: u8) -> u8 {
        let (new_value, did_overflow) = self.registers.a.overflowing_sub(value);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = true;
        self.registers.f.carry = did_overflow;
        self.registers.f.half_carry = (self.registers.a & 0x0F) > (value & 0x0F);
        new_value
    }

    fn decimal_adjust_accumulator(&mut self) -> u8 {
        let unadjusted_value = self.registers.a;
        let flags = self.registers.f;

        let mut correction = 0x00;
        if flags.half_carry || (!flags.subtract && (unadjusted_value & 0x0F) > 0x09) {
            correction |= 0x06;
        }
        if flags.carry || (!flags.subtract && unadjusted_value > 0x99) {
            correction |= 0x60;
        }

        let adjusted_value = if !flags.subtract {
            unadjusted_value.wrapping_add(correction)
        } else {
            unadjusted_value.wrapping_sub(correction)
        };

        self.registers.f.zero = adjusted_value == 0;
        self.registers.f.carry = correction >= 0x60;
        self.registers.f.half_carry = false;

        adjusted_value
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

    fn restart(&mut self, restart_target: RestartTarget) -> u16 {
        let address_to_return_to = self.registers.pc.wrapping_add(1);
        self.push(address_to_return_to);
        restart_target as u16
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

    fn rotate(&mut self, value: u8, direction: RotateDirection, set_zero: bool) -> u8 {
        let cycled_bit = match direction {
            RotateDirection::Left => (value & 0x80) >> 7,
            RotateDirection::Right => (value & 0x01) << 7,
        };
        let shifted_value = match direction {
            RotateDirection::Left => value << 1,
            RotateDirection::Right => value >> 1,
        };
        let new_value = shifted_value | cycled_bit;
        self.registers.f.zero = set_zero && new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = cycled_bit != 0;
        new_value
    }

    fn bit_test(&mut self, value: u8, bit_to_test: u8) {
        let mask = (1 << bit_to_test) as u8;
        self.registers.f.zero = (mask & value) == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = true;
    }

    fn reset_bit(&mut self, value: u8, bit_to_reset: u8) -> u8 {
        let mask = (1 << bit_to_reset).not() as u8;
        value & mask
    }

    fn bit_shift(&mut self, value: u8, direction: RotateDirection, retain_shifted_bit: bool) -> u8 {
        let shifted_value = match direction {
            RotateDirection::Left => value << 1,
            RotateDirection::Right => value >> 1,
        };
        let shifted_bit = match direction {
            RotateDirection::Left => (value & 0x80) >> 7,
            RotateDirection::Right => (value & 0x01),
        };
        let new_bit = match (direction, retain_shifted_bit) {
            (RotateDirection::Left, _) => 0,
            (RotateDirection::Right, false) => 0,
            (RotateDirection::Right, true) => shifted_bit << 7,
        };
        let new_value = shifted_value | new_bit;
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = shifted_bit != 0;
        new_value
    }

    fn swap_nibbles(&mut self, value: u8) -> u8 {
        let new_lower = (value & 0xF0) >> 4;
        let new_upper = (value & 0x0F) << 4;
        let swapped = new_lower | new_upper;
        self.registers.f.zero = swapped == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = false;
        swapped
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
        self.halted = false;
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

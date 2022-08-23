// a simple stack language like interface on top of reglang
use crate::reglang::{Immediate, Instruction, Load, Register, Store};
use byteorder::{ByteOrder, LittleEndian};
use strum_macros::{Display, EnumIter};

#[allow(non_camel_case_types)]
#[derive(EnumIter, Debug, Display, Eq, PartialEq, Copy, Clone, FromPrimitive, ToPrimitive)]
pub enum ByteInstr {
    ADDI,
    SLTI,
    ANDI,
    ORI,
    XORI,
    SLLI,
    SRAI,
    ADD,
    SLT,
    AND,
    OR,
    XOR,
    SLL,
    SRA,
    LOAD,
    STORE,
}

pub struct Assembler {}

trait ValueAssembler {
    fn disassemble(input: &[u8]) -> Self;
    fn assemble(&self, output: &mut Vec<u8>);
}

impl Assembler {
    pub fn new() -> Assembler {
        Assembler {}
    }

    pub fn assemble(&self, instructions: &[Instruction]) -> Vec<u8> {
        let mut result = Vec::new();
        for instruction in instructions {
            ByteInstr::assemble(instruction, &mut result);
        }
        result
    }

    pub fn disassemble(&self, values: &[u8]) -> Vec<Instruction> {
        let mut result = Vec::new();
        let mut index: usize = 0;
        while index < values.len() {
            let instruction_values = &values[index..index + 7];
            if let Some(byte_instr) = ByteInstr::decode(instruction_values[6]) {
                result.push(byte_instr.disassemble(instruction_values));
            }
            index += 7;
        }
        result
    }
}

impl ByteInstr {
    fn encode(&self) -> u8 {
        num::ToPrimitive::to_u8(self).unwrap()
    }

    fn decode(value: u8) -> Option<ByteInstr> {
        num::FromPrimitive::from_u8(value)
    }

    fn assemble(instruction: &Instruction, output: &mut Vec<u8>) {
        match instruction {
            Instruction::AddI(immediate) => {
                immediate.assemble(output);
                output.push(ByteInstr::ADDI.encode());
            }
            Instruction::SltI(immediate) => {
                immediate.assemble(output);
                output.push(ByteInstr::SLTI.encode());
            }
            Instruction::AndI(immediate) => {
                immediate.assemble(output);
                output.push(ByteInstr::ANDI.encode());
            }
            Instruction::OrI(immediate) => {
                immediate.assemble(output);
                output.push(ByteInstr::ORI.encode());
            }
            Instruction::XorI(immediate) => {
                immediate.assemble(output);
                output.push(ByteInstr::XORI.encode());
            }
            Instruction::SllI(immediate) => {
                immediate.assemble(output);
                output.push(ByteInstr::SLLI.encode());
            }
            Instruction::SraI(immediate) => {
                immediate.assemble(output);
                output.push(ByteInstr::SRAI.encode());
            }
            Instruction::Add(register) => {
                register.assemble(output);
                output.push(ByteInstr::ADD.encode());
            }
            Instruction::Slt(register) => {
                register.assemble(output);
                output.push(ByteInstr::SLT.encode());
            }
            Instruction::And(register) => {
                register.assemble(output);
                output.push(ByteInstr::AND.encode());
            }
            Instruction::Or(register) => {
                register.assemble(output);
                output.push(ByteInstr::OR.encode());
            }
            Instruction::Xor(register) => {
                register.assemble(output);
                output.push(ByteInstr::XOR.encode());
            }
            Instruction::Sll(register) => {
                register.assemble(output);
                output.push(ByteInstr::SLL.encode());
            }
            Instruction::Sra(register) => {
                register.assemble(output);
                output.push(ByteInstr::SRA.encode());
            }
            Instruction::Load(load) => {
                load.assemble(output);
                output.push(ByteInstr::LOAD.encode());
            }
            Instruction::Store(store) => {
                store.assemble(output);
                output.push(ByteInstr::STORE.encode());
            }
        }
    }

    fn disassemble(&self, values: &[u8]) -> Instruction {
        match self {
            ByteInstr::ADDI => Instruction::AddI(Immediate::disassemble(values)),
            ByteInstr::SLTI => Instruction::SltI(Immediate::disassemble(values)),
            ByteInstr::ANDI => Instruction::AndI(Immediate::disassemble(values)),
            ByteInstr::ORI => Instruction::OrI(Immediate::disassemble(values)),
            ByteInstr::XORI => Instruction::XorI(Immediate::disassemble(values)),
            ByteInstr::SLLI => Instruction::SllI(Immediate::disassemble(values)),
            ByteInstr::SRAI => Instruction::SraI(Immediate::disassemble(values)),
            ByteInstr::ADD => Instruction::Add(Register::disassemble(values)),
            ByteInstr::SLT => Instruction::Slt(Register::disassemble(values)),
            ByteInstr::AND => Instruction::And(Register::disassemble(values)),
            ByteInstr::OR => Instruction::Or(Register::disassemble(values)),
            ByteInstr::XOR => Instruction::Xor(Register::disassemble(values)),
            ByteInstr::SLL => Instruction::Sll(Register::disassemble(values)),
            ByteInstr::SRA => Instruction::Sra(Register::disassemble(values)),
            ByteInstr::LOAD => Instruction::Load(Load::disassemble(values)),
            ByteInstr::STORE => Instruction::Store(Store::disassemble(values)),
        }
    }
}

impl ValueAssembler for Immediate {
    fn disassemble(input: &[u8]) -> Self {
        Immediate {
            value: bytes_to_i16(&input[0..2]),
            rs: bytes_to_i16(&input[2..4]),
            rd: bytes_to_i16(&input[4..6]),
        }
    }
    fn assemble(&self, output: &mut Vec<u8>) {
        output.extend(i16_to_bytes(self.value));
        output.extend(i16_to_bytes(self.rs));
        output.extend(i16_to_bytes(self.rd));
    }
}

impl ValueAssembler for Load {
    fn disassemble(input: &[u8]) -> Self {
        Load {
            offset: bytes_to_i16(&input[0..2]),
            rs: bytes_to_i16(&input[2..4]),
            rd: bytes_to_i16(&input[4..6]),
        }
    }
    fn assemble(&self, output: &mut Vec<u8>) {
        output.extend(i16_to_bytes(self.offset));
        output.extend(i16_to_bytes(self.rs));
        output.extend(i16_to_bytes(self.rd));
    }
}

impl ValueAssembler for Store {
    fn disassemble(input: &[u8]) -> Self {
        Store {
            offset: bytes_to_i16(&input[0..2]),
            rs: bytes_to_i16(&input[2..4]),
            rd: bytes_to_i16(&input[4..6]),
        }
    }
    fn assemble(&self, output: &mut Vec<u8>) {
        output.extend(i16_to_bytes(self.offset));
        output.extend(i16_to_bytes(self.rs));
        output.extend(i16_to_bytes(self.rd));
    }
}

impl ValueAssembler for Register {
    fn disassemble(input: &[u8]) -> Self {
        Register {
            rs1: bytes_to_i16(&input[0..2]),
            rs2: bytes_to_i16(&input[2..4]),
            rd: bytes_to_i16(&input[4..6]),
        }
    }
    fn assemble(&self, output: &mut Vec<u8>) {
        output.extend(i16_to_bytes(self.rs1));
        output.extend(i16_to_bytes(self.rs2));
        output.extend(i16_to_bytes(self.rd));
    }
}

fn i16_to_bytes(value: i16) -> [u8; 2] {
    let mut buffer = [0u8; 2];
    LittleEndian::write_i16(&mut buffer, value);
    buffer
}

fn bytes_to_i16(input: &[u8]) -> i16 {
    LittleEndian::read_i16(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regstack_assemble() {
        let assembler = Assembler::new();
        let bytes = assembler.assemble(&[Instruction::Add(Register {
            rs1: 0,
            rs2: 1,
            rd: 2,
        })]);
        assert_eq!(bytes, vec![0, 0, 1, 0, 2, 0, ByteInstr::ADD.encode()]);
    }

    #[test]
    fn test_regstack_disassemble() {
        let assembler = Assembler::new();
        let bytes = assembler.assemble(&[Instruction::Add(Register {
            rs1: 0,
            rs2: 1,
            rd: 2,
        })]);
        let instructions = assembler.disassemble(&bytes);
        assert_eq!(instructions.len(), 1);
        assert_eq!(
            instructions[0],
            Instruction::Add(Register {
                rs1: 0,
                rs2: 1,
                rd: 2,
            })
        );
    }

    #[test]
    fn test_regstack_disassemble_invalid_instruction() {
        // 127 isn't going to be a valid instruction soon
        let bytes = vec![0, 0, 1, 0, 2, 0, 127];
        let assembler = Assembler::new();
        let instructions = assembler.disassemble(&bytes);
        assert_eq!(instructions.len(), 0);
    }
}

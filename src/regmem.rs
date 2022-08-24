use crate::reglang::{Immediate, Instruction, Load, Register, Store};
use byteorder::{ByteOrder, LittleEndian};

#[allow(non_camel_case_types)]
#[derive(Debug, Eq, PartialEq, Copy, Clone, FromPrimitive, ToPrimitive)]
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
    LB,
    SB,
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
            if let Some(byte_instr) = ByteInstr::decode(instruction_values[0]) {
                result.push(byte_instr.disassemble(&instruction_values[1..]));
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
            Instruction::Addi(immediate) => {
                output.push(ByteInstr::ADDI.encode());
                immediate.assemble(output);
            }
            Instruction::Slti(immediate) => {
                output.push(ByteInstr::SLTI.encode());
                immediate.assemble(output);
            }
            Instruction::Andi(immediate) => {
                output.push(ByteInstr::ANDI.encode());
                immediate.assemble(output);
            }
            Instruction::Ori(immediate) => {
                output.push(ByteInstr::ORI.encode());
                immediate.assemble(output);
            }
            Instruction::Xori(immediate) => {
                output.push(ByteInstr::XORI.encode());
                immediate.assemble(output);
            }
            Instruction::Slli(immediate) => {
                output.push(ByteInstr::SLLI.encode());
                immediate.assemble(output);
            }
            Instruction::Srai(immediate) => {
                output.push(ByteInstr::SRAI.encode());
                immediate.assemble(output);
            }
            Instruction::Add(register) => {
                output.push(ByteInstr::ADD.encode());
                register.assemble(output);
            }
            Instruction::Slt(register) => {
                output.push(ByteInstr::SLT.encode());
                register.assemble(output);
            }
            Instruction::And(register) => {
                output.push(ByteInstr::AND.encode());
                register.assemble(output);
            }
            Instruction::Or(register) => {
                output.push(ByteInstr::OR.encode());
                register.assemble(output);
            }
            Instruction::Xor(register) => {
                output.push(ByteInstr::XOR.encode());
                register.assemble(output);
            }
            Instruction::Sll(register) => {
                output.push(ByteInstr::SLL.encode());
                register.assemble(output);
            }
            Instruction::Sra(register) => {
                output.push(ByteInstr::SRA.encode());
                register.assemble(output);
            }
            Instruction::Lb(load) => {
                output.push(ByteInstr::LB.encode());
                load.assemble(output);
            }
            Instruction::Sb(store) => {
                output.push(ByteInstr::SB.encode());
                store.assemble(output);
            }
            _ => {}
        }
    }

    fn disassemble(&self, values: &[u8]) -> Instruction {
        match self {
            ByteInstr::ADDI => Instruction::Addi(Immediate::disassemble(values)),
            ByteInstr::SLTI => Instruction::Slti(Immediate::disassemble(values)),
            ByteInstr::ANDI => Instruction::Andi(Immediate::disassemble(values)),
            ByteInstr::ORI => Instruction::Ori(Immediate::disassemble(values)),
            ByteInstr::XORI => Instruction::Xori(Immediate::disassemble(values)),
            ByteInstr::SLLI => Instruction::Slli(Immediate::disassemble(values)),
            ByteInstr::SRAI => Instruction::Srai(Immediate::disassemble(values)),
            ByteInstr::ADD => Instruction::Add(Register::disassemble(values)),
            ByteInstr::SLT => Instruction::Slt(Register::disassemble(values)),
            ByteInstr::AND => Instruction::And(Register::disassemble(values)),
            ByteInstr::OR => Instruction::Or(Register::disassemble(values)),
            ByteInstr::XOR => Instruction::Xor(Register::disassemble(values)),
            ByteInstr::SLL => Instruction::Sll(Register::disassemble(values)),
            ByteInstr::SRA => Instruction::Sra(Register::disassemble(values)),
            ByteInstr::LB => Instruction::Lb(Load::disassemble(values)),
            ByteInstr::SB => Instruction::Sb(Store::disassemble(values)),
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
    fn test_assemble() {
        let assembler = Assembler::new();
        let bytes = assembler.assemble(&[Instruction::Add(Register {
            rs1: 0,
            rs2: 1,
            rd: 2,
        })]);
        assert_eq!(bytes, vec![ByteInstr::ADD.encode(), 0, 0, 1, 0, 2, 0]);
    }

    #[test]
    fn test_disassemble() {
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
    fn test_disassemble_invalid_instruction() {
        // 127 isn't going to be a valid instruction soon
        let bytes = vec![127, 0, 0, 1, 0, 2, 0];
        let assembler = Assembler::new();
        let instructions = assembler.disassemble(&bytes);
        assert_eq!(instructions.len(), 0);
    }
}

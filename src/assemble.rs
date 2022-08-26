use crate::lang::{Branch, BranchTarget, Immediate, Instruction, Load, Opcode, Register, Store};
use byteorder::{ByteOrder, LittleEndian};

pub struct Assembler {}

trait ValueAssembler {
    fn assemble(&self, output: &mut Vec<u8>);
}

trait ValueDisassembler {
    fn size() -> usize;
    fn disassemble(input: &[u8]) -> Self;
}

impl ValueAssembler for Instruction {
    fn assemble(&self, output: &mut Vec<u8>) {
        output.push(opcode(self));
        use Instruction::*;
        match self {
            Addi(immediate) | Slti(immediate) | Sltiu(immediate) | Andi(immediate)
            | Ori(immediate) | Xori(immediate) | Slli(immediate) | Srli(immediate)
            | Srai(immediate) => immediate.assemble(output),
            Add(register) | Sub(register) | Sll(register) | Srl(register) | Sra(register) => {
                register.assemble(output)
            }
            Lh(load) | Lbu(load) | Lb(load) => load.assemble(output),
            Sh(store) | Sb(store) => store.assemble(output),
            _ => {
                panic!("unimplemented instruction: {:?}", self)
            }
        }
    }
}

fn opcode(instruction: &Instruction) -> u8 {
    let opcode: Opcode = instruction.into();
    opcode.encode()
}

impl Default for Assembler {
    fn default() -> Self {
        Assembler::new()
    }
}

impl Assembler {
    pub fn new() -> Assembler {
        Assembler {}
    }

    pub fn assemble(&self, instructions: &[Instruction]) -> Vec<u8> {
        let mut result = Vec::new();
        for instruction in instructions {
            instruction.assemble(&mut result);
        }
        result
    }

    pub fn disassemble(&self, values: &[u8]) -> Vec<Instruction> {
        let mut result = Vec::new();
        let mut index: usize = 0;
        while index < values.len() {
            if let Some(opcode) = Opcode::decode(values[index]) {
                let start = index + 1;
                let end = start + opcode.size();
                if end > values.len() {
                    break;
                }
                result.push(opcode.disassemble(&values[start..end]));
                index = end;
            } else {
                index += 1;
            }
        }
        result
    }
}

impl Opcode {
    fn encode(&self) -> u8 {
        num::ToPrimitive::to_u8(self).unwrap()
    }

    fn decode(value: u8) -> Option<Opcode> {
        num::FromPrimitive::from_u8(value)
    }

    fn size(&self) -> usize {
        use Opcode::*;
        match self {
            Addi | Slti | Sltiu | Andi | Ori | Xori | Slli | Srli | Srai => Immediate::size(),
            Add | Sub | Slt | Sltu | And | Or | Xor | Sll | Srl | Sra => Register::size(),
            Lh | Lbu | Lb => Load::size(),
            Sh | Sb => Store::size(),
            Beq => Branch::size(),
            Target => BranchTarget::size(),
        }
    }
    fn disassemble(&self, values: &[u8]) -> Instruction {
        use Opcode::*;
        match self {
            Addi => Instruction::Addi(Immediate::disassemble(values)),
            Slti => Instruction::Slti(Immediate::disassemble(values)),
            Sltiu => Instruction::Sltiu(Immediate::disassemble(values)),
            Andi => Instruction::Andi(Immediate::disassemble(values)),
            Ori => Instruction::Ori(Immediate::disassemble(values)),
            Xori => Instruction::Xori(Immediate::disassemble(values)),
            Slli => Instruction::Slli(Immediate::disassemble(values)),
            Srli => Instruction::Srli(Immediate::disassemble(values)),
            Srai => Instruction::Srai(Immediate::disassemble(values)),
            Add => Instruction::Add(Register::disassemble(values)),
            Sub => Instruction::Sub(Register::disassemble(values)),
            Slt => Instruction::Slt(Register::disassemble(values)),
            Sltu => Instruction::Sltu(Register::disassemble(values)),
            And => Instruction::And(Register::disassemble(values)),
            Or => Instruction::Or(Register::disassemble(values)),
            Xor => Instruction::Xor(Register::disassemble(values)),
            Sll => Instruction::Sll(Register::disassemble(values)),
            Srl => Instruction::Srl(Register::disassemble(values)),
            Sra => Instruction::Sra(Register::disassemble(values)),
            Lh => Instruction::Lh(Load::disassemble(values)),
            Lbu => Instruction::Lbu(Load::disassemble(values)),
            Lb => Instruction::Lb(Load::disassemble(values)),
            Sh => Instruction::Sh(Store::disassemble(values)),
            Sb => Instruction::Sb(Store::disassemble(values)),
            Beq => Instruction::Beq(Branch::disassemble(values)),
            Target => Instruction::Target(BranchTarget::disassemble(values)),
        }
    }
}

fn clampreg(value: u8) -> u8 {
    value % 32
}

impl ValueDisassembler for Immediate {
    fn size() -> usize {
        4
    }

    fn disassemble(input: &[u8]) -> Immediate {
        Immediate {
            value: bytes_to_i16(&input[0..2]),
            rs: clampreg(input[2]),
            rd: clampreg(input[3]),
        }
    }
}

impl ValueAssembler for Immediate {
    fn assemble(&self, output: &mut Vec<u8>) {
        output.extend(i16_to_bytes(self.value));
        output.push(self.rs);
        output.push(self.rd);
    }
}

impl ValueDisassembler for Load {
    fn size() -> usize {
        4
    }
    fn disassemble(input: &[u8]) -> Load {
        Load {
            offset: bytes_to_u16(&input[0..2]),
            rs: clampreg(input[2]),
            rd: clampreg(input[3]),
        }
    }
}

impl ValueAssembler for Load {
    fn assemble(&self, output: &mut Vec<u8>) {
        output.extend(u16_to_bytes(self.offset));
        output.push(self.rs);
        output.push(self.rd);
    }
}

impl ValueDisassembler for Store {
    fn size() -> usize {
        4
    }
    fn disassemble(input: &[u8]) -> Self {
        Store {
            offset: bytes_to_u16(&input[0..2]),
            rs: clampreg(input[2]),
            rd: clampreg(input[3]),
        }
    }
}

impl ValueAssembler for Store {
    fn assemble(&self, output: &mut Vec<u8>) {
        output.extend(u16_to_bytes(self.offset));
        output.push(self.rs);
        output.push(self.rd);
    }
}

impl ValueDisassembler for Register {
    fn size() -> usize {
        3
    }
    fn disassemble(input: &[u8]) -> Self {
        Register {
            rs1: clampreg(input[0]),
            rs2: clampreg(input[1]),
            rd: clampreg(input[2]),
        }
    }
}

impl ValueAssembler for Register {
    fn assemble(&self, output: &mut Vec<u8>) {
        output.push(self.rs1);
        output.push(self.rs2);
        output.push(self.rd);
    }
}

impl ValueDisassembler for Branch {
    fn size() -> usize {
        3
    }
    fn disassemble(input: &[u8]) -> Self {
        Branch {
            target: input[0],
            rs1: clampreg(input[1]),
            rs2: clampreg(input[2]),
        }
    }
}

impl ValueAssembler for Branch {
    fn assemble(&self, output: &mut Vec<u8>) {
        output.push(self.target);
        output.push(self.rs1);
        output.push(self.rs2);
    }
}

impl ValueDisassembler for BranchTarget {
    fn size() -> usize {
        1
    }
    fn disassemble(input: &[u8]) -> Self {
        BranchTarget {
            identifier: input[0],
        }
    }
}

impl ValueAssembler for BranchTarget {
    fn assemble(&self, output: &mut Vec<u8>) {
        output.push(self.identifier);
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

fn u16_to_bytes(value: u16) -> [u8; 2] {
    let mut buffer = [0u8; 2];
    LittleEndian::write_u16(&mut buffer, value);
    buffer
}

fn bytes_to_u16(input: &[u8]) -> u16 {
    LittleEndian::read_u16(input)
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
        assert_eq!(bytes, vec![Opcode::Add.encode(), 0, 1, 2]);
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
        // 127 isn't going to be a valid instruction soon, but 0 is, but there isn't
        // enough data to disassemble it
        let bytes = vec![127, 0, 0];
        let assembler = Assembler::new();
        let instructions = assembler.disassemble(&bytes);
        assert_eq!(instructions.len(), 0);
    }

    #[test]
    fn test_disassemble_register_out_of_range() {
        let bytes = vec![0, 10, 0, 43, 0];
        let assembler = Assembler::new();
        let instructions = assembler.disassemble(&bytes);
        assert_eq!(instructions.len(), 1);
        assert_eq!(
            instructions[0],
            Instruction::Addi(Immediate {
                value: 10,
                rs: 11, // 43 clamped to 32
                rd: 0,
            })
        );
    }
}

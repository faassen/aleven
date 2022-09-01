use crate::lang::{
    Branch, BranchTarget, CallId, Immediate, Instruction, Load, Opcode, Register, Store,
};
use byteorder::{ByteOrder, LittleEndian};

pub struct Serializer {}

#[derive(Debug, PartialEq, Eq)]
pub enum OpcodeType {
    Register,
    Immediate,
    Load,
    Store,
    Branch,
    BranchTarget,
    Call,
}

trait ValueSerializer {
    fn serialize(&self, output: &mut Vec<u8>);
}

trait ValueDerializer {
    fn size() -> usize;
    fn deserialize(input: &[u8]) -> Self;
}

impl ValueSerializer for Instruction {
    fn serialize(&self, output: &mut Vec<u8>) {
        output.push(opcode(self));
        use Instruction::*;
        match self {
            Addi(immediate) | Slti(immediate) | Sltiu(immediate) | Andi(immediate)
            | Ori(immediate) | Xori(immediate) | Slli(immediate) | Srli(immediate)
            | Srai(immediate) => immediate.serialize(output),
            Add(register) | Sub(register) | Sll(register) | Srl(register) | Sra(register) => {
                register.serialize(output)
            }
            Lh(load) | Lbu(load) | Lb(load) => load.serialize(output),
            Sh(store) | Sb(store) => store.serialize(output),
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

impl Default for Serializer {
    fn default() -> Self {
        Serializer::new()
    }
}

impl Serializer {
    pub fn new() -> Serializer {
        Serializer {}
    }

    pub fn serialize(&self, instructions: &[Instruction]) -> Vec<u8> {
        let mut result = Vec::new();
        for instruction in instructions {
            instruction.serialize(&mut result);
        }
        result
    }

    pub fn deserialize(&self, values: &[u8]) -> Vec<Instruction> {
        let mut result = Vec::new();
        let mut index: usize = 0;
        while index < values.len() {
            if let Some(opcode) = Opcode::decode(values[index]) {
                let start = index + 1;
                let opcode_type: OpcodeType = opcode.into();
                let end = start + opcode_type.size();
                if end > values.len() {
                    break;
                }
                result.push(opcode.deserialize(&values[start..end]));
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

    fn deserialize(&self, values: &[u8]) -> Instruction {
        use Opcode::*;
        match self {
            Addi => Instruction::Addi(Immediate::deserialize(values)),
            Slti => Instruction::Slti(Immediate::deserialize(values)),
            Sltiu => Instruction::Sltiu(Immediate::deserialize(values)),
            Andi => Instruction::Andi(Immediate::deserialize(values)),
            Ori => Instruction::Ori(Immediate::deserialize(values)),
            Xori => Instruction::Xori(Immediate::deserialize(values)),
            Slli => Instruction::Slli(Immediate::deserialize(values)),
            Srli => Instruction::Srli(Immediate::deserialize(values)),
            Srai => Instruction::Srai(Immediate::deserialize(values)),
            Add => Instruction::Add(Register::deserialize(values)),
            Sub => Instruction::Sub(Register::deserialize(values)),
            Slt => Instruction::Slt(Register::deserialize(values)),
            Sltu => Instruction::Sltu(Register::deserialize(values)),
            And => Instruction::And(Register::deserialize(values)),
            Or => Instruction::Or(Register::deserialize(values)),
            Xor => Instruction::Xor(Register::deserialize(values)),
            Sll => Instruction::Sll(Register::deserialize(values)),
            Srl => Instruction::Srl(Register::deserialize(values)),
            Sra => Instruction::Sra(Register::deserialize(values)),
            Lh => Instruction::Lh(Load::deserialize(values)),
            Lbu => Instruction::Lbu(Load::deserialize(values)),
            Lb => Instruction::Lb(Load::deserialize(values)),
            Sh => Instruction::Sh(Store::deserialize(values)),
            Sb => Instruction::Sb(Store::deserialize(values)),
            Beq => Instruction::Beq(Branch::deserialize(values)),
            Target => Instruction::Target(BranchTarget::deserialize(values)),
            Call => Instruction::Call(CallId::deserialize(values)),
        }
    }
}

impl OpcodeType {
    fn size(&self) -> usize {
        match self {
            OpcodeType::Immediate => Immediate::size(),
            OpcodeType::Register => Register::size(),
            OpcodeType::Load => Load::size(),
            OpcodeType::Store => Store::size(),
            OpcodeType::Branch => Branch::size(),
            OpcodeType::BranchTarget => BranchTarget::size(),
            OpcodeType::Call => CallId::size(),
        }
    }
}

fn clampreg(value: u8) -> u8 {
    value % 32
}

impl ValueDerializer for Immediate {
    fn size() -> usize {
        4
    }

    fn deserialize(input: &[u8]) -> Immediate {
        Immediate {
            value: bytes_to_i16(&input[0..2]),
            rs: clampreg(input[2]),
            rd: clampreg(input[3]),
        }
    }
}

impl ValueSerializer for Immediate {
    fn serialize(&self, output: &mut Vec<u8>) {
        output.extend(i16_to_bytes(self.value));
        output.push(self.rs);
        output.push(self.rd);
    }
}

impl ValueDerializer for Load {
    fn size() -> usize {
        4
    }
    fn deserialize(input: &[u8]) -> Load {
        Load {
            offset: bytes_to_u16(&input[0..2]),
            rs: clampreg(input[2]),
            rd: clampreg(input[3]),
        }
    }
}

impl ValueSerializer for Load {
    fn serialize(&self, output: &mut Vec<u8>) {
        output.extend(u16_to_bytes(self.offset));
        output.push(self.rs);
        output.push(self.rd);
    }
}

impl ValueDerializer for Store {
    fn size() -> usize {
        4
    }
    fn deserialize(input: &[u8]) -> Self {
        Store {
            offset: bytes_to_u16(&input[0..2]),
            rs: clampreg(input[2]),
            rd: clampreg(input[3]),
        }
    }
}

impl ValueSerializer for Store {
    fn serialize(&self, output: &mut Vec<u8>) {
        output.extend(u16_to_bytes(self.offset));
        output.push(self.rs);
        output.push(self.rd);
    }
}

impl ValueDerializer for Register {
    fn size() -> usize {
        3
    }
    fn deserialize(input: &[u8]) -> Self {
        Register {
            rs1: clampreg(input[0]),
            rs2: clampreg(input[1]),
            rd: clampreg(input[2]),
        }
    }
}

impl ValueSerializer for Register {
    fn serialize(&self, output: &mut Vec<u8>) {
        output.push(self.rs1);
        output.push(self.rs2);
        output.push(self.rd);
    }
}

impl ValueDerializer for Branch {
    fn size() -> usize {
        3
    }
    fn deserialize(input: &[u8]) -> Self {
        Branch {
            target: input[0],
            rs1: clampreg(input[1]),
            rs2: clampreg(input[2]),
        }
    }
}

impl ValueSerializer for Branch {
    fn serialize(&self, output: &mut Vec<u8>) {
        output.push(self.target);
        output.push(self.rs1);
        output.push(self.rs2);
    }
}

impl ValueDerializer for BranchTarget {
    fn size() -> usize {
        1
    }
    fn deserialize(input: &[u8]) -> Self {
        BranchTarget {
            identifier: input[0],
        }
    }
}

impl ValueSerializer for BranchTarget {
    fn serialize(&self, output: &mut Vec<u8>) {
        output.push(self.identifier);
    }
}

impl ValueDerializer for CallId {
    fn size() -> usize {
        2
    }
    fn deserialize(input: &[u8]) -> Self {
        CallId {
            identifier: bytes_to_u16(&input[0..2]),
        }
    }
}

impl ValueSerializer for CallId {
    fn serialize(&self, output: &mut Vec<u8>) {
        output.extend(u16_to_bytes(self.identifier));
    }
}

impl From<Opcode> for OpcodeType {
    fn from(opcode: Opcode) -> Self {
        use Opcode::*;
        match opcode {
            Addi | Slti | Sltiu | Andi | Ori | Xori | Slli | Srli | Srai => OpcodeType::Immediate,
            Add | Sub | Slt | Sltu | And | Or | Xor | Sll | Srl | Sra => OpcodeType::Register,
            Lh | Lbu | Lb => OpcodeType::Load,
            Sh | Sb => OpcodeType::Store,
            Beq => OpcodeType::Branch,
            Target => OpcodeType::BranchTarget,
            Call => OpcodeType::Call,
        }
    }
}

impl From<(Opcode, Immediate)> for Instruction {
    fn from((opcode, immediate): (Opcode, Immediate)) -> Self {
        use Opcode::*;
        match opcode {
            Addi => Instruction::Addi(immediate),
            Slti => Instruction::Slti(immediate),
            Sltiu => Instruction::Sltiu(immediate),
            Andi => Instruction::Andi(immediate),
            Ori => Instruction::Ori(immediate),
            Xori => Instruction::Xori(immediate),
            Slli => Instruction::Slli(immediate),
            Srli => Instruction::Srli(immediate),
            Srai => Instruction::Srai(immediate),
            _ => {
                panic!("Invalid opcode for immediate instruction: {:?}", opcode)
            }
        }
    }
}

impl From<(Opcode, Register)> for Instruction {
    fn from((opcode, register): (Opcode, Register)) -> Self {
        use Opcode::*;
        match opcode {
            Add => Instruction::Add(register),
            Sub => Instruction::Sub(register),
            Slt => Instruction::Slt(register),
            Sltu => Instruction::Sltu(register),
            And => Instruction::And(register),
            Or => Instruction::Or(register),
            Xor => Instruction::Xor(register),
            Sll => Instruction::Sll(register),
            Srl => Instruction::Srl(register),
            Sra => Instruction::Sra(register),
            _ => {
                panic!("Invalid opcode for register instruction: {:?}", opcode)
            }
        }
    }
}

impl From<(Opcode, Load)> for Instruction {
    fn from((opcode, load): (Opcode, Load)) -> Self {
        use Opcode::*;
        match opcode {
            Lh => Instruction::Lh(load),
            Lbu => Instruction::Lbu(load),
            Lb => Instruction::Lb(load),
            _ => {
                panic!("Invalid opcode for load instruction: {:?}", opcode)
            }
        }
    }
}

impl From<(Opcode, Store)> for Instruction {
    fn from((opcode, store): (Opcode, Store)) -> Self {
        use Opcode::*;
        match opcode {
            Sh => Instruction::Sh(store),
            Sb => Instruction::Sb(store),
            _ => {
                panic!("Invalid opcode for store instruction: {:?}", opcode)
            }
        }
    }
}

impl From<(Opcode, Branch)> for Instruction {
    fn from((opcode, branch): (Opcode, Branch)) -> Self {
        use Opcode::*;
        match opcode {
            Beq => Instruction::Beq(branch),
            _ => {
                panic!("Invalid opcode for branch instruction: {:?}", opcode)
            }
        }
    }
}

impl From<(Opcode, BranchTarget)> for Instruction {
    fn from((opcode, branch_target): (Opcode, BranchTarget)) -> Self {
        use Opcode::*;
        match opcode {
            Target => Instruction::Target(branch_target),
            _ => {
                panic!("Invalid opcode for branch target instruction: {:?}", opcode)
            }
        }
    }
}

impl From<(Opcode, CallId)> for Instruction {
    fn from((opcode, call_id): (Opcode, CallId)) -> Self {
        use Opcode::*;
        match opcode {
            Call => Instruction::Call(call_id),
            _ => {
                panic!("Invalid opcode for call instruction: {:?}", opcode)
            }
        }
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
    fn test_serialize() {
        let serializer = Serializer::new();
        let bytes = serializer.serialize(&[Instruction::Add(Register {
            rs1: 0,
            rs2: 1,
            rd: 2,
        })]);
        assert_eq!(bytes, vec![Opcode::Add.encode(), 0, 1, 2]);
    }

    #[test]
    fn test_deserialize() {
        let serializer = Serializer::new();
        let bytes = serializer.serialize(&[Instruction::Add(Register {
            rs1: 0,
            rs2: 1,
            rd: 2,
        })]);
        let instructions = serializer.deserialize(&bytes);
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
    fn test_deserialize_invalid_instruction() {
        // 127 isn't going to be a valid instruction soon, but 0 is, but there isn't
        // enough data to disassemble it
        let bytes = vec![127, 0, 0];
        let serializer = Serializer::new();
        let instructions = serializer.deserialize(&bytes);
        assert_eq!(instructions.len(), 0);
    }

    #[test]
    fn test_deserialize_register_out_of_range() {
        let bytes = vec![0, 10, 0, 43, 0];
        let serializer = Serializer::new();
        let instructions = serializer.deserialize(&bytes);
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

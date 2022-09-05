use crate::lang::{
    Branch, BranchOpcode, BranchTarget, BranchTargetOpcode, CallId, CallIdOpcode, Immediate,
    ImmediateOpcode, Instruction, Load, LoadOpcode, Register, RegisterOpcode, Store, StoreOpcode,
};
use byteorder::{ByteOrder, LittleEndian};
use num::{FromPrimitive, ToPrimitive};

enum OpcodeWithType {
    Immediate(ImmediateOpcode),
    Register(RegisterOpcode),
    Load(LoadOpcode),
    Store(StoreOpcode),
    Branch(BranchOpcode),
    BranchTarget(BranchTargetOpcode),
    CallId(CallIdOpcode),
}

impl OpcodeWithType {
    fn size(&self) -> usize {
        match self {
            OpcodeWithType::Immediate(_opcode) => Immediate::size(),
            OpcodeWithType::Register(_opcode) => Register::size(),
            OpcodeWithType::Load(_opcode) => Load::size(),
            OpcodeWithType::Store(_opcode) => Store::size(),
            OpcodeWithType::Branch(_opcode) => Branch::size(),
            OpcodeWithType::BranchTarget(_opcode) => BranchTarget::size(),
            OpcodeWithType::CallId(_opcode) => CallId::size(),
        }
    }

    fn deserialize(&self, values: &[u8]) -> Instruction {
        match self {
            OpcodeWithType::Immediate(opcode) => {
                Instruction::Immediate(Immediate::deserialize(*opcode, values))
            }
            OpcodeWithType::Register(opcode) => {
                Instruction::Register(Register::deserialize(*opcode, values))
            }
            OpcodeWithType::Load(opcode) => Instruction::Load(Load::deserialize(*opcode, values)),
            OpcodeWithType::Store(opcode) => {
                Instruction::Store(Store::deserialize(*opcode, values))
            }
            OpcodeWithType::Branch(opcode) => {
                Instruction::Branch(Branch::deserialize(*opcode, values))
            }
            OpcodeWithType::BranchTarget(opcode) => {
                Instruction::BranchTarget(BranchTarget::deserialize(*opcode, values))
            }
            OpcodeWithType::CallId(opcode) => {
                Instruction::CallId(CallId::deserialize(*opcode, values))
            }
        }
    }
}

impl From<&Instruction> for u8 {
    fn from(instruction: &Instruction) -> Self {
        match instruction {
            Instruction::Immediate(Immediate { opcode, .. }) => opcode.to_u8().unwrap(),
            Instruction::Register(Register { opcode, .. }) => opcode.to_u8().unwrap(),
            Instruction::Load(Load { opcode, .. }) => opcode.to_u8().unwrap(),
            Instruction::Store(Store { opcode, .. }) => opcode.to_u8().unwrap(),
            Instruction::Branch(Branch { opcode, .. }) => opcode.to_u8().unwrap(),
            Instruction::BranchTarget(BranchTarget { opcode, .. }) => opcode.to_u8().unwrap(),
            Instruction::CallId(CallId { opcode, .. }) => opcode.to_u8().unwrap(),
        }
    }
}

fn decode_opcode(value: u8) -> Option<OpcodeWithType> {
    ImmediateOpcode::from_u8(value)
        .map(OpcodeWithType::Immediate)
        .or_else(|| RegisterOpcode::from_u8(value).map(OpcodeWithType::Register))
        .or_else(|| LoadOpcode::from_u8(value).map(OpcodeWithType::Load))
        .or_else(|| StoreOpcode::from_u8(value).map(OpcodeWithType::Store))
        .or_else(|| BranchOpcode::from_u8(value).map(OpcodeWithType::Branch))
        .or_else(|| BranchTargetOpcode::from_u8(value).map(OpcodeWithType::BranchTarget))
        .or_else(|| CallIdOpcode::from_u8(value).map(OpcodeWithType::CallId))
}

pub struct Serializer {}

trait ValueSerializer {
    fn serialize(&self, output: &mut Vec<u8>);
}

pub trait ValueDeserializer<T> {
    fn size() -> usize;
    fn deserialize(opcode: T, input: &[u8]) -> Self;
}

impl ValueSerializer for Instruction {
    fn serialize(&self, output: &mut Vec<u8>) {
        output.push(self.into());
        use Instruction::*;
        match self {
            Immediate(immediate) => immediate.serialize(output),
            Register(register) => register.serialize(output),
            Load(load) => load.serialize(output),
            Store(store) => store.serialize(output),
            Branch(branch) => branch.serialize(output),
            BranchTarget(branch_target) => branch_target.serialize(output),
            CallId(call_id) => call_id.serialize(output),
        }
    }
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
            if let Some(opcode_with_type) = decode_opcode(values[index]) {
                let start = index + 1;
                let end = start + opcode_with_type.size();
                if end > values.len() {
                    break;
                }
                result.push(opcode_with_type.deserialize(&values[start..end]));
                index = end;
            } else {
                index += 1;
            }
        }
        result
    }
}

fn clampreg(value: u8) -> u8 {
    value % 32
}

impl ValueDeserializer<ImmediateOpcode> for Immediate {
    fn size() -> usize {
        4
    }

    fn deserialize(opcode: ImmediateOpcode, input: &[u8]) -> Immediate {
        Immediate {
            opcode,
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

impl ValueDeserializer<LoadOpcode> for Load {
    fn size() -> usize {
        4
    }
    fn deserialize(opcode: LoadOpcode, input: &[u8]) -> Load {
        Load {
            opcode,
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

impl ValueDeserializer<StoreOpcode> for Store {
    fn size() -> usize {
        4
    }
    fn deserialize(opcode: StoreOpcode, input: &[u8]) -> Self {
        Store {
            opcode,
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

impl ValueDeserializer<RegisterOpcode> for Register {
    fn size() -> usize {
        3
    }
    fn deserialize(opcode: RegisterOpcode, input: &[u8]) -> Self {
        Register {
            opcode,
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

impl ValueDeserializer<BranchOpcode> for Branch {
    fn size() -> usize {
        3
    }
    fn deserialize(opcode: BranchOpcode, input: &[u8]) -> Self {
        Branch {
            opcode,
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

impl ValueDeserializer<BranchTargetOpcode> for BranchTarget {
    fn size() -> usize {
        1
    }
    fn deserialize(opcode: BranchTargetOpcode, input: &[u8]) -> Self {
        BranchTarget {
            opcode,
            identifier: input[0],
        }
    }
}

impl ValueSerializer for BranchTarget {
    fn serialize(&self, output: &mut Vec<u8>) {
        output.push(self.identifier);
    }
}

impl ValueDeserializer<CallIdOpcode> for CallId {
    fn size() -> usize {
        2
    }
    fn deserialize(opcode: CallIdOpcode, input: &[u8]) -> Self {
        CallId {
            opcode,
            identifier: bytes_to_u16(&input[0..2]),
        }
    }
}

impl ValueSerializer for CallId {
    fn serialize(&self, output: &mut Vec<u8>) {
        output.extend(u16_to_bytes(self.identifier));
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
        let bytes = serializer.serialize(&[Instruction::Register(Register {
            opcode: RegisterOpcode::Add,
            rs1: 0,
            rs2: 1,
            rd: 2,
        })]);
        assert_eq!(bytes, vec![RegisterOpcode::Add.to_u8().unwrap(), 0, 1, 2]);
    }

    #[test]
    fn test_deserialize() {
        let serializer = Serializer::new();
        let bytes = serializer.serialize(&[Instruction::Register(Register {
            opcode: RegisterOpcode::Add,
            rs1: 0,
            rs2: 1,
            rd: 2,
        })]);
        let instructions = serializer.deserialize(&bytes);
        assert_eq!(instructions.len(), 1);
        assert_eq!(
            instructions[0],
            Instruction::Register(Register {
                opcode: RegisterOpcode::Add,
                rs1: 0,
                rs2: 1,
                rd: 2,
            })
        );
    }

    #[test]
    fn test_deserialize_all_types() {
        let serializer = Serializer::new();
        let instructions = [
            Instruction::Immediate(Immediate {
                opcode: ImmediateOpcode::Addi,
                value: 0,
                rs: 1,
                rd: 2,
            }),
            Instruction::Register(Register {
                opcode: RegisterOpcode::Add,
                rs1: 0,
                rs2: 1,
                rd: 2,
            }),
            Instruction::Load(Load {
                opcode: LoadOpcode::Lb,
                offset: 0,
                rs: 1,
                rd: 2,
            }),
            Instruction::Store(Store {
                opcode: StoreOpcode::Sb,
                offset: 0,
                rs: 1,
                rd: 2,
            }),
            Instruction::Branch(Branch {
                opcode: BranchOpcode::Beq,
                target: 0,
                rs1: 1,
                rs2: 2,
            }),
            Instruction::BranchTarget(BranchTarget {
                opcode: BranchTargetOpcode::Target,
                identifier: 0,
            }),
            Instruction::CallId(CallId {
                opcode: CallIdOpcode::Call,
                identifier: 0,
            }),
        ];

        let bytes = serializer.serialize(&instructions);
        let instructions_deserialized = serializer.deserialize(&bytes);
        assert_eq!(instructions_deserialized, instructions);
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
            Instruction::Immediate(Immediate {
                opcode: ImmediateOpcode::Addi,
                value: 10,
                rs: 11, // 43 clamped to 32
                rd: 0,
            })
        );
    }
}

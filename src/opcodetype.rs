use crate::lang::{
    Branch, BranchTarget, CallId, Immediate, Instruction, Load, Opcode, Register, Store,
};
use crate::serializer::ValueDeserializer;

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

#[derive(Debug)]
pub enum InstructionValue<'a> {
    Register(&'a Register),
    Immediate(&'a Immediate),
    Load(&'a Load),
    Store(&'a Store),
    Branch(&'a Branch),
    BranchTarget(&'a BranchTarget),
    Call(&'a CallId),
}

impl OpcodeType {
    pub fn size(&self) -> usize {
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

impl<'a> From<&'a Instruction> for InstructionValue<'a> {
    fn from(instruction: &'a Instruction) -> Self {
        use Instruction::*;
        match instruction {
            Addi(immediate) | Slti(immediate) | Sltiu(immediate) | Andi(immediate)
            | Ori(immediate) | Xori(immediate) | Slli(immediate) | Srli(immediate)
            | Srai(immediate) => InstructionValue::Immediate(immediate),
            Add(register) | Sub(register) | Slt(register) | Sltu(register) | And(register)
            | Or(register) | Xor(register) | Sll(register) | Srl(register) | Sra(register) => {
                InstructionValue::Register(register)
            }
            Lh(load) | Lbu(load) | Lb(load) => InstructionValue::Load(load),
            Sh(store) | Sb(store) => InstructionValue::Store(store),
            Beq(branch) => InstructionValue::Branch(branch),
            Target(branch_target) => InstructionValue::BranchTarget(branch_target),
            Call(call_id) => InstructionValue::Call(call_id),
        }
    }
}

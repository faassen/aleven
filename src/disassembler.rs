use crate::lang::Instruction;
use crate::lang::Opcode;
use crate::opcodetype::InstructionValue;

trait Disassembler {
    fn disassemble(&self) -> String;
}

impl Disassembler for Instruction {
    fn disassemble(&self) -> String {
        let opcode: Opcode = self.into();
        let opcode = opcode.to_string().to_ascii_lowercase();
        let instruction_value: InstructionValue = self.into();
        match instruction_value {
            InstructionValue::Immediate(immediate) => format!(
                "r{} = {} r{} {}",
                immediate.rd, opcode, immediate.rs, immediate.value
            ),
            InstructionValue::Register(register) => format!(
                "r{} = {} r{} r{}",
                register.rd, opcode, register.rs1, register.rs2
            ),
            InstructionValue::Load(load) => {
                format!("r{} = {} r{} r{}", load.rd, opcode, load.rs, load.offset)
            }
            InstructionValue::Store(store) => {
                format!("{} r{} {} = r{}", opcode, store.rd, store.offset, store.rs)
            }
            InstructionValue::Branch(branch) => {
                format!(
                    "{} r{} r{} {}",
                    opcode, branch.rs1, branch.rs2, branch.target
                )
            }
            InstructionValue::BranchTarget(branch_target) => {
                format!("{} {}", opcode, branch_target.identifier)
            }
            InstructionValue::Call(call_id) => format!("{} {}", opcode, call_id.identifier),
        }
    }
}

pub fn disassemble(instructions: &[Instruction]) -> String {
    instructions
        .iter()
        .map(|instruction| instruction.disassemble())
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::{Immediate, Register};

    #[test]
    fn test_disassemble() {
        let instructions = vec![
            Instruction::Add(Register {
                rd: 0,
                rs1: 1,
                rs2: 2,
            }),
            Instruction::Addi(Immediate {
                rd: 0,
                rs: 1,
                value: 10,
            }),
        ];

        let disassembled = disassemble(&instructions);
        assert_eq!(disassembled, "r0 = add r1 r2\nr0 = addi r1 10");
    }
}

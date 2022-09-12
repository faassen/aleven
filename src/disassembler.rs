use crate::lang::Instruction;

trait Disassembler {
    fn disassemble(&self) -> String;
}

impl Disassembler for Instruction {
    fn disassemble(&self) -> String {
        use Instruction::*;
        let opcode = self.opcode_str().to_lowercase();
        match self {
            Immediate(immediate) => format!(
                "r{} = {} r{} {}",
                immediate.rd, opcode, immediate.rs, immediate.value
            ),
            Register(register) => format!(
                "r{} = {} r{} r{}",
                register.rd, opcode, register.rs1, register.rs2
            ),
            Load(load) => {
                format!("r{} = {} r{} {}", load.rd, opcode, load.rs, load.offset)
            }
            Store(store) => {
                format!("{} r{} {} = r{}", opcode, store.rd, store.offset, store.rs)
            }
            Branch(branch) => {
                format!(
                    "{} r{} r{} t{}",
                    opcode, branch.rs1, branch.rs2, branch.target
                )
            }
            BranchTarget(branch_target) => {
                format!("{} t{}", opcode, branch_target.identifier)
            }
            CallId(call_id) => format!("{} f{}", opcode, call_id.identifier),
            Switch(switch) => format!(
                "{} r{} f{} {}",
                opcode, switch.rs, switch.identifier, switch.amount
            ),
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
    use crate::disassembler::disassemble;
    use crate::lang::{Immediate, ImmediateOpcode, Register, RegisterOpcode};

    #[test]
    fn test_disassemble() {
        let instructions = vec![
            Instruction::Register(Register {
                opcode: RegisterOpcode::Add,
                rd: 0,
                rs1: 1,
                rs2: 2,
            }),
            Instruction::Immediate(Immediate {
                opcode: ImmediateOpcode::Addi,
                rd: 0,
                rs: 1,
                value: 10,
            }),
        ];

        let disassembled = disassemble(&instructions);
        assert_eq!(disassembled, "r0 = add r1 r2\nr0 = addi r1 10");
    }
}

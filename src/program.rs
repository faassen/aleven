use crate::lang::Instruction;
use crate::lang::Processor;
use rustc_hash::FxHashMap;

pub struct Program {
    instructions: Vec<Instruction>,
}

impl Program {
    pub fn new(instructions: &[Instruction]) -> Program {
        Program {
            instructions: Program::cleanup(instructions),
        }
    }

    pub fn execute(&self, processor: &mut Processor, memory: &mut [u8]) {
        let targets = Program::targets(&self.instructions);
        processor.execute(&self.instructions, memory, &targets);
    }

    pub fn cleanup(instructions: &[Instruction]) -> Vec<Instruction> {
        // clean up program by removing branching instructions that don't have
        // targets or point to a target that's earlier
        let targets = Program::targets(instructions);
        let mut result = Vec::new();

        for (index, instruction) in instructions.iter().enumerate() {
            match instruction {
                Instruction::Beq(branch) => {
                    let target = branch.target;
                    let target_index = targets.get(&target);
                    if let Some(target_index) = target_index {
                        if *target_index > index {
                            result.push(instruction.clone());
                        }
                    }
                }
                _ => {
                    result.push(instruction.clone());
                }
            }
        }
        result
    }

    fn targets(instructions: &[Instruction]) -> FxHashMap<u8, usize> {
        let mut targets = FxHashMap::default();
        for (index, instruction) in instructions.iter().enumerate() {
            if let Instruction::Target(target) = instruction {
                targets.insert(target.identifier, index);
            }
        }
        targets
    }
}

use crate::lang::Instruction;
use crate::lang::{BranchTarget, BranchTargetOpcode, CallId, CallIdOpcode, Processor};
use crate::llvm::CodeGen;
use crate::llvm::ProgramFunc;
use inkwell::execution_engine::JitFunction;
use inkwell::values::FunctionValue;
use rustc_hash::FxHashMap;
use rustc_hash::FxHashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    name: String,
    instructions: Vec<Instruction>,
    repeat: u8,
}

impl Function {
    pub fn new(name: String, instructions: &[Instruction], repeat: u8) -> Function {
        Function {
            name,
            instructions: Function::cleanup_branches(instructions),
            repeat,
        }
    }

    pub fn interpret(&self, memory: &mut [u8], processor: &mut Processor, functions: &[Function]) {
        let targets = Function::targets(&self.instructions);
        let repeat = self.get_repeat();
        for _i in 0..repeat {
            processor.execute(&self.instructions, memory, &targets, functions);
        }
    }

    pub fn compile<'ctx>(
        &self,
        id: usize,
        codegen: &'ctx CodeGen,
        memory_len: u16,
        functions: &FxHashMap<u16, FunctionValue<'ctx>>,
    ) -> FunctionValue<'ctx> {
        codegen.compile_function(
            id,
            self.get_repeat(),
            &self.instructions,
            memory_len,
            functions,
        )
    }

    pub fn compile_as_program<'ctx>(
        &self,
        codegen: &'ctx CodeGen,
        memory_len: u16,
    ) -> JitFunction<'ctx, ProgramFunc> {
        let inner_function = self.compile(0, codegen, memory_len, &FxHashMap::default());
        let mut functions = FxHashMap::default();
        functions.insert(0, inner_function);
        // put in program id 0 as this function is only used for testing purposes
        let llvm_program = codegen
            .compile_program(0, &functions)
            .expect("Unable to JIT compile `program`");
        llvm_program
    }

    pub fn run(func: &JitFunction<ProgramFunc>, memory: &mut [u8]) {
        unsafe {
            func.call(memory.as_mut_ptr());
        }
    }

    pub fn get_instructions(&self) -> &[Instruction] {
        &self.instructions
    }

    pub fn get_repeat(&self) -> u8 {
        if self.repeat > 0 {
            self.repeat
        } else {
            1
        }
    }

    pub fn get_call_ids(&self) -> impl Iterator<Item = u16> + '_ {
        self.instructions
            .iter()
            .filter_map(|instruction| match instruction {
                Instruction::CallId(CallId {
                    opcode: CallIdOpcode::Call,
                    identifier,
                }) => Some(*identifier),
                _ => None,
            })
    }

    pub fn get_call_id_set(&self) -> FxHashSet<u16> {
        self.get_call_ids().collect::<FxHashSet<u16>>()
    }

    fn cleanup_branches(instructions: &[Instruction]) -> Vec<Instruction> {
        // clean up program by removing branching that point to a target that's earlier
        // for branches that don't have a target, a synthetic target at the end is
        // jumped to instead
        let targets = Function::targets(instructions);
        let mut result = Vec::new();

        let unique_target = Self::get_unique_target(&targets);

        for (index, instruction) in instructions.iter().enumerate() {
            match instruction {
                Instruction::Branch(branch) => {
                    let target = branch.target;
                    let target_index = targets.get(&target);
                    if let Some(target_index) = target_index {
                        if *target_index > index {
                            result.push(instruction.clone());
                        }
                    } else if let Some(unique_target_index) = unique_target {
                        let mut retargeted_branch = branch.clone();
                        retargeted_branch.target = unique_target_index;
                        result.push(Instruction::Branch(retargeted_branch));
                    }
                }
                _ => {
                    result.push(instruction.clone());
                }
            }
        }
        if let Some(unique_target_index) = unique_target {
            result.push(Instruction::BranchTarget(BranchTarget {
                opcode: BranchTargetOpcode::Target,
                identifier: unique_target_index,
            }));
        }
        result
    }

    pub fn cleanup_calls(&self, functions: &[Function], seen: &FxHashSet<u16>) -> Function {
        let mut new_instructions = Vec::new();
        for instruction in self.instructions.iter() {
            match instruction {
                Instruction::CallId(CallId {
                    opcode: CallIdOpcode::Call,
                    identifier,
                }) => {
                    if !seen.contains(identifier) {
                        let identifier = *identifier as usize;
                        if identifier >= functions.len() {
                            continue;
                        }
                        new_instructions.push(instruction.clone());
                    }
                }
                _ => {
                    new_instructions.push(instruction.clone());
                }
            }
        }
        Function {
            name: self.name.clone(),
            instructions: new_instructions,
            repeat: self.repeat,
        }
    }

    fn targets(instructions: &[Instruction]) -> FxHashMap<u8, usize> {
        let mut targets = FxHashMap::default();
        for (index, instruction) in instructions.iter().enumerate() {
            if let Instruction::BranchTarget(BranchTarget {
                opcode: _,
                identifier,
            }) = instruction
            {
                targets.insert(*identifier, index);
            }
        }
        targets
    }

    fn get_unique_target(targets: &FxHashMap<u8, usize>) -> Option<u8> {
        let mut index: u8 = 0;
        loop {
            if targets.get(&index).is_none() {
                return Some(index);
            }
            if index == 255 {
                return None;
            }
            index += 1;
        }
    }

    pub fn restrict_call_budget<F>(&self, f: &mut F) -> Function
    where
        F: FnMut(u16) -> bool,
    {
        let mut new_instructions = Vec::new();
        for instruction in self.instructions.iter() {
            match instruction {
                Instruction::CallId(CallId {
                    opcode: CallIdOpcode::Call,
                    identifier,
                }) => {
                    if f(*identifier) {
                        new_instructions.push(instruction.clone());
                    }
                }
                _ => {
                    new_instructions.push(instruction.clone());
                }
            }
        }
        Function {
            name: self.name.clone(),
            instructions: new_instructions,
            repeat: self.repeat,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::lang::{Branch, BranchOpcode};

    #[test]
    fn test_non_existent_branch_target() {
        let function = Function::new(
            "foo".to_string(),
            &[Instruction::Branch(Branch {
                opcode: BranchOpcode::Beq,
                rs1: 0,
                rs2: 1,
                target: 100, // does not exist, so will go to end
            })],
            0,
        );

        assert_eq!(
            function,
            Function::new(
                "foo".to_string(),
                &[Instruction::Branch(Branch {
                    opcode: BranchOpcode::Beq,
                    rs1: 0,
                    rs2: 1,
                    target: 0
                }),],
                0
            )
        );
    }
}

use crate::lang::Instruction;
use crate::lang::{Branch, BranchTarget, Processor};
use crate::llvm::CodeGen;
use crate::llvm::ProgramFunc;
use inkwell::execution_engine::JitFunction;
use inkwell::values::FunctionValue;
use rustc_hash::FxHashMap;
use rustc_hash::FxHashSet;

#[derive(Debug, PartialEq, Eq)]
pub struct Function {
    instructions: Vec<Instruction>,
}

impl Function {
    pub fn new(instructions: &[Instruction]) -> Function {
        Function {
            instructions: Function::cleanup_branches(instructions),
        }
    }

    pub fn interpret(&self, memory: &mut [u8], processor: &mut Processor, functions: &[Function]) {
        let targets = Function::targets(&self.instructions);
        processor.execute(&self.instructions, memory, &targets, functions);
    }

    pub fn compile<'ctx>(
        &self,
        id: usize,
        codegen: &'ctx CodeGen,
        memory_len: u16,
        functions: &FxHashMap<u16, FunctionValue<'ctx>>,
    ) -> FunctionValue<'ctx> {
        codegen.compile_function(id, &self.instructions, memory_len, functions)
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

    pub fn get_call_ids(&self) -> FxHashSet<u16> {
        self.instructions
            .iter()
            .filter_map(|instruction| match instruction {
                Instruction::Call(call) => Some(call.identifier),
                _ => None,
            })
            .collect::<FxHashSet<u16>>()
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
                Instruction::Beq(branch) => {
                    let target = branch.target;
                    let target_index = targets.get(&target);
                    if let Some(target_index) = target_index {
                        if *target_index > index {
                            result.push(instruction.clone());
                        }
                    } else if let Some(unique_target_index) = unique_target {
                        result.push(Instruction::Beq(Branch {
                            rs1: branch.rs1,
                            rs2: branch.rs2,
                            target: unique_target_index,
                        }))
                    }
                }
                _ => {
                    result.push(instruction.clone());
                }
            }
        }
        if let Some(unique_target_index) = unique_target {
            result.push(Instruction::Target(BranchTarget {
                identifier: unique_target_index,
            }));
        }
        result
    }

    pub fn cleanup_calls(&self, functions: &[Function], seen: &FxHashSet<u16>) -> Function {
        let mut new_instructions = Vec::new();
        for instruction in self.instructions.iter() {
            match instruction {
                Instruction::Call(call) => {
                    if !seen.contains(&call.identifier) {
                        let identifier = call.identifier as usize;
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
        Function::new(&new_instructions)
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
}

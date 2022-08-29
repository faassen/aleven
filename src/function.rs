use crate::lang::Instruction;
use crate::lang::Processor;
use crate::llvm::CodeGen;
use crate::llvm::ProgramFunc;
use inkwell::execution_engine::JitFunction;
use rustc_hash::FxHashMap;

#[derive(Debug)]
pub struct Function {
    instructions: Vec<Instruction>,
}

impl Function {
    pub fn new(instructions: &[Instruction]) -> Function {
        Function {
            instructions: Function::cleanup(instructions),
        }
    }

    pub fn interpret(&self, memory: &mut [u8]) {
        let mut processor = Processor::new();
        let targets = Function::targets(&self.instructions);
        processor.execute(&self.instructions, memory, &targets);
    }

    pub fn compile<'ctx>(
        &self,
        codegen: &'ctx CodeGen,
        memory_len: u16,
    ) -> JitFunction<'ctx, ProgramFunc> {
        let llvm_program = codegen
            .compile_program(&self.instructions, memory_len)
            .expect("Unable to JIT compile `program`");
        llvm_program
    }

    pub fn run(func: JitFunction<ProgramFunc>, memory: &mut [u8]) {
        unsafe {
            func.call(memory.as_mut_ptr());
        }
    }

    pub fn get_instructions(&self) -> &[Instruction] {
        &self.instructions
    }

    fn cleanup(instructions: &[Instruction]) -> Vec<Instruction> {
        // clean up program by removing branching instructions that don't have
        // targets or point to a target that's earlier
        let targets = Function::targets(instructions);
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
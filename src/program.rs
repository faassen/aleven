use inkwell::execution_engine::JitFunction;

use crate::function::Function;
use crate::lang::{Instruction, Processor};
use crate::llvm::ProgramFunc;
// use crate::CodeGen;

pub struct Program {
    functions: Vec<Function>,
}

impl Program {
    pub fn new(functions: &[&[Instruction]]) -> Program {
        Program {
            functions: functions
                .iter()
                .map(|instructions| Function::new(instructions))
                .collect(),
        }
    }

    pub fn from_instructions(instructions: &[Instruction]) -> Program {
        Program::new(&[instructions])
    }

    pub fn interpret(&self, memory: &mut [u8]) {
        let mut processor = Processor::new();
        self.interpret_with_processor(memory, &mut processor);
    }

    pub fn interpret_with_processor(&self, memory: &mut [u8], processor: &mut Processor) {
        self.call(memory, processor, 0);
    }

    pub fn call(&self, memory: &mut [u8], processor: &mut Processor, id: usize) {
        self.functions[id].interpret(memory, processor, &self.functions);
    }

    // pub fn compile_function(id: u16, function: &Function, codegen: &CodeGen) {
    //     function.compile(codegen, memory_len)
    // }

    // pub fn compile(&self, codegen: &CodeGen, memory_len: u16) -> JitFunction<ProgramFunc> {
    //     codegen
    //         .compile_program(&self.functions, memory_len)
    //         .expect("Unable to JIT compile `program`")
    // }
}

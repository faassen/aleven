use crate::cache::FunctionValueCache;
use crate::function::Function;
use crate::lang::{Instruction, Processor};
use crate::llvm::CodeGen;
use crate::llvm::ProgramFunc;
use inkwell::execution_engine::JitFunction;
use inkwell::values::FunctionValue;
use rustc_hash::{FxHashMap, FxHashSet};

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

    pub fn get_function(&self, id: u16) -> &Function {
        &self.functions[id as usize]
    }

    pub fn call(&self, memory: &mut [u8], processor: &mut Processor, id: usize) {
        self.functions[id].interpret(memory, processor, &self.functions);
    }

    pub fn compile<'ctx>(
        &'ctx self,
        codegen: &'ctx CodeGen,
        memory_size: u16,
        cache: &mut FunctionValueCache<'ctx>,
    ) -> JitFunction<ProgramFunc> {
        let dependency_map = cache.compile(0, &self, codegen, memory_size);
        codegen.compile_program(&dependency_map).unwrap()
    }
}

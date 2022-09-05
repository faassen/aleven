use crate::cache::FunctionValueCache;
use crate::function::Function;
use crate::lang::Instruction;
use crate::llvm::CodeGen;
use crate::program::Program;
use inkwell::context::Context;

pub type RunnerFunc = fn(&[Instruction], &mut [u8]);
pub type RunnerProgram = fn(&[&[Instruction]], &mut [u8]);

pub fn run_interpreter_program(funcs: &[&[Instruction]], memory: &mut [u8]) {
    let program = Program::new(funcs);
    program.interpret(memory);
}

pub fn run_llvm_program(funcs: &[&[Instruction]], memory: &mut [u8]) {
    let program = Program::new(funcs);
    let context = Context::create();
    let codegen = CodeGen::new(&context);
    let mut cache = FunctionValueCache::new();
    let func = program.compile(0, &codegen, memory.len() as u16, &mut cache);
    codegen.module.verify().unwrap();
    Function::run(&func, memory);
}

pub fn run_llvm_func(instructions: &[Instruction], memory: &mut [u8]) {
    run_llvm_program(&[instructions], memory);
}

pub fn run_interpreter_func(instructions: &[Instruction], memory: &mut [u8]) {
    run_interpreter_program(&[instructions], memory);
}

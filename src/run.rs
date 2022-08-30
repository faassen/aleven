use crate::cache::FunctionValueCache;
use crate::function::Function;
use crate::lang::Instruction;
use crate::llvm::CodeGen;
use crate::program::Program;
use inkwell::context::Context;

pub type RunnerFunc = fn(&[Instruction], &mut [u8]);
pub type RunnerProgram = fn(&[&[Instruction]], &mut [u8]);

pub fn run_llvm_func(instructions: &[Instruction], memory: &mut [u8]) {
    let function = Function::new(instructions);
    let context = Context::create();
    let codegen = CodeGen::new(&context);
    let func = function.compile_as_program(&codegen, memory.len() as u16);
    codegen.module.verify().unwrap();
    Function::run(func, memory);
}

pub fn run_interpreter_func(instructions: &[Instruction], memory: &mut [u8]) {
    Program::from_instructions(instructions).interpret(memory);
}

pub fn run_interpreter_program(funcs: &[&[Instruction]], memory: &mut [u8]) {
    let program = Program::new(funcs);
    program.interpret(memory);
}

pub fn run_llvm_program(funcs: &[&[Instruction]], memory: &mut [u8]) {
    let program = Program::new(funcs);
    let context = Context::create();
    let codegen = CodeGen::new(&context);
    let mut cache = FunctionValueCache::new();
    let func = program.compile(&codegen, memory.len() as u16, &mut cache);
    codegen.module.verify().unwrap();
    Function::run(func, memory);
}

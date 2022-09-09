use crate::cache::FunctionValueCache;
use crate::function::Function;
use crate::lang::Instruction;
use crate::llvm::CodeGen;
use crate::program::Program;
use inkwell::context::Context;

pub type Run = fn(&Program, &mut [u8]);
pub type Runner = fn(&[(u8, &[Instruction])], &mut [u8]);
pub type RunnerFunc = fn(&[Instruction], &mut [u8]);
pub type RunnerProgram = fn(&[&[Instruction]], &mut [u8]);

pub fn interpreted(program: &Program, memory: &mut [u8]) {
    program.interpret(memory);
}

pub fn compiled(program: &Program, memory: &mut [u8]) {
    let context = Context::create();
    let codegen = CodeGen::new(&context);
    let mut cache = FunctionValueCache::new();
    let func = program.compile(0, &codegen, memory.len() as u16, &mut cache);
    codegen.module.verify().unwrap();
    Function::run(&func, memory);
}

pub fn run_interpreter(funcs: &[(u8, &[Instruction])], memory: &mut [u8]) {
    let program = Program::new(funcs);
    program.interpret(memory);
}

pub fn run_llvm(funcs: &[(u8, &[Instruction])], memory: &mut [u8]) {
    let program = Program::new(funcs);
    let context = Context::create();
    let codegen = CodeGen::new(&context);
    let mut cache = FunctionValueCache::new();
    let func = program.compile(0, &codegen, memory.len() as u16, &mut cache);
    codegen.module.verify().unwrap();
    Function::run(&func, memory);
}

fn repeat_0<'a>(funcs: &'a [&'a [Instruction]]) -> Vec<(u8, &'a [Instruction])> {
    funcs.iter().map(|f| (0, *f)).collect()
}

pub fn run_interpreter_program(funcs: &[&[Instruction]], memory: &mut [u8]) {
    run_interpreter(&repeat_0(funcs), memory);
}

pub fn run_llvm_program(funcs: &[&[Instruction]], memory: &mut [u8]) {
    run_llvm(&repeat_0(funcs), memory);
}

pub fn run_llvm_func(instructions: &[Instruction], memory: &mut [u8]) {
    run_llvm_program(&[instructions], memory);
}

pub fn run_interpreter_func(instructions: &[Instruction], memory: &mut [u8]) {
    run_interpreter_program(&[instructions], memory);
}

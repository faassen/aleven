use aleven::CodeGen;
use aleven::Function;
use aleven::Instruction;
use aleven::Program;
use inkwell::context::Context;

pub type Runner = fn(&[Instruction], &mut [u8]);

pub fn run_llvm(instructions: &[Instruction], memory: &mut [u8]) {
    let function = Function::new(instructions);
    let context = Context::create();
    let codegen = CodeGen::new(&context);
    let func = function.compile(&codegen, memory.len() as u16);
    codegen.module.verify().unwrap();
    Function::run(func, memory);
}

pub fn run_interpreter(instructions: &[Instruction], memory: &mut [u8]) {
    Program::from_instructions(instructions).interpret(memory);
}

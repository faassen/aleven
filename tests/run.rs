use aleven::CodeGen;
use aleven::Instruction;
use aleven::Program;
use inkwell::context::Context;

pub type Runner = fn(&[Instruction], &mut [u8]);

pub fn run_llvm(instructions: &[Instruction], memory: &mut [u8]) {
    let program = Program::new(instructions);
    let context = Context::create();
    let codegen = CodeGen::new(&context);
    let func = program.compile(&codegen, memory.len() as u16);
    codegen.module.verify().unwrap();
    Program::run(func, memory);
}

pub fn run_interpreter(instructions: &[Instruction], memory: &mut [u8]) {
    Program::new(instructions).interpret(memory);
}

#![no_main]
extern crate aleven;
use aleven::Assembler;
use aleven::CodeGen;
use aleven::Program;
use inkwell::context::Context;
use inkwell::OptimizationLevel;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let assembler = Assembler::new();
    let instructions = assembler.disassemble(data);

    let program = Program::new(&instructions);

    let context = Context::create();
    let codegen = CodeGen::new(&context);

    let func = program.compile(&codegen, data.len() as u16);
    codegen.module.verify().unwrap();
    let mut memory = data.to_vec();
    Program::run(func, &mut memory);
});

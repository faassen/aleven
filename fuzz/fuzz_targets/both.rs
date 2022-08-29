#![no_main]
extern crate aleven;
use aleven::Assembler;
use aleven::CodeGen;
use aleven::Function;
use inkwell::context::Context;
use inkwell::OptimizationLevel;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let assembler = Assembler::new();
    let instructions = assembler.disassemble(data);

    let function = Function::new(&instructions);

    let mut memory_llvm = data.to_vec();
    let mut memory_interpreter = data.to_vec();

    let context = Context::create();
    let codegen = CodeGen::new(&context);
    let func = function.compile(&codegen, data.len() as u16);
    codegen.module.verify().unwrap();
    Function::run(func, &mut memory_llvm);

    function.interpret(&mut memory_interpreter);

    // the effect should be the same
    assert_eq!(memory_llvm, memory_interpreter);
});

#![no_main]
extern crate aleven;
use aleven::CodeGen;
use aleven::Function;
use aleven::FunctionValueCache;
use aleven::Program;
use aleven::Serializer;
use inkwell::context::Context;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let serializer = Serializer::new();
    let instructions = serializer.deserialize(data);
    let mut memory_llvm = data.to_vec();
    let mut memory_interpreter = data.to_vec();

    let context = Context::create();
    let codegen = CodeGen::new(&context);

    let program = Program::from_instructions(&instructions);
    let func = program.compile(
        0,
        &codegen,
        memory_llvm.len() as u16,
        &mut FunctionValueCache::new(),
    );
    codegen.module.verify().unwrap();
    Function::run(&func, &mut memory_llvm);

    Program::from_instructions(&instructions).interpret(&mut memory_interpreter);

    // the effect should be the same
    assert_eq!(memory_llvm, memory_interpreter);
});

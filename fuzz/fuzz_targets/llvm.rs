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

    let context = Context::create();
    let codegen = CodeGen::new(&context);

    let program = Program::from_instructions(&instructions);
    let mut memory = data.to_vec();
    let func = program.compile(
        0,
        &codegen,
        memory.len() as u16,
        &mut FunctionValueCache::new(),
    );
    codegen.module.verify().unwrap();

    Function::run(&func, &mut memory);
});

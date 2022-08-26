#![no_main]
extern crate aleven;
use aleven::assemble::Assembler;
use aleven::lang::Program;
use aleven::llvm::CodeGen;
use inkwell::context::Context;
use inkwell::OptimizationLevel;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let assembler = Assembler::new();
    let instructions = assembler.disassemble(data);
    let instructions = Program::cleanup(&instructions);

    let context = Context::create();
    let module = context.create_module("program");
    let execution_engine = module
        .create_jit_execution_engine(OptimizationLevel::None)
        .expect("Execution engine couldn't be built");
    let codegen = CodeGen {
        context: &context,
        module,
        builder: context.create_builder(),
        execution_engine,
    };
    let program = codegen
        .compile_program(&instructions, data.len() as u16)
        .expect("Unable to JIT compile `program`");
    codegen.module.verify().unwrap();
    let mut memory = data.to_vec();
    unsafe {
        program.call(memory.as_mut_ptr());
    }
});

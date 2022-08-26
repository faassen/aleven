#![no_main]
extern crate aleven;
use aleven::assemble::Assembler;
use aleven::lang::{Processor, Program};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let assembler = Assembler::new();
    let instructions = assembler.disassemble(data);
    let mut processor = Processor::new();
    let mut memory = data.to_vec();
    Program::new(&instructions).execute(&mut processor, &mut memory);
});

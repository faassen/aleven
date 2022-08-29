#![no_main]
extern crate aleven;
use aleven::Assembler;
use aleven::Function;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let assembler = Assembler::new();
    let instructions = assembler.disassemble(data);
    let mut memory = data.to_vec();
    Function::new(&instructions).interpret(&mut memory);
});

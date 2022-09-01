#![no_main]
extern crate aleven;
use aleven::Program;
use aleven::Serializer;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let serializer = Serializer::new();
    let instructions = serializer.deserialize(data);
    let mut memory = data.to_vec();
    Program::from_instructions(&instructions).interpret(&mut memory);
});

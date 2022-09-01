#![no_main]
extern crate aleven;
use aleven::Serializer;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let serializer = Serializer::new();
    let instructions = serializer.deserialize(data);
    let text = aleven::disassemble(&instructions);
    let parsed = aleven::parse(&text).unwrap();
    assert_eq!(instructions, parsed);
});

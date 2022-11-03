use aleven::parse_program;
use aleven::run::{compiled, interpreted, Run};
use parameterized::parameterized;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

#[parameterized(run={compiled, interpreted})]
fn test_stackmachine(run: Run) {
    let f = File::open("stackmachine.ale").unwrap();
    let mut reader = BufReader::new(f);
    let mut buffer = String::new();
    reader.read_to_string(&mut buffer).unwrap();
    let program = parse_program(&buffer).unwrap();

    let mut memory = [0u8; 1024];
    memory[0] = 4;
    memory[1] = 0; // program start
    memory[2] = 200;
    memory[3] = 0; // one below stack start

    //program
    memory[4] = 1;
    memory[5] = 1;
    memory[6] = 3; // add
    memory[7] = 1;
    memory[8] = 3;

    run(&program, &mut memory);

    // 400 as 16 bit numbers, little endian
    assert_eq!(memory[400], 3);
}

use aleven::{Immediate, Instruction, Store};
use parameterized::parameterized;

mod run;

use run::{run_interpreter, run_llvm, Runner};

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_addi_basic(runner: Runner) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 33,
            rs: 1,
            rd: 2,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 2,
            rd: 3, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 33);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_addi_register_has_value(runner: Runner) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 10,
            rs: 1,
            rd: 1,
        }),
        Instruction::Addi(Immediate {
            value: 33,
            rs: 1,
            rd: 2,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 2,
            rd: 3, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 43);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_addi_register_rs_is_rd(runner: Runner) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 10,
            rs: 1,
            rd: 1,
        }),
        Instruction::Addi(Immediate {
            value: 33,
            rs: 1,
            rd: 1,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 1,
            rd: 2, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 43);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_addi_register_dec(runner: Runner) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 10,
            rs: 1,
            rd: 1,
        }),
        Instruction::Addi(Immediate {
            value: -1,
            rs: 1,
            rd: 1,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 1,
            rd: 2, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 9);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_slti_less(runner: Runner) {
    let instructions = [
        Instruction::Slti(Immediate {
            value: 5,
            rs: 1,
            rd: 2,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 2,
            rd: 3, // defaults to 0
        }),
    ];
    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 1);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_slti_less_negative(runner: Runner) {
    let instructions = [
        Instruction::Slti(Immediate {
            value: -4,
            rs: 1,
            rd: 2,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 2,
            rd: 3, // defaults to 0
        }),
    ];
    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_sltiu_less(runner: Runner) {
    let instructions = [
        Instruction::Sltiu(Immediate {
            value: -4, // treated as large number instead
            rs: 1,
            rd: 2,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 2,
            rd: 3, // defaults to 0
        }),
    ];
    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 1);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_slti_equal(runner: Runner) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 5,
            rs: 1,
            rd: 1,
        }),
        Instruction::Slti(Immediate {
            value: 5,
            rs: 1,
            rd: 2,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 2,
            rd: 3, // defaults to 0
        }),
    ];
    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_slti_greater(runner: Runner) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 6,
            rs: 1,
            rd: 1,
        }),
        Instruction::Slti(Immediate {
            value: 5,
            rs: 1,
            rd: 2,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 2,
            rd: 3, // defaults to 0
        }),
    ];
    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_andi(runner: Runner) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 0b1010101,
            rs: 1,
            rd: 1,
        }),
        Instruction::Andi(Immediate {
            value: 0b1111110,
            rs: 1,
            rd: 2,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 2,
            rd: 3, // defaults to 0
        }),
    ];
    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0b1010100);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_ori(runner: Runner) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 0b1010100,
            rs: 1,
            rd: 1,
        }),
        Instruction::Ori(Immediate {
            value: 0b1111110,
            rs: 1,
            rd: 2,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 2,
            rd: 3, // defaults to 0
        }),
    ];
    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0b1111110);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_xori(runner: Runner) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 0b1010100,
            rs: 1,
            rd: 1,
        }),
        Instruction::Xori(Immediate {
            value: 0b1111010,
            rs: 1,
            rd: 2,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 2,
            rd: 3, // defaults to 0
        }),
    ];
    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0b0101110);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_slli(runner: Runner) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 5,
            rs: 1,
            rd: 1,
        }),
        Instruction::Slli(Immediate {
            value: 2,
            rs: 1,
            rd: 2,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 2,
            rd: 3, // defaults to 0
        }),
    ];
    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 20);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_srai(runner: Runner) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 20,
            rs: 1,
            rd: 1,
        }),
        Instruction::Srai(Immediate {
            value: 2,
            rs: 1,
            rd: 2,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 2,
            rd: 3, // defaults to 0
        }),
    ];
    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 5);
}

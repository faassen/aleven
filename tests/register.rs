use aleven::{Immediate, Instruction, Register, Store};
use byteorder::{ByteOrder, LittleEndian};
use parameterized::parameterized;

mod run;

use run::{run_interpreter_func, run_llvm_func, RunnerFunc};

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_add(runner: RunnerFunc) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 33,
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: 44,
            rs: 1,
            rd: 3,
        }),
        Instruction::Add(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 77);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_add_negative(runner: RunnerFunc) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 33,
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: -11,
            rs: 1,
            rd: 3,
        }),
        Instruction::Add(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 22);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_sub(runner: RunnerFunc) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 33,
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: 11,
            rs: 1,
            rd: 3,
        }),
        Instruction::Sub(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 22);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_add_wrapping(runner: RunnerFunc) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: i16::MAX,
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: 1,
            rs: 1,
            rd: 3,
        }),
        Instruction::Add(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sh(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    let value = LittleEndian::read_i16(&memory[20..]);
    assert_eq!(value, i16::MIN);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_add_sh(runner: RunnerFunc) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 255,
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: 255,
            rs: 1,
            rd: 3,
        }),
        Instruction::Add(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sh(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    let value = LittleEndian::read_u16(&memory[20..]);
    assert_eq!(value, 255 * 2);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_slt_less(runner: RunnerFunc) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 33,
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: 44,
            rs: 1,
            rd: 3,
        }),
        Instruction::Slt(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 1);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_slt_less_negative(runner: RunnerFunc) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: -33,
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: 44,
            rs: 1,
            rd: 3,
        }),
        Instruction::Slt(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 1);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_slt_equal(runner: RunnerFunc) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 44,
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: 44,
            rs: 1,
            rd: 3,
        }),
        Instruction::Slt(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_slt_greater(runner: RunnerFunc) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 44,
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: 33,
            rs: 1,
            rd: 3,
        }),
        Instruction::Slt(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_sltu_less(runner: RunnerFunc) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 33,
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: 44,
            rs: 1,
            rd: 3,
        }),
        Instruction::Sltu(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 1);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_sltu_less_negative(runner: RunnerFunc) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: -33, // interpreted as big
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: 44,
            rs: 1,
            rd: 3,
        }),
        Instruction::Sltu(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_sltu_equal(runner: RunnerFunc) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 44,
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: 44,
            rs: 1,
            rd: 3,
        }),
        Instruction::Sltu(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_sltu_greater(runner: RunnerFunc) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 44,
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: 33,
            rs: 1,
            rd: 3,
        }),
        Instruction::Sltu(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_and(runner: RunnerFunc) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 0b1010101,
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: 0b1111110,
            rs: 1,
            rd: 3,
        }),
        Instruction::And(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0b1010100);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_or(runner: RunnerFunc) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 0b1010100,
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: 0b1111110,
            rs: 1,
            rd: 3,
        }),
        Instruction::Or(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0b1111110);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_xor(runner: RunnerFunc) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 0b1111010,
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: 0b1010100,
            rs: 1,
            rd: 3,
        }),
        Instruction::Xor(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0b0101110);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_sll(runner: RunnerFunc) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 0b101,
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: 2,
            rs: 1,
            rd: 3,
        }),
        Instruction::Sll(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0b10100);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_sll_shift_too_large(runner: RunnerFunc) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 0b101,
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: 100,
            rs: 1,
            rd: 3,
        }),
        Instruction::Sll(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0b101);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_srl(runner: RunnerFunc) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 0b10100,
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: 2,
            rs: 1,
            rd: 3,
        }),
        Instruction::Srl(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0b101);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_srl_too_large(runner: RunnerFunc) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 0b10100,
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: 100,
            rs: 1,
            rd: 3,
        }),
        Instruction::Srl(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0b10100);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_srl_negative(runner: RunnerFunc) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: -20,
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: 2,
            rs: 1,
            rd: 3,
        }),
        Instruction::Srl(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sh(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    let value = LittleEndian::read_i16(&memory[20..]);
    assert_eq!(value, 16379);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_sra(runner: RunnerFunc) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 0b10100,
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: 2,
            rs: 1,
            rd: 3,
        }),
        Instruction::Sra(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0b101);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_sra_negative(runner: RunnerFunc) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: -20,
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: 2,
            rs: 1,
            rd: 3,
        }),
        Instruction::Sra(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sh(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    let value = LittleEndian::read_i16(&memory[20..]);
    assert_eq!(value, -5);
}

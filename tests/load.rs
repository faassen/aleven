use aleven::{Immediate, Instruction, Load, Store};
use byteorder::{ByteOrder, LittleEndian};
use parameterized::parameterized;

mod run;

use run::{run_interpreter_func, run_llvm_func, RunnerFunc};

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_lb_in_bounds(runner: RunnerFunc) {
    let instructions = [
        Instruction::Lb(Load {
            offset: 0,
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
    memory[0] = 11;
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 11);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_lb_out_of_bounds_means_zero(runner: RunnerFunc) {
    let instructions = [
        Instruction::Lb(Load {
            offset: 65,
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
    memory[10] = 100;
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_lbu_out_of_bounds_means_nop(runner: RunnerFunc) {
    let instructions = [
        Instruction::Lbu(Load {
            offset: 65,
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

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_lh_sh(runner: RunnerFunc) {
    let instructions = [
        Instruction::Lh(Load {
            offset: 0,
            rs: 1,
            rd: 2,
        }),
        Instruction::Sh(Store {
            offset: 10,
            rs: 2,
            rd: 3, // defaults to 0
        }),
    ];
    let mut memory = [0u8; 64];
    memory[0] = 2;
    memory[1] = 1;
    runner(&instructions, &mut memory);
    // is at 20 as pointer type is 16 bits
    assert_eq!(memory[20], 2);
    assert_eq!(memory[21], 1);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_lh_aligns(runner: RunnerFunc) {
    let instructions = [
        Instruction::Lh(Load {
            offset: 1, // aligned back to 0
            rs: 1,
            rd: 2,
        }),
        Instruction::Sh(Store {
            offset: 10,
            rs: 2,
            rd: 3, // defaults to 0
        }),
    ];
    let mut memory = [0u8; 64];
    memory[2] = 2;
    memory[3] = 1;
    runner(&instructions, &mut memory);
    assert_eq!(memory[20], 2);
    assert_eq!(memory[21], 1);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_lh_out_of_bounds(runner: RunnerFunc) {
    let instructions = [
        Instruction::Lh(Load {
            offset: 65, // out of bounds
            rs: 1,
            rd: 2,
        }),
        Instruction::Sh(Store {
            offset: 10,
            rs: 2,
            rd: 3, // defaults to 0
        }),
    ];
    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory, [0u8; 64]);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_lb_sign_extends(runner: RunnerFunc) {
    let instructions = [
        Instruction::Lb(Load {
            offset: 0,
            rs: 1,
            rd: 2,
        }),
        Instruction::Sh(Store {
            offset: 10,
            rs: 2,
            rd: 3, // defaults to 0
        }),
    ];
    let mut memory = [0u8; 64];
    memory[0] = -4i8 as u8; // FFFC
    runner(&instructions, &mut memory);
    let value = LittleEndian::read_i16(&memory[20..]);
    assert_eq!(value, -4);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_lbu_zero_extends(runner: RunnerFunc) {
    let instructions = [
        Instruction::Lbu(Load {
            offset: 0,
            rs: 1,
            rd: 2,
        }),
        Instruction::Sh(Store {
            offset: 10,
            rs: 2,
            rd: 3, // defaults to 0
        }),
    ];
    let mut memory = [0u8; 64];
    memory[0] = -4i8 as u8; // FC
    runner(&instructions, &mut memory);
    let value = LittleEndian::read_i16(&memory[20..]);
    assert_eq!(value, 252);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_lb_sign_extends_with_sra(runner: RunnerFunc) {
    let instructions = [
        Instruction::Lb(Load {
            offset: 0,
            rs: 1,
            rd: 2,
        }),
        Instruction::Srai(Immediate {
            value: 2,
            rs: 2,
            rd: 2,
        }),
        Instruction::Sh(Store {
            offset: 10,
            rs: 2,
            rd: 3, // defaults to 0
        }),
    ];
    let mut memory = [0u8; 64];
    memory[0] = -4i8 as u8; // FFFC
    runner(&instructions, &mut memory);
    let value = LittleEndian::read_i16(&memory[20..]);
    assert_eq!(value, -1);
    assert_eq!(value, 0xFFFFu16 as i16);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_lbu_zero_extends_sra(runner: RunnerFunc) {
    let instructions = [
        Instruction::Lbu(Load {
            offset: 0,
            rs: 1,
            rd: 2,
        }),
        Instruction::Srai(Immediate {
            value: 2,
            rs: 2,
            rd: 2,
        }),
        Instruction::Sh(Store {
            offset: 10,
            rs: 2,
            rd: 3, // defaults to 0
        }),
    ];
    let mut memory = [0u8; 64];
    memory[0] = -4i8 as u8;
    // lbu interprets this as 252, or 0000011111100
    runner(&instructions, &mut memory);
    let value = LittleEndian::read_u16(&memory[20..]);
    assert_eq!(value, 63);
}

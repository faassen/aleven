use aleven::{Instruction, Load, Store};
use parameterized::parameterized;

mod run;

use run::{run_interpreter, run_llvm, Runner};

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_lb_in_bounds(runner: Runner) {
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

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_lb_out_of_bounds_means_zero(runner: Runner) {
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

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_lbu_out_of_bounds_means_nop(runner: Runner) {
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

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_lh_sh(runner: Runner) {
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

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_lh_aligns(runner: Runner) {
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

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_lh_out_of_bounds(runner: Runner) {
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

use aleven::{Instruction, Load, Store};

use parameterized::parameterized;

mod run;

use run::{run_interpreter_func, run_llvm_func, RunnerFunc};

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_sb_out_of_bounds(runner: RunnerFunc) {
    let instructions = [
        Instruction::Lb(Load {
            offset: 0,
            rs: 1,
            rd: 2,
        }),
        Instruction::Sb(Store {
            offset: 65,
            rs: 2,
            rd: 3, // defaults to 0
        }),
    ];
    let mut memory = [0u8; 64];
    memory[0] = 11;
    let expected = memory;
    runner(&instructions, &mut memory);
    assert_eq!(memory, expected);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_sh_aligns(runner: RunnerFunc) {
    let instructions = [
        Instruction::Lh(Load {
            offset: 0,
            rs: 1,
            rd: 2,
        }),
        // it's not possible to misalign this as it's already even
        Instruction::Sh(Store {
            offset: 11,
            rs: 2,
            rd: 3, // defaults to 0
        }),
    ];
    let mut memory = [0u8; 64];
    memory[0] = 2;
    memory[1] = 1;
    runner(&instructions, &mut memory);
    assert_eq!(memory[22], 2);
    assert_eq!(memory[23], 1);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_sh_out_of_bounds(runner: RunnerFunc) {
    let instructions = [
        Instruction::Lh(Load {
            offset: 0,
            rs: 1,
            rd: 2,
        }),
        // it's not possible to misalign this as it's already even
        Instruction::Sh(Store {
            offset: 32, // should be out of bounds as x 2
            rs: 2,
            rd: 3, // defaults to 0
        }),
    ];
    let mut memory = [0u8; 64];
    memory[0] = 2;
    memory[1] = 1;
    let expected = memory;
    runner(&instructions, &mut memory);
    assert_eq!(memory, expected);
}

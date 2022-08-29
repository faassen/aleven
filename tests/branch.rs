use aleven::run::{run_interpreter_func, run_llvm_func, RunnerFunc};
use aleven::{Branch, BranchTarget, Immediate, Instruction, Load, Store};
use parameterized::parameterized;

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_beq_simple(runner: RunnerFunc) {
    let instructions = [
        Instruction::Lb(Load {
            offset: 0,
            rs: 1,
            rd: 2,
        }),
        Instruction::Lb(Load {
            offset: 1,
            rs: 1,
            rd: 3,
        }),
        Instruction::Beq(Branch {
            rs1: 2,
            rs2: 3,
            target: 1,
        }),
        Instruction::Lb(Load {
            offset: 2,
            rs: 1,
            rd: 4,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
        Instruction::Target(BranchTarget { identifier: 1 }),
    ];

    let mut memory = [0u8; 64];
    memory[0] = 10;
    memory[1] = 10;
    memory[2] = 30;

    runner(&instructions, &mut memory);
    // branch happened, so no store
    assert_eq!(memory[10], 0);

    let mut memory = [0u8; 64];
    memory[0] = 10;
    memory[1] = 20;
    memory[2] = 30;

    runner(&instructions, &mut memory);
    // branch happened, so store of 30
    assert_eq!(memory[10], 30);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_beq_nonexistent_target_means_nop(runner: RunnerFunc) {
    let instructions = [
        Instruction::Lb(Load {
            offset: 0,
            rs: 1,
            rd: 2,
        }),
        Instruction::Lb(Load {
            offset: 1,
            rs: 1,
            rd: 3,
        }),
        Instruction::Beq(Branch {
            rs1: 2,
            rs2: 3,
            target: 2, // does not exist!
        }),
        Instruction::Lb(Load {
            offset: 2,
            rs: 1,
            rd: 4,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
        Instruction::Target(BranchTarget { identifier: 1 }),
    ];

    let mut memory = [0u8; 64];
    memory[0] = 10;
    memory[1] = 10;
    memory[2] = 30;

    runner(&instructions, &mut memory);
    // since noop, branch happened
    assert_eq!(memory[10], 30);

    // in the other case, it's the same noop, so store happens
    let mut memory = [0u8; 64];
    memory[0] = 10;
    memory[1] = 20;
    memory[2] = 30;
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 30);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_beq_earlier_target_means_nop(runner: RunnerFunc) {
    let instructions = [
        Instruction::Lb(Load {
            offset: 0,
            rs: 1,
            rd: 2,
        }),
        Instruction::Lb(Load {
            offset: 1,
            rs: 1,
            rd: 3,
        }),
        Instruction::Target(BranchTarget { identifier: 1 }),
        Instruction::Beq(Branch {
            rs1: 2,
            rs2: 3,
            target: 1, // exists, but before me
        }),
        Instruction::Lb(Load {
            offset: 2,
            rs: 1,
            rd: 4,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    memory[0] = 10;
    memory[1] = 10;
    memory[2] = 30;

    runner(&instructions, &mut memory);
    // since noop, branch happened
    assert_eq!(memory[10], 30);

    // in the other case, it's the same noop, so store happens
    let mut memory = [0u8; 64];
    memory[0] = 10;
    memory[1] = 20;
    memory[2] = 30;
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 30);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_addi_after_beq(runner: RunnerFunc) {
    use Instruction::*;
    let instructions = [
        Target(BranchTarget { identifier: 176 }),
        Lh(Load {
            offset: 8728,
            rs: 24,
            rd: 24,
        }),
        Beq(Branch {
            target: 255,
            rs1: 31,
            rs2: 31,
        }),
        Addi(Immediate {
            value: 6168,
            rs: 24,
            rd: 24,
        }),
        Target(BranchTarget { identifier: 255 }),
        Addi(Immediate {
            value: 0,
            rs: 24,
            rd: 24,
        }),
    ];
    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
}

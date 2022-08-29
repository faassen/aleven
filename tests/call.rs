use aleven::run::{run_interpreter_program, RunnerProgram};
use aleven::{CallId, Instruction, Load, Store};
use parameterized::parameterized;

#[parameterized(runner={run_interpreter_program})]
fn test_call(runner: RunnerProgram) {
    let main_instructions = [
        Instruction::Lb(Load {
            offset: 0,
            rs: 1,
            rd: 2,
        }),
        Instruction::Call(CallId { identifier: 1 }),
    ];

    let sub_instructions = [Instruction::Sb(Store {
        offset: 10,
        rs: 2,
        rd: 3, // defaults to 0
    })];
    let mut memory = [0u8; 64];
    memory[0] = 11;

    runner(&[&main_instructions, &sub_instructions], &mut memory);

    assert_eq!(memory[10], 11);
}

#[parameterized(runner={run_interpreter_program})]
fn test_nested_call(runner: RunnerProgram) {
    let main_instructions = [
        Instruction::Lb(Load {
            offset: 0,
            rs: 31,
            rd: 1,
        }),
        Instruction::Lb(Load {
            offset: 1,
            rs: 31,
            rd: 2,
        }),
        Instruction::Lb(Load {
            offset: 2,
            rs: 31,
            rd: 3,
        }),
        Instruction::Lb(Load {
            offset: 3,
            rs: 31,
            rd: 4,
        }),
        Instruction::Call(CallId { identifier: 1 }),
        Instruction::Sb(Store {
            offset: 13,
            rs: 4,
            rd: 31,
        }),
    ];

    let sub_instructions = [
        Instruction::Sb(Store {
            offset: 10,
            rs: 1,
            rd: 31,
        }),
        Instruction::Call(CallId { identifier: 2 }),
        Instruction::Sb(Store {
            offset: 12,
            rs: 3,
            rd: 31,
        }),
    ];

    let sub_sub_instructions = [Instruction::Sb(Store {
        offset: 11,
        rs: 2,
        rd: 31,
    })];

    let mut memory = [0u8; 64];
    memory[0] = 11;
    memory[1] = 12;
    memory[2] = 13;
    memory[3] = 14;

    runner(
        &[&main_instructions, &sub_instructions, &sub_sub_instructions],
        &mut memory,
    );

    assert_eq!(memory[10], 11);
    assert_eq!(memory[11], 12);
    assert_eq!(memory[12], 13);
    assert_eq!(memory[13], 14);
}

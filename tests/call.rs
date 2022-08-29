use aleven::{CallId, Instruction, Load, Store};
use parameterized::parameterized;

mod run;

use run::{run_interpreter_program, RunnerProgram};

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

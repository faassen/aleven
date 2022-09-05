use aleven::parse;
use aleven::run::{run_interpreter_program, run_llvm_program, RunnerProgram};
use parameterized::parameterized;

#[parameterized(runner={run_llvm_program, run_interpreter_program})]
fn test_call(runner: RunnerProgram) {
    let main_instructions = parse(
        "
    r2 = lb r1 0
    call 1",
    )
    .unwrap();
    let sub_instructions = parse("sb r3 10 = r2").unwrap();

    let mut memory = [0u8; 64];
    memory[0] = 11;

    runner(&[&main_instructions, &sub_instructions], &mut memory);

    assert_eq!(memory[10], 11);
}

#[parameterized(runner={run_llvm_program, run_interpreter_program})]
fn test_nested_call(runner: RunnerProgram) {
    let main_instructions = parse(
        "
    r1 = lb r31 0
    r2 = lb r31 1
    r3 = lb r31 2
    r4 = lb r31 3
    call 1
    sb r31 13 = r4",
    )
    .unwrap();

    let sub_instructions = parse(
        "
    sb r31 10 = r1
    call 2
    sb r31 12 = r3",
    )
    .unwrap();

    let sub_sub_instructions = parse("sb r31 11 = r2").unwrap();

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

#[parameterized(runner={run_interpreter_program})]
fn test_no_recursion_basic(runner: RunnerProgram) {
    let main_instructions = parse("call 0").unwrap();
    let mut memory = [0u8; 64];
    runner(&[&main_instructions], &mut memory);
}

#[parameterized(runner={run_interpreter_program})]
fn test_no_calls_into_unknown(runner: RunnerProgram) {
    let main_instructions = parse("call 1").unwrap();
    let mut memory = [0u8; 64];
    runner(&[&main_instructions], &mut memory);
}

use aleven::parse;
use aleven::run::{run_interpreter_repeat_func, run_llvm_repeat_func, RepeatRunnerFunc};
use parameterized::parameterized;

#[parameterized(runner={run_interpreter_repeat_func})]
fn test_repeat(runner: RepeatRunnerFunc) {
    let instructions = parse(
        "
    r1 = addi r4 30
    sb r2 0 = r1
    r2 = addi r2 1
    ",
    )
    .unwrap();

    let mut memory = [0u8; 64];

    runner(&instructions, &mut memory, 10);

    for value in memory.iter().take(10) {
        assert_eq!(*value, 30);
    }
    assert_eq!(memory[10], 0);
}

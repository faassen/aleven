use aleven::parse;
use aleven::run::{run_interpreter_func, run_llvm_func, RunnerFunc};
use parameterized::parameterized;

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_sb_out_of_bounds(runner: RunnerFunc) {
    let instructions = parse(
        "
    r2 = lb r1 0
    sb r3 65 = r2",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    memory[0] = 11;
    let expected = memory;
    runner(&instructions, &mut memory);
    assert_eq!(memory, expected);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_sh_aligns(runner: RunnerFunc) {
    let instructions = parse(
        "
    r2 = lh r1 0
    sh r3 11 = r2",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    memory[0] = 2;
    memory[1] = 1;
    runner(&instructions, &mut memory);
    assert_eq!(memory[22], 2);
    assert_eq!(memory[23], 1);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_sh_out_of_bounds(runner: RunnerFunc) {
    let instructions = parse(
        "
    r2 = lh r1 0
    sh r3 32 = r2",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    memory[0] = 2;
    memory[1] = 1;
    let expected = memory;
    runner(&instructions, &mut memory);
    assert_eq!(memory, expected);
}

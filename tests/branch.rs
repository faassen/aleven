use aleven::parse;
use aleven::run::{run_interpreter_func, run_llvm_func, RunnerFunc};
use parameterized::parameterized;

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_beq_simple(runner: RunnerFunc) {
    let instructions = parse(
        "
    r2 = lb r1 0
    r3 = lb r1 1
    beq r2 r3 1
    r4 = lb r1 2
    sb r5 10 = r4
    target 1
    ",
    )
    .unwrap();

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
fn test_beq_nonexistent_target_means_target_is_end(runner: RunnerFunc) {
    let instructions = parse(
        "
    r2 = lb r1 0
    r3 = lb r1 1
    beq r2 r3 2 # does not exist!
    r4 = lb r1 2
    sb r5 10 = r4
    target 1
    ",
    )
    .unwrap();

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
fn test_beq_earlier_target_means_nop(runner: RunnerFunc) {
    let instructions = parse(
        "
    r2 = lb r1 0
    r3 = lb r1 1
    target 1
    beq r2 r3 1 # exists but before me
    r4 = lb r1 2
    sb r5 10 = r4
    ",
    )
    .unwrap();

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
    let instructions = parse(
        "
    r24 = lh r24 8728
    beq r31 r31 255
    r24 = addi r24 6168
    target 255
    r24 = addi r24 0
    ",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_bne_simple(runner: RunnerFunc) {
    let instructions = parse(
        "
    r2 = lb r1 0
    r3 = lb r1 1
    bne r2 r3 1
    r4 = lb r1 2
    sb r5 10 = r4
    target 1
    ",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    memory[0] = 10;
    memory[1] = 15;
    memory[2] = 30;

    runner(&instructions, &mut memory);
    // branch happened, so no store
    assert_eq!(memory[10], 0);

    let mut memory = [0u8; 64];
    memory[0] = 10;
    memory[1] = 10;
    memory[2] = 30;

    runner(&instructions, &mut memory);
    // branch happened, so store of 30
    assert_eq!(memory[10], 30);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_blt_simple(runner: RunnerFunc) {
    let instructions = parse(
        "
    r2 = lb r1 0
    r3 = lb r1 1
    blt r2 r3 1
    r4 = lb r1 2
    sb r5 10 = r4
    target 1
    ",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    memory[0] = 10;
    memory[1] = 15;
    memory[2] = 30;

    runner(&instructions, &mut memory);
    // branch happened, so no store
    assert_eq!(memory[10], 0);

    let mut memory = [0u8; 64];
    memory[0] = 10;
    memory[1] = 10;
    memory[2] = 30;

    runner(&instructions, &mut memory);
    // branch happened, so store of 30
    assert_eq!(memory[10], 30);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_blt_negative(runner: RunnerFunc) {
    let instructions = parse(
        "
    r2 = lb r1 0
    r3 = lb r1 1
    blt r2 r3 1
    r4 = lb r1 2
    sb r5 10 = r4
    target 1
    ",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    memory[0] = -1_i8 as u8;
    memory[1] = 10;
    memory[2] = 30;

    runner(&instructions, &mut memory);
    // branch happened, so no store
    assert_eq!(memory[10], 0);

    let mut memory = [0u8; 64];
    memory[0] = 20;
    memory[1] = 10;
    memory[2] = 30;

    runner(&instructions, &mut memory);
    // branch did not happen, so store of 30
    assert_eq!(memory[10], 30);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_bltu_simple(runner: RunnerFunc) {
    let instructions = parse(
        "
    r2 = lb r1 0
    r3 = lb r1 1
    bltu r2 r3 1
    r4 = lb r1 2
    sb r5 10 = r4
    target 1
    ",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    memory[0] = 5;
    memory[1] = 10;
    memory[2] = 30;

    runner(&instructions, &mut memory);
    // branch happened, so no store
    assert_eq!(memory[10], 0);

    // bltu is unsigned, so -1 is actually greater than 10
    let mut memory = [0u8; 64];
    memory[0] = -1_i8 as u8;
    memory[1] = 10;
    memory[2] = 30;

    runner(&instructions, &mut memory);
    // branch did not happen, so store of 30
    assert_eq!(memory[10], 30);
}

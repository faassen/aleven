use aleven::parse;
use aleven::run::{run_interpreter_func, run_llvm_func, RunnerFunc};
use aleven::Serializer;
use parameterized::parameterized;

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_bug1(runner: RunnerFunc) {
    let assembler = Serializer::new();
    let instructions = assembler.deserialize(&[10, 0, 43, 45]);
    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_bug2(runner: RunnerFunc) {
    let assembler = Serializer::new();
    let instructions = assembler.deserialize(&[11, 42, 222, 10]);
    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_bug3(runner: RunnerFunc) {
    let assembler = Serializer::new();
    let instructions = assembler.deserialize(&[]);
    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_bug4(runner: RunnerFunc) {
    let assembler = Serializer::new();
    let instructions = assembler.deserialize(&[7, 92, 209, 218, 176]);
    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_bug5(runner: RunnerFunc) {
    let assembler = Serializer::new();
    let instructions = assembler.deserialize(&[254, 22, 68, 156, 25, 49]);
    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_bug6(runner: RunnerFunc) {
    let assembler = Serializer::new();
    let instructions =
        assembler.deserialize(&[5, 0, 0, 0, 0, 0, 0, 91, 27, 0, 0, 0, 96, 0, 1, 213, 21]);
    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_bug7(runner: RunnerFunc) {
    let assembler = Serializer::new();
    let instructions = assembler.deserialize(&[
        5, 234, 234, 234, 234, 234, 234, 234, 234, 29, 21, 234, 234, 234, 234, 32, 10, 32, 6, 10,
    ]);
    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_bug8(runner: RunnerFunc) {
    let assembler = Serializer::new();
    let instructions = assembler.deserialize(&[
        0, 0, 234, 249, 185, 255, 230, 5, 191, 150, 150, 150, 150, 150, 150, 150, 150, 150, 150,
        150, 150, 150, 150, 150, 150, 150, 22, 6, 70, 0, 22,
    ]);
    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_bug9(runner: RunnerFunc) {
    let assembler = Serializer::new();
    let data = [
        20, 77, 22, 146, 146, 146, 146, 146, 146, 146, 146, 146, 146, 146, 146, 146, 146, 0, 146,
        146, 146, 146, 146, 146, 146, 146, 146, 146, 146, 146, 146, 146, 146, 146, 146, 146, 22,
        22, 0, 0, 0, 0, 0, 233, 0,
    ];
    let instructions = assembler.deserialize(&data);
    let mut memory = data.to_vec();
    runner(&instructions, &mut memory);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_bug10(runner: RunnerFunc) {
    let assembler = Serializer::new();
    let data = [25, 24, 24, 24, 24, 24];
    let instructions = assembler.deserialize(&data);
    let mut memory = data.to_vec();
    runner(&instructions, &mut memory);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_bug11(runner: RunnerFunc) {
    let assembler = Serializer::new();
    let data = [
        19, 25, 176, 25, 255, 25, 255, 255, 255, 255, 25, 25, 255, 12, 255, 25, 255, 12, 25, 255,
        255, 25, 25,
    ];
    let instructions = assembler.deserialize(&data);
    let mut memory = data.to_vec();
    runner(&instructions, &mut memory);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_bug12(runner: RunnerFunc) {
    let assembler = Serializer::new();
    let data = [
        25, 176, 19, 24, 34, 24, 24, 24, 255, 255, 255, 255, 24, 24, 24, 24, 24, 24, 24, 24, 24,
        24, 24, 24, 24, 24, 24, 24, 24, 9, 9, 235, 24, 90, 0, 0, 0, 24, 24, 24, 24, 235, 176, 25,
        255, 25, 19, 25, 126, 25, 176, 25, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
        24, 235, 176, 25, 255, 25, 19, 25, 126, 25, 176, 25, 255, 25, 19, 25, 25, 25, 0, 0, 0, 0,
        24, 24, 24, 24, 24, 24, 24, 25, 126,
    ];
    let instructions = assembler.deserialize(&data);
    let mut memory = data.to_vec();
    runner(&instructions, &mut memory);
}

#[test]
fn test_bug13() {
    let data = [
        23, 81, 23, 255, 255, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 44, 23, 23,
        23, 23, 255, 255, 37, 20, 1, 0, 23, 23, 23, 23, 23, 255, 255, 255, 255, 23, 23, 23, 23, 23,
        23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 255, 255, 23, 23, 23, 0, 0, 23, 23, 23, 23, 23,
        23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 44, 23, 23, 23, 23, 255, 255, 161,
        23, 23, 23, 23, 23, 255, 255, 0, 0, 0, 0, 0, 112, 0, 0, 255, 255, 37, 23, 23, 23, 23, 23,
        23, 23, 23, 23, 20, 1, 0, 44, 23, 23, 23, 23, 255, 255, 23, 23, 23, 23,
    ];

    let instructions = parse(
        "
    r23 = lb r23 1
    sb r23 65535 = r23",
    )
    .unwrap();

    // offset 81 + 65535 is beyond the bounds, so should have no effect
    // for some reason position 80 is different, so it looks like there
    // was a wraparound for the write in llvm but not in the interpreter
    // In the end I fixed the interpreter to match llvm to fix this test
    let mut memory0 = data.to_vec();
    run_llvm_func(&instructions, &mut memory0);

    let mut memory1 = data.to_vec();
    run_interpreter_func(&instructions, &mut memory1);

    assert_eq!(memory0, memory1);
}

#[test]
fn test_bug14() {
    let data = [
        37, 37, 19, 16, 16, 244, 16, 16, 16, 153, 16, 16, 153, 16, 16, 1, 0, 10, 16, 244, 16, 16,
        19, 16, 16, 244, 16, 16, 16, 1, 0, 0, 0, 0, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22,
        22, 22, 22, 22, 22, 22, 22, 22, 22, 170, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22,
        22, 22, 22, 22, 22, 22, 22, 22, 22, 255, 255, 255, 255, 6, 6, 0, 0, 25, 14, 11, 255, 6, 14,
        96, 23, 49, 176, 14, 0, 6, 25, 14, 59, 11, 255, 6, 255, 22, 22, 22, 22, 22, 22, 22, 22,
        153, 10, 16, 22, 22, 234, 233, 233, 232, 22, 22, 22, 22, 22, 22, 22, 22, 0, 16, 16, 16, 1,
        244, 16, 16, 153, 193, 16, 16, 1, 0, 10, 16, 244, 16, 16, 19, 16, 16, 244, 6, 6, 0, 0, 25,
        14, 11, 255, 6, 14, 96, 23, 49, 176, 14, 0, 6, 25, 14, 59, 11, 255, 6, 255, 22, 22, 22, 22,
        22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 0, 16, 16, 22, 22, 22, 255,
        255, 255, 255, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 16, 1, 244, 16, 16, 16,
        153, 22, 22, 22, 224, 22, 22, 16, 16, 1, 22, 22, 22, 22, 22, 22, 0, 25, 227, 1, 254, 23, 0,
        0,
    ];
    let instructions = parse(
        "
      r22 = addi r22 5632
      r22 = slli r22 5887 
      r22 = sub r16 r22
      sh r22 5654 = r22
    ",
    )
    .unwrap();

    let mut memory0 = data.to_vec();
    run_llvm_func(&instructions, &mut memory0);

    let mut memory1 = data.to_vec();
    run_interpreter_func(&instructions, &mut memory1);
    assert_eq!(memory0, memory1);
}

#[test]
fn test_bug15() {
    let data = [
        1, 0, 0, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22,
        22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22,
        22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22,
        22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22,
        22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 128, 128, 128, 128, 128, 128,
        128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
        128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 22, 22, 22, 22, 22, 22,
        22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22,
        255, 255, 255, 255, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 49, 54, 98, 105, 116, 45, 109,
        111, 100, 101, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 0, 25, 227, 254,
        23, 0,
    ];

    let instructions = parse(
        "
    sh r0 32790 = r0",
    )
    .unwrap();

    let mut memory0 = data.to_vec();
    run_llvm_func(&instructions, &mut memory0);

    let mut memory1 = data.to_vec();
    run_interpreter_func(&instructions, &mut memory1);
    assert_eq!(memory0, data);
    assert_eq!(memory0, memory1);
}

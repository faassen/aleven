use aleven::parse;
use aleven::run::{run_interpreter_func, run_llvm_func, RunnerFunc};
use byteorder::{ByteOrder, LittleEndian};
use parameterized::parameterized;

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_add(runner: RunnerFunc) {
    let instructions = parse(
        "
    r2 = addi r1 33
    r3 = addi r1 44
    r4 = add r2 r3
    sb r5 10 = r4
    ",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 77);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_add_negative(runner: RunnerFunc) {
    let instructions = parse(
        "
    r2 = addi r1 33
    r3 = addi r1 -11
    r4 = add r2 r3
    sb r5 10 = r4
    ",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 22);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_sub(runner: RunnerFunc) {
    let instructions = parse(
        "
    r2 = addi r1 33
    r3 = addi r1 11
    r4 = sub r2 r3
    sb r5 10 = r4
    ",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 22);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_add_wrapping(runner: RunnerFunc) {
    let max = i16::MAX.to_string();
    let code = format!(
        "
        r2 = addi r1 {max}
        r3 = addi r1 1
        r4 = add r2 r3
        sh r5 10 = r4"
    );
    let instructions = parse(&code).unwrap();

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    let value = LittleEndian::read_i16(&memory[20..]);
    assert_eq!(value, i16::MIN);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_add_sh(runner: RunnerFunc) {
    let instructions = parse(
        "
    r2 = addi r1 255
    r3 = addi r1 255
    r4 = add r2 r3
    sh r5 10 = r4
    ",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    let value = LittleEndian::read_u16(&memory[20..]);
    assert_eq!(value, 255 * 2);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_slt_less(runner: RunnerFunc) {
    let instructions = parse(
        "
    r2 = addi r1 33
    r3 = addi r1 44
    r4 = slt r2 r3
    sb r5 10 = r4
    ",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 1);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_slt_less_negative(runner: RunnerFunc) {
    let instructions = parse(
        "
    r2 = addi r1 -33
    r3 = addi r1 44
    r4 = slt r2 r3
    sb r5 10 = r4
    ",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 1);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_slt_equal(runner: RunnerFunc) {
    let instructions = parse(
        "
    r2 = addi r1 44
    r3 = addi r1 44
    r4 = slt r2 r3
    sb r5 10 = r4
    ",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_slt_greater(runner: RunnerFunc) {
    let instructions = parse(
        "
    r2 = addi r1 44
    r3 = addi r1 33
    r4 = slt r2 r3
    sb r5 10 = r4
    ",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_sltu_less(runner: RunnerFunc) {
    let instructions = parse(
        "
    r2 = addi r1 33
    r3 = addi r1 44
    r4 = sltu r2 r3
    sb r5 10 = r4
    ",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 1);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_sltu_less_negative(runner: RunnerFunc) {
    let instructions = parse(
        "
    r2 = addi r1 -33
    r3 = addi r1 44
    r4 = sltu r2 r3
    sb r5 10 = r4
    ",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_sltu_equal(runner: RunnerFunc) {
    let instructions = parse(
        "
    r2 = addi r1 44
    r3 = addi r1 44
    r4 = sltu r2 r3
    sb r5 10 = r4
    ",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_sltu_greater(runner: RunnerFunc) {
    let instructions = parse(
        "
    r2 = addi r1 44
    r3 = addi r1 33
    r4 = slt r2 r3
    sb r5 10 = r4
    ",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_and(runner: RunnerFunc) {
    let b1 = 0b1010101.to_string();
    let b2 = 0b1111110.to_string();

    let code = format!(
        "
    r2 = addi r1 {b1}
    r3 = addi r1 {b2}
    r4 = and r2 r3
    sb r5 10 = r4
    "
    );
    let instructions = parse(&code).unwrap();

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0b1010100);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_or(runner: RunnerFunc) {
    let b1 = 0b1010100.to_string();
    let b2 = 0b1111110.to_string();

    let code = format!(
        "
    r2 = addi r1 {b1}
    r3 = addi r1 {b2}
    r4 = or r2 r3
    sb r5 10 = r4
    "
    );
    let instructions = parse(&code).unwrap();

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0b1111110);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_xor(runner: RunnerFunc) {
    let b1 = 0b1111010.to_string();
    let b2 = 0b1010100.to_string();

    let code = format!(
        "
    r2 = addi r1 {b1}
    r3 = addi r1 {b2}
    r4 = xor r2 r3
    sb r5 10 = r4
    "
    );
    let instructions = parse(&code).unwrap();

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0b0101110);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_sll(runner: RunnerFunc) {
    let b1 = 0b101.to_string();

    let code = format!(
        "
    r2 = addi r1 {b1}
    r3 = addi r1 2
    r4 = sll r2 r3
    sb r5 10 = r4
    "
    );
    let instructions = parse(&code).unwrap();

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0b10100);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_sll_shift_too_large(runner: RunnerFunc) {
    let b1 = 0b101.to_string();

    let code = format!(
        "
    r2 = addi r1 {b1}
    r3 = addi r1 100
    r4 = sll r2 r3
    sb r5 10 = r4
    "
    );
    let instructions = parse(&code).unwrap();

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0b101);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_srl(runner: RunnerFunc) {
    let b1 = 0b10100.to_string();

    let code = format!(
        "
    r2 = addi r1 {b1}
    r3 = addi r1 2
    r4 = srl r2 r3
    sb r5 10 = r4
    "
    );
    let instructions = parse(&code).unwrap();

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0b101);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_srl_too_large(runner: RunnerFunc) {
    let b1 = 0b10100.to_string();

    let code = format!(
        "
    r2 = addi r1 {b1}
    r3 = addi r1 100
    r4 = srl r2 r3
    sb r5 10 = r4
    "
    );
    let instructions = parse(&code).unwrap();

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0b10100);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_srl_negative(runner: RunnerFunc) {
    let instructions = parse(
        "
    r2 = addi r1 -20
    r3 = addi r1 2
    r4 = srl r2 r3
    sh r5 10 = r4
    ",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    let value = LittleEndian::read_i16(&memory[20..]);
    assert_eq!(value, 16379);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_sra(runner: RunnerFunc) {
    let b1 = 0b10100.to_string();

    let code = format!(
        "
    r2 = addi r1 {b1}
    r3 = addi r1 2
    r4 = sra r2 r3
    sb r5 10 = r4
    "
    );
    let instructions = parse(&code).unwrap();

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0b101);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_sra_negative(runner: RunnerFunc) {
    let instructions = parse(
        "
    r2 = addi r1 -20
    r3 = addi r1 2
    r4 = sra r2 r3
    sh r5 10 = r4
    ",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    let value = LittleEndian::read_i16(&memory[20..]);
    assert_eq!(value, -5);
}

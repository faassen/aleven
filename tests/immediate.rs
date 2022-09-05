use aleven::parse;
use aleven::run::{run_interpreter_func, run_llvm_func, RunnerFunc};
use byteorder::{ByteOrder, LittleEndian};
use parameterized::parameterized;

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_addi_basic(runner: RunnerFunc) {
    let instructions = parse(
        "
    r2 = addi r1 33
    sb r3 10 = r2",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 33);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_addi_register_has_value(runner: RunnerFunc) {
    let instructions = parse(
        "
    r1 = addi r1 10
    r2 = addi r1 33
    sb r3 10 = r2",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 43);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_addi_register_rs_is_rd(runner: RunnerFunc) {
    let instructions = parse(
        "
    r1 = addi r1 10
    r1 = addi r1 33
    sb r2 10 = r1",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 43);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_addi_register_dec(runner: RunnerFunc) {
    let instructions = parse(
        "
    r1 = addi r1 10
    r1 = addi r1 -1
    sb r2 10 = r1",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 9);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_slti_less(runner: RunnerFunc) {
    let instructions = parse(
        "
    r2 = slti r1 5
    sb r3 10 = r2",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 1);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_slti_less_negative(runner: RunnerFunc) {
    let instructions = parse(
        "
    r2 = slti r1 -4
    sb r3 10 = r2",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_sltiu_less(runner: RunnerFunc) {
    let instructions = parse(
        "
    r2 = sltiu r1 -4
    sb r3 10 = r2
    ",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 1);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_slti_equal(runner: RunnerFunc) {
    let instructions = parse(
        "
    r1 = addi r1 5
    r2 = slti r1 5
    sb r3 10 = r2",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_slti_greater(runner: RunnerFunc) {
    let instructions = parse(
        "
    r1 = addi r1 6
    r2 = slti r1 5
    sb r3 10 = r2",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_andi(runner: RunnerFunc) {
    let b1 = 0b1010101.to_string();
    let b2 = 0b1111110.to_string();

    let code = format!(
        "
    r1 = addi r1 {b1}
    r2 = andi r1 {b2}
    sb r3 10 = r2"
    );

    let instructions = parse(&code).unwrap();

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0b1010100);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_ori(runner: RunnerFunc) {
    let b1 = 0b1010100.to_string();
    let b2 = 0b1111110.to_string();

    let code = format!(
        "
    r1 = addi r1 {b1}
    r2 = ori r1 {b2}
    sb r3 10 = r2"
    );

    let instructions = parse(&code).unwrap();

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0b1111110);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_xori(runner: RunnerFunc) {
    let b1 = 0b1010100.to_string();
    let b2 = 0b1111010.to_string();

    let code = format!(
        "
    r1 = addi r1 {b1}
    r2 = xori r1 {b2}
    sb r3 10 = r2"
    );

    let instructions = parse(&code).unwrap();

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0b0101110);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_slli(runner: RunnerFunc) {
    let instructions = parse(
        "
    r1 = addi r1 5
    r2 = slli r1 2
    sb r3 10 = r2",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 20);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_srai(runner: RunnerFunc) {
    let instructions = parse(
        "
    r1 = addi r1 20
    r2 = srai r1 2
    sb r3 10 = r2",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 5);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_srli_zero_extends(runner: RunnerFunc) {
    let instructions = parse(
        "
    r2 = lb r1 0
    r2 = srli r2 2
    sh r3 10 = r2
    ",
    )
    .unwrap();

    let mut memory = [0u8; 64];
    memory[0] = -1i8 as u8;
    runner(&instructions, &mut memory);
    let value = LittleEndian::read_u16(&memory[20..]);
    assert_eq!(value, 0b11111111111111);
    assert_eq!(value, 16383);
}

use aleven::run::{run_interpreter_func, run_llvm_func, RunnerFunc};
use aleven::{Immediate, Instruction, Load, Store};
use byteorder::{ByteOrder, LittleEndian};
use parameterized::parameterized;

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_addi_basic(runner: RunnerFunc) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 33,
            rs: 1,
            rd: 2,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 2,
            rd: 3, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 33);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_addi_register_has_value(runner: RunnerFunc) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 10,
            rs: 1,
            rd: 1,
        }),
        Instruction::Addi(Immediate {
            value: 33,
            rs: 1,
            rd: 2,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 2,
            rd: 3, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 43);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_addi_register_rs_is_rd(runner: RunnerFunc) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 10,
            rs: 1,
            rd: 1,
        }),
        Instruction::Addi(Immediate {
            value: 33,
            rs: 1,
            rd: 1,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 1,
            rd: 2, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 43);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_addi_register_dec(runner: RunnerFunc) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 10,
            rs: 1,
            rd: 1,
        }),
        Instruction::Addi(Immediate {
            value: -1,
            rs: 1,
            rd: 1,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 1,
            rd: 2, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 9);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_slti_less(runner: RunnerFunc) {
    let instructions = [
        Instruction::Slti(Immediate {
            value: 5,
            rs: 1,
            rd: 2,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 2,
            rd: 3, // defaults to 0
        }),
    ];
    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 1);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_slti_less_negative(runner: RunnerFunc) {
    let instructions = [
        Instruction::Slti(Immediate {
            value: -4,
            rs: 1,
            rd: 2,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 2,
            rd: 3, // defaults to 0
        }),
    ];
    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_sltiu_less(runner: RunnerFunc) {
    let instructions = [
        Instruction::Sltiu(Immediate {
            value: -4, // treated as large number instead
            rs: 1,
            rd: 2,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 2,
            rd: 3, // defaults to 0
        }),
    ];
    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 1);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_slti_equal(runner: RunnerFunc) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 5,
            rs: 1,
            rd: 1,
        }),
        Instruction::Slti(Immediate {
            value: 5,
            rs: 1,
            rd: 2,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 2,
            rd: 3, // defaults to 0
        }),
    ];
    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_slti_greater(runner: RunnerFunc) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 6,
            rs: 1,
            rd: 1,
        }),
        Instruction::Slti(Immediate {
            value: 5,
            rs: 1,
            rd: 2,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 2,
            rd: 3, // defaults to 0
        }),
    ];
    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_andi(runner: RunnerFunc) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 0b1010101,
            rs: 1,
            rd: 1,
        }),
        Instruction::Andi(Immediate {
            value: 0b1111110,
            rs: 1,
            rd: 2,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 2,
            rd: 3, // defaults to 0
        }),
    ];
    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0b1010100);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_ori(runner: RunnerFunc) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 0b1010100,
            rs: 1,
            rd: 1,
        }),
        Instruction::Ori(Immediate {
            value: 0b1111110,
            rs: 1,
            rd: 2,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 2,
            rd: 3, // defaults to 0
        }),
    ];
    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0b1111110);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_xori(runner: RunnerFunc) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 0b1010100,
            rs: 1,
            rd: 1,
        }),
        Instruction::Xori(Immediate {
            value: 0b1111010,
            rs: 1,
            rd: 2,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 2,
            rd: 3, // defaults to 0
        }),
    ];
    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0b0101110);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_slli(runner: RunnerFunc) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 5,
            rs: 1,
            rd: 1,
        }),
        Instruction::Slli(Immediate {
            value: 2,
            rs: 1,
            rd: 2,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 2,
            rd: 3, // defaults to 0
        }),
    ];
    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 20);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_srai(runner: RunnerFunc) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 20,
            rs: 1,
            rd: 1,
        }),
        Instruction::Srai(Immediate {
            value: 2,
            rs: 1,
            rd: 2,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 2,
            rd: 3, // defaults to 0
        }),
    ];
    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 5);
}

#[parameterized(runner={run_llvm_func, run_interpreter_func})]
fn test_srli_zero_extends_srli(runner: RunnerFunc) {
    let instructions = [
        Instruction::Lb(Load {
            offset: 0,
            rs: 1,
            rd: 2,
        }),
        Instruction::Srli(Immediate {
            value: 2,
            rs: 2,
            rd: 2,
        }),
        Instruction::Sh(Store {
            offset: 10,
            rs: 2,
            rd: 3, // defaults to 0
        }),
    ];
    let mut memory = [0u8; 64];
    memory[0] = -1i8 as u8;
    runner(&instructions, &mut memory);
    let value = LittleEndian::read_u16(&memory[20..]);
    assert_eq!(value, 0b11111111111111);
    assert_eq!(value, 16383);
}

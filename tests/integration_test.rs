use aleven::assemble::Assembler;
use aleven::lang::Processor;
use aleven::lang::{Branch, BranchTarget, Immediate, Instruction, Load, Register, Store};
use aleven::llvm::CodeGen;
use aleven::program::Program;
use byteorder::{ByteOrder, LittleEndian};
use inkwell::context::Context;
use parameterized::parameterized;

type Runner = fn(&[Instruction], &mut [u8]);

fn run_llvm(instructions: &[Instruction], memory: &mut [u8]) {
    let program = Program::new(instructions);
    let context = Context::create();
    let codegen = CodeGen::new(&context);
    let func = program.compile(&codegen, memory.len() as u16);
    codegen.module.verify().unwrap();
    Program::run(func, memory);
}

fn run_interpreter(instructions: &[Instruction], memory: &mut [u8]) {
    let mut processor = Processor::new();
    Program::new(instructions).interpret(&mut processor, memory);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_add_immediate_basic(runner: Runner) {
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

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_add_immediate_register_has_value(runner: Runner) {
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

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_add_immediate_register_rs_is_rd(runner: Runner) {
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

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_add_immediate_register_dec(runner: Runner) {
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

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_slt_immediate_less(runner: Runner) {
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

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_slt_immediate_less_negative(runner: Runner) {
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

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_sltu_immediate_less(runner: Runner) {
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

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_slt_immediate_equal(runner: Runner) {
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

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_slt_immediate_greater(runner: Runner) {
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

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_and_immediate(runner: Runner) {
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

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_or_immediate(runner: Runner) {
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

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_xor_immediate(runner: Runner) {
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

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_sll_immediate(runner: Runner) {
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

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_sra_immediate(runner: Runner) {
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

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_lb_in_bounds(runner: Runner) {
    let instructions = [
        Instruction::Lb(Load {
            offset: 0,
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
    memory[0] = 11;
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 11);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_sb_out_of_bounds(runner: Runner) {
    let instructions = [
        Instruction::Lb(Load {
            offset: 0,
            rs: 1,
            rd: 2,
        }),
        Instruction::Sb(Store {
            offset: 65,
            rs: 2,
            rd: 3, // defaults to 0
        }),
    ];
    let mut memory = [0u8; 64];
    memory[0] = 11;
    let expected = memory;
    runner(&instructions, &mut memory);
    assert_eq!(memory, expected);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_lb_out_of_bounds_means_zero(runner: Runner) {
    let instructions = [
        Instruction::Lb(Load {
            offset: 65,
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
    memory[10] = 100;
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_lbu_out_of_bounds_means_nop(runner: Runner) {
    let instructions = [
        Instruction::Lbu(Load {
            offset: 65,
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

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_lh_sh(runner: Runner) {
    let instructions = [
        Instruction::Lh(Load {
            offset: 0,
            rs: 1,
            rd: 2,
        }),
        Instruction::Sh(Store {
            offset: 10,
            rs: 2,
            rd: 3, // defaults to 0
        }),
    ];
    let mut memory = [0u8; 64];
    memory[0] = 2;
    memory[1] = 1;
    runner(&instructions, &mut memory);
    // is at 20 as pointer type is 16 bits
    assert_eq!(memory[20], 2);
    assert_eq!(memory[21], 1);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_lh_aligns(runner: Runner) {
    let instructions = [
        Instruction::Lh(Load {
            offset: 1, // aligned back to 0
            rs: 1,
            rd: 2,
        }),
        Instruction::Sh(Store {
            offset: 10,
            rs: 2,
            rd: 3, // defaults to 0
        }),
    ];
    let mut memory = [0u8; 64];
    memory[2] = 2;
    memory[3] = 1;
    runner(&instructions, &mut memory);
    assert_eq!(memory[20], 2);
    assert_eq!(memory[21], 1);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_lh_out_of_bounds(runner: Runner) {
    let instructions = [
        Instruction::Lh(Load {
            offset: 65, // out of bounds
            rs: 1,
            rd: 2,
        }),
        Instruction::Sh(Store {
            offset: 10,
            rs: 2,
            rd: 3, // defaults to 0
        }),
    ];
    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory, [0u8; 64]);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_sh_aligns(runner: Runner) {
    let instructions = [
        Instruction::Lh(Load {
            offset: 0,
            rs: 1,
            rd: 2,
        }),
        // it's not possible to misalign this as it's already even
        Instruction::Sh(Store {
            offset: 11,
            rs: 2,
            rd: 3, // defaults to 0
        }),
    ];
    let mut memory = [0u8; 64];
    memory[0] = 2;
    memory[1] = 1;
    runner(&instructions, &mut memory);
    assert_eq!(memory[22], 2);
    assert_eq!(memory[23], 1);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_sh_out_of_bounds(runner: Runner) {
    let instructions = [
        Instruction::Lh(Load {
            offset: 0,
            rs: 1,
            rd: 2,
        }),
        // it's not possible to misalign this as it's already even
        Instruction::Sh(Store {
            offset: 32, // should be out of bounds as x 2
            rs: 2,
            rd: 3, // defaults to 0
        }),
    ];
    let mut memory = [0u8; 64];
    memory[0] = 2;
    memory[1] = 1;
    let expected = memory;
    runner(&instructions, &mut memory);
    assert_eq!(memory, expected);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_lb_sign_extends(runner: Runner) {
    let instructions = [
        Instruction::Lb(Load {
            offset: 0,
            rs: 1,
            rd: 2,
        }),
        Instruction::Sh(Store {
            offset: 10,
            rs: 2,
            rd: 3, // defaults to 0
        }),
    ];
    let mut memory = [0u8; 64];
    memory[0] = -4i8 as u8; // FFFC
    runner(&instructions, &mut memory);
    let value = LittleEndian::read_i16(&memory[20..]);
    assert_eq!(value, -4);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_lbu_zero_extends(runner: Runner) {
    let instructions = [
        Instruction::Lbu(Load {
            offset: 0,
            rs: 1,
            rd: 2,
        }),
        Instruction::Sh(Store {
            offset: 10,
            rs: 2,
            rd: 3, // defaults to 0
        }),
    ];
    let mut memory = [0u8; 64];
    memory[0] = -4i8 as u8; // FC
    runner(&instructions, &mut memory);
    let value = LittleEndian::read_i16(&memory[20..]);
    assert_eq!(value, 252);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_lb_sign_extends_with_sra(runner: Runner) {
    let instructions = [
        Instruction::Lb(Load {
            offset: 0,
            rs: 1,
            rd: 2,
        }),
        Instruction::Srai(Immediate {
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
    memory[0] = -4i8 as u8; // FFFC
    runner(&instructions, &mut memory);
    let value = LittleEndian::read_i16(&memory[20..]);
    assert_eq!(value, -1);
    assert_eq!(value, 0xFFFFu16 as i16);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_lbu_zero_extends_sra(runner: Runner) {
    let instructions = [
        Instruction::Lbu(Load {
            offset: 0,
            rs: 1,
            rd: 2,
        }),
        Instruction::Srai(Immediate {
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
    memory[0] = -4i8 as u8;
    // lbu interprets this as 252, or 0000011111100
    runner(&instructions, &mut memory);
    let value = LittleEndian::read_u16(&memory[20..]);
    assert_eq!(value, 63);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_srli_zero_extends_srli(runner: Runner) {
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

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_add(runner: Runner) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 33,
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: 44,
            rs: 1,
            rd: 3,
        }),
        Instruction::Add(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 77);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_add_negative(runner: Runner) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 33,
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: -11,
            rs: 1,
            rd: 3,
        }),
        Instruction::Add(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 22);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_sub(runner: Runner) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 33,
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: 11,
            rs: 1,
            rd: 3,
        }),
        Instruction::Sub(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 22);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_add_wrapping(runner: Runner) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: i16::MAX,
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: 1,
            rs: 1,
            rd: 3,
        }),
        Instruction::Add(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sh(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    let value = LittleEndian::read_i16(&memory[20..]);
    assert_eq!(value, i16::MIN);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_add_sh(runner: Runner) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 255,
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: 255,
            rs: 1,
            rd: 3,
        }),
        Instruction::Add(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sh(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    let value = LittleEndian::read_u16(&memory[20..]);
    assert_eq!(value, 255 * 2);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_slt_less(runner: Runner) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 33,
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: 44,
            rs: 1,
            rd: 3,
        }),
        Instruction::Slt(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 1);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_slt_less_negative(runner: Runner) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: -33,
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: 44,
            rs: 1,
            rd: 3,
        }),
        Instruction::Slt(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 1);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_slt_equal(runner: Runner) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 44,
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: 44,
            rs: 1,
            rd: 3,
        }),
        Instruction::Slt(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_slt_greater(runner: Runner) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 44,
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: 33,
            rs: 1,
            rd: 3,
        }),
        Instruction::Slt(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_sltu_less(runner: Runner) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 33,
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: 44,
            rs: 1,
            rd: 3,
        }),
        Instruction::Sltu(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 1);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_sltu_less_negative(runner: Runner) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: -33, // interpreted as big
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: 44,
            rs: 1,
            rd: 3,
        }),
        Instruction::Sltu(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_sltu_equal(runner: Runner) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 44,
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: 44,
            rs: 1,
            rd: 3,
        }),
        Instruction::Sltu(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_sltu_greater(runner: Runner) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 44,
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: 33,
            rs: 1,
            rd: 3,
        }),
        Instruction::Sltu(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_and(runner: Runner) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 0b1010101,
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: 0b1111110,
            rs: 1,
            rd: 3,
        }),
        Instruction::And(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0b1010100);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_or(runner: Runner) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 0b1010100,
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: 0b1111110,
            rs: 1,
            rd: 3,
        }),
        Instruction::Or(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0b1111110);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_xor(runner: Runner) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 0b1111010,
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: 0b1010100,
            rs: 1,
            rd: 3,
        }),
        Instruction::Xor(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0b0101110);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_sll(runner: Runner) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 0b101,
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: 2,
            rs: 1,
            rd: 3,
        }),
        Instruction::Sll(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0b10100);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_sll_shift_too_large(runner: Runner) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 0b101,
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: 100,
            rs: 1,
            rd: 3,
        }),
        Instruction::Sll(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0b101);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_srl(runner: Runner) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 0b10100,
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: 2,
            rs: 1,
            rd: 3,
        }),
        Instruction::Srl(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0b101);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_srl_too_large(runner: Runner) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 0b10100,
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: 100,
            rs: 1,
            rd: 3,
        }),
        Instruction::Srl(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0b10100);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_srl_negative(runner: Runner) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: -20,
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: 2,
            rs: 1,
            rd: 3,
        }),
        Instruction::Srl(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sh(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    let value = LittleEndian::read_i16(&memory[20..]);
    assert_eq!(value, 16379);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_sra(runner: Runner) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: 0b10100,
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: 2,
            rs: 1,
            rd: 3,
        }),
        Instruction::Sra(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    assert_eq!(memory[10], 0b101);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_sra_negative(runner: Runner) {
    let instructions = [
        Instruction::Addi(Immediate {
            value: -20,
            rs: 1,
            rd: 2,
        }),
        Instruction::Addi(Immediate {
            value: 2,
            rs: 1,
            rd: 3,
        }),
        Instruction::Sra(Register {
            rs1: 2,
            rs2: 3,
            rd: 4,
        }),
        Instruction::Sh(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
    let value = LittleEndian::read_i16(&memory[20..]);
    assert_eq!(value, -5);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_beq_simple(runner: Runner) {
    let instructions = [
        Instruction::Lb(Load {
            offset: 0,
            rs: 1,
            rd: 2,
        }),
        Instruction::Lb(Load {
            offset: 1,
            rs: 1,
            rd: 3,
        }),
        Instruction::Beq(Branch {
            rs1: 2,
            rs2: 3,
            target: 1,
        }),
        Instruction::Lb(Load {
            offset: 2,
            rs: 1,
            rd: 4,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
        Instruction::Target(BranchTarget { identifier: 1 }),
    ];

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

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_beq_nonexistent_target_means_nop(runner: Runner) {
    let instructions = [
        Instruction::Lb(Load {
            offset: 0,
            rs: 1,
            rd: 2,
        }),
        Instruction::Lb(Load {
            offset: 1,
            rs: 1,
            rd: 3,
        }),
        Instruction::Beq(Branch {
            rs1: 2,
            rs2: 3,
            target: 2, // does not exist!
        }),
        Instruction::Lb(Load {
            offset: 2,
            rs: 1,
            rd: 4,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
        Instruction::Target(BranchTarget { identifier: 1 }),
    ];

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

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_beq_earlier_target_means_nop(runner: Runner) {
    let instructions = [
        Instruction::Lb(Load {
            offset: 0,
            rs: 1,
            rd: 2,
        }),
        Instruction::Lb(Load {
            offset: 1,
            rs: 1,
            rd: 3,
        }),
        Instruction::Target(BranchTarget { identifier: 1 }),
        Instruction::Beq(Branch {
            rs1: 2,
            rs2: 3,
            target: 1, // exists, but before me
        }),
        Instruction::Lb(Load {
            offset: 2,
            rs: 1,
            rd: 4,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 4,
            rd: 5, // defaults to 0
        }),
    ];

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

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_addi_after_beq(runner: Runner) {
    use Instruction::*;
    let instructions = [
        Target(BranchTarget { identifier: 176 }),
        Lh(Load {
            offset: 8728,
            rs: 24,
            rd: 24,
        }),
        Beq(Branch {
            target: 255,
            rs1: 31,
            rs2: 31,
        }),
        Addi(Immediate {
            value: 6168,
            rs: 24,
            rd: 24,
        }),
        Target(BranchTarget { identifier: 255 }),
        Addi(Immediate {
            value: 0,
            rs: 24,
            rd: 24,
        }),
    ];
    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_bug1(runner: Runner) {
    let assembler = Assembler::new();
    let instructions = assembler.disassemble(&[10, 0, 43, 45]);
    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_bug2(runner: Runner) {
    let assembler = Assembler::new();
    let instructions = assembler.disassemble(&[11, 42, 222, 10]);
    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_bug3(runner: Runner) {
    let assembler = Assembler::new();
    let instructions = assembler.disassemble(&[]);
    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_bug4(runner: Runner) {
    let assembler = Assembler::new();
    let instructions = assembler.disassemble(&[7, 92, 209, 218, 176]);
    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_bug5(runner: Runner) {
    let assembler = Assembler::new();
    let instructions = assembler.disassemble(&[254, 22, 68, 156, 25, 49]);
    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_bug6(runner: Runner) {
    let assembler = Assembler::new();
    let instructions =
        assembler.disassemble(&[5, 0, 0, 0, 0, 0, 0, 91, 27, 0, 0, 0, 96, 0, 1, 213, 21]);
    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_bug7(runner: Runner) {
    let assembler = Assembler::new();
    let instructions = assembler.disassemble(&[
        5, 234, 234, 234, 234, 234, 234, 234, 234, 29, 21, 234, 234, 234, 234, 32, 10, 32, 6, 10,
    ]);
    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_bug8(runner: Runner) {
    let assembler = Assembler::new();
    let instructions = assembler.disassemble(&[
        0, 0, 234, 249, 185, 255, 230, 5, 191, 150, 150, 150, 150, 150, 150, 150, 150, 150, 150,
        150, 150, 150, 150, 150, 150, 150, 22, 6, 70, 0, 22,
    ]);
    let mut memory = [0u8; 64];
    runner(&instructions, &mut memory);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_bug9(runner: Runner) {
    let assembler = Assembler::new();
    let data = [
        20, 77, 22, 146, 146, 146, 146, 146, 146, 146, 146, 146, 146, 146, 146, 146, 146, 0, 146,
        146, 146, 146, 146, 146, 146, 146, 146, 146, 146, 146, 146, 146, 146, 146, 146, 146, 22,
        22, 0, 0, 0, 0, 0, 233, 0,
    ];
    let instructions = assembler.disassemble(&data);
    let mut memory = data.to_vec();
    runner(&instructions, &mut memory);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_bug10(runner: Runner) {
    let assembler = Assembler::new();
    let data = [25, 24, 24, 24, 24, 24];
    let instructions = assembler.disassemble(&data);
    let mut memory = data.to_vec();
    runner(&instructions, &mut memory);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_bug11(runner: Runner) {
    let assembler = Assembler::new();
    let data = [
        19, 25, 176, 25, 255, 25, 255, 255, 255, 255, 25, 25, 255, 12, 255, 25, 255, 12, 25, 255,
        255, 25, 25,
    ];
    let instructions = assembler.disassemble(&data);
    let mut memory = data.to_vec();
    runner(&instructions, &mut memory);
}

#[parameterized(runner={run_llvm, run_interpreter})]
fn test_bug12(runner: Runner) {
    let assembler = Assembler::new();
    let data = [
        25, 176, 19, 24, 34, 24, 24, 24, 255, 255, 255, 255, 24, 24, 24, 24, 24, 24, 24, 24, 24,
        24, 24, 24, 24, 24, 24, 24, 24, 9, 9, 235, 24, 90, 0, 0, 0, 24, 24, 24, 24, 235, 176, 25,
        255, 25, 19, 25, 126, 25, 176, 25, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
        24, 235, 176, 25, 255, 25, 19, 25, 126, 25, 176, 25, 255, 25, 19, 25, 25, 25, 0, 0, 0, 0,
        24, 24, 24, 24, 24, 24, 24, 25, 126,
    ];
    let instructions = assembler.disassemble(&data);
    let mut memory = data.to_vec();
    runner(&instructions, &mut memory);
}

#[test]
fn test_bug13() {
    use Instruction::*;
    let data = [
        23, 81, 23, 255, 255, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 44, 23, 23,
        23, 23, 255, 255, 37, 20, 1, 0, 23, 23, 23, 23, 23, 255, 255, 255, 255, 23, 23, 23, 23, 23,
        23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 255, 255, 23, 23, 23, 0, 0, 23, 23, 23, 23, 23,
        23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 44, 23, 23, 23, 23, 255, 255, 161,
        23, 23, 23, 23, 23, 255, 255, 0, 0, 0, 0, 0, 112, 0, 0, 255, 255, 37, 23, 23, 23, 23, 23,
        23, 23, 23, 23, 20, 1, 0, 44, 23, 23, 23, 23, 255, 255, 23, 23, 23, 23,
    ];

    let instructions = [
        Lb(Load {
            offset: 1, // load 81 into register 23
            rs: 23,
            rd: 23,
        }),
        Sb(Store {
            offset: 65535, // save register 23 to memory at offset 65535
            rs: 23,
            rd: 23,
        }),
    ];

    // offset 81 + 65535 is beyond the bounds, so should have no effect
    // for some reason position 80 is different, so it looks like there
    // was a wraparound for the write in llvm but not in the interpreter
    // In the end I fixed the interpreter to match llvm to fix this test
    let mut memory0 = data.to_vec();
    run_llvm(&instructions, &mut memory0);

    let mut memory1 = data.to_vec();
    run_interpreter(&instructions, &mut memory1);

    assert_eq!(memory0, memory1);
}

#[test]
fn test_bug14() {
    use Instruction::*;
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
    let instructions = [
        // register 22 has 5632 in it
        Addi(Immediate {
            value: 5632,
            rs: 22,
            rd: 22,
        }),
        // shifting with 5887 should make it stay the same
        Slli(Immediate {
            value: 5887,
            rs: 22,
            rd: 22,
        }),
        // now we do r16 - r22, which should be -5632
        Sub(Register {
            rs1: 16,
            rs2: 22,
            rd: 22,
        }),
        // and now we write to -5632 + 5654
        Sh(Store {
            offset: 5654,
            rs: 22,
            rd: 22,
        }),
    ];
    let mut memory0 = data.to_vec();
    run_llvm(&instructions, &mut memory0);

    let mut memory1 = data.to_vec();
    run_interpreter(&instructions, &mut memory1);
    assert_eq!(memory0, memory1);
}

#[test]
fn test_bug15() {
    use Instruction::*;
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
    // the interpreter wrote data here, while the compiler did not
    // position 44 is written to with 0 by the interpreter
    // fixed the interpreter so that overflows don't cause a write
    let instructions = [Sh(Store {
        offset: 32790,
        rs: 0,
        rd: 0,
    })];
    let mut memory0 = data.to_vec();
    run_llvm(&instructions, &mut memory0);

    let mut memory1 = data.to_vec();
    run_interpreter(&instructions, &mut memory1);
    assert_eq!(memory0, data);
    assert_eq!(memory0, memory1);
}

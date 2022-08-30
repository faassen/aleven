use aleven::{CodeGen, Function, FunctionValueCache};
use aleven::{Immediate, Instruction, Program, Register, Store};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use inkwell::context::Context;

const INSTRUCTIONS: [Instruction; 4] = [
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

fn interpreter_benchmark(c: &mut Criterion) {
    let mut memory = [0u8; 64];
    let program = Program::new(&[&INSTRUCTIONS]);
    c.bench_function("interpreter", |b| {
        b.iter(|| program.interpret(black_box(&mut memory)))
    });
}

fn llvm_benchmark(c: &mut Criterion) {
    let mut memory = [0u8; 64];
    let program = Program::new(&[&INSTRUCTIONS]);
    let mut cache = FunctionValueCache::new();
    let context = Context::create();
    let codegen = CodeGen::new(&context);
    let f = program.compile(0, &codegen, memory.len() as u16, &mut cache);

    c.bench_function("llvm", |b| {
        b.iter(|| Function::run(&f, black_box(&mut memory)))
    });
}

criterion_group!(benches, interpreter_benchmark, llvm_benchmark);
criterion_main!(benches);

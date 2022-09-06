use aleven::parse;
use aleven::Program;
use aleven::{CodeGen, Function, FunctionValueCache};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use inkwell::context::Context;

const CODE: &str = "
r2 = addi r1 33
r3 = addi r2 44
r4 = add r2 r3
sb r5 10 = r4
";

fn interpreter_benchmark(c: &mut Criterion) {
    let mut memory = [0u8; 64];
    let program = Program::new(&[(0, &parse(CODE).unwrap())]);
    c.bench_function("interpreter", |b| {
        b.iter(|| program.interpret(black_box(&mut memory)))
    });
}

fn llvm_benchmark(c: &mut Criterion) {
    let mut memory = [0u8; 64];
    let program = Program::new(&[(0, &parse(CODE).unwrap())]);
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

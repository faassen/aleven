use crate::reglang::{Immediate, Instruction, Load, Register, Store};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::Module;
use inkwell::values::{IntValue, PointerValue};
use inkwell::{AddressSpace, OptimizationLevel};
use std::error::Error;

/// Convenience type alias for the `sum` function.
///
/// Calling this is innately `unsafe` because there's no guarantee it doesn't
/// do `unsafe` operations internally.
type SumFunc = unsafe extern "C" fn(u64, u64, u64) -> u64;

type ProgramFunc = unsafe extern "C" fn(*mut u8) -> ();

struct CodeGen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    execution_engine: ExecutionEngine<'ctx>,
}

type Registers<'a> = [IntValue<'a>; 32];

impl<'ctx> CodeGen<'ctx> {
    fn jit_compile_program(
        &self,
        instructions: &[Instruction],
    ) -> Option<JitFunction<ProgramFunc>> {
        let i8_type = self.context.i8_type();
        let i16_type = self.context.i16_type();

        let void_type = self.context.void_type();
        let ptr_type = i8_type.ptr_type(AddressSpace::Generic);
        let fn_type = void_type.fn_type(&[ptr_type.into()], false);

        let function = self.module.add_function("program", fn_type, None);
        let basic_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(basic_block);

        let ptr = function.get_nth_param(0)?.into_pointer_value();

        let mut registers: Registers = [i16_type.const_int(0, false); 32];

        for instruction in instructions {
            match instruction {
                Instruction::AddI(immediate) => self.jit_compile_addi(&mut registers, immediate),
                Instruction::Add(register) => {}
                Instruction::Load(load) => {
                    self.jit_compile_load(&mut registers, ptr, load);
                }
                Instruction::Store(store) => {
                    self.jit_compile_store(&registers, ptr, store);
                }
                _ => {}
            }
        }
        self.builder.build_return(None);

        unsafe { self.execution_engine.get_function("program").ok() }
    }

    fn jit_compile_addi(&self, registers: &mut Registers<'ctx>, immediate: &Immediate) {
        let i16_type = self.context.i16_type();
        // XXX u64, how do negatives work?
        let value = i16_type.const_int(immediate.value as u64, false);
        let rs = registers[immediate.rs as usize];
        let sum = self.builder.build_int_add(value, rs, "addi");
        registers[immediate.rd as usize] = sum;
    }

    fn jit_compile_load(
        &self,
        registers: &mut Registers<'ctx>,
        ptr: PointerValue<'ctx>,
        load: &Load,
    ) {
        let offset = self.context.i16_type().const_int(load.offset as u64, false);
        let index = self
            .builder
            .build_int_add(offset, registers[load.rs as usize], "index");
        let address = unsafe { self.builder.build_gep(ptr, &[index], "gep index") };
        let value = self.builder.build_load(address, "load");
        registers[load.rd as usize] = value.into_int_value();
    }

    fn jit_compile_store(&self, registers: &Registers, ptr: PointerValue, store: &Store) {
        let offset = self
            .context
            .i16_type()
            .const_int(store.offset as u64, false);
        let index = self
            .builder
            .build_int_add(offset, registers[store.rd as usize], "index");
        let address = unsafe { self.builder.build_gep(ptr, &[index], "gep index") };
        self.builder
            .build_store(address, registers[store.rs as usize]);
    }

    fn jit_compile_add(&self, registers: &Register) {}

    fn jit_compile_sum(&self) -> Option<JitFunction<SumFunc>> {
        let i64_type = self.context.i64_type();
        let fn_type = i64_type.fn_type(&[i64_type.into(), i64_type.into(), i64_type.into()], false);
        let function = self.module.add_function("sum", fn_type, None);
        let basic_block = self.context.append_basic_block(function, "entry");

        self.builder.position_at_end(basic_block);

        let x = function.get_nth_param(0)?.into_int_value();
        let y = function.get_nth_param(1)?.into_int_value();
        let z = function.get_nth_param(2)?.into_int_value();

        let sum = self.builder.build_int_add(x, y, "sum");
        let sum = self.builder.build_int_add(sum, z, "sum");

        self.builder.build_return(Some(&sum));

        unsafe { self.execution_engine.get_function("sum").ok() }
    }
}

pub fn main() -> Result<(), Box<dyn Error>> {
    let context = Context::create();
    let module = context.create_module("program");
    let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None)?;
    let codegen = CodeGen {
        context: &context,
        module,
        builder: context.create_builder(),
        execution_engine,
    };

    let mut memory = [0u8; 64];
    memory[0] = 11;

    let instructions = [
        Instruction::AddI(Immediate {
            value: 33,
            rs: 0,
            rd: 1,
        }),
        Instruction::Store(Store {
            offset: 10,
            rs: 1,
            rd: 2, // defaults to 0
        }),
    ];

    println!("Compiling program");
    let program = codegen
        .jit_compile_program(&instructions)
        .ok_or("Unable to JIT compile `program`")?;

    println!("Running program");
    unsafe {
        program.call(memory.as_mut_ptr());
    }
    println!("Expecting memory");
    println!("{:?}", memory);
    // assert_eq!(memory[2], 12);
    // assert_eq!(memory[1], 11);

    Ok(())

    // let context = Context::create();
    // let module = context.create_module("sum");
    // let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None)?;
    // let codegen = CodeGen {
    //     context: &context,
    //     module,
    //     builder: context.create_builder(),
    //     execution_engine,
    // };

    // let sum = codegen
    //     .jit_compile_sum()
    //     .ok_or("Unable to JIT compile `sum`")?;

    // let x = 1u64;
    // let y = 2u64;
    // let z = 3u64;

    // unsafe {
    //     println!("{} + {} + {} = {}", x, y, z, sum.call(x, y, z));
    //     assert_eq!(sum.call(x, y, z), x + y + z);
    // }

    // Ok(())
}

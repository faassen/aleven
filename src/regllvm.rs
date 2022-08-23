use crate::reglang::{Immediate, Instruction, Load, Register, Store};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::Module;
use inkwell::values::{IntValue, PointerValue};
use inkwell::{AddressSpace, IntPredicate, OptimizationLevel};
use std::error::Error;

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
                Instruction::SltI(immediate) => self.jit_compile_slti(&mut registers, immediate),
                Instruction::AndI(immediate) => self.jit_compile_andi(&mut registers, immediate),
                Instruction::Add(register) => self.jit_compile_add(&mut registers, register),
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

    fn jit_compile_immediate(
        &self,
        registers: &mut Registers<'ctx>,
        immediate: &Immediate,
        f: fn(&Builder<'ctx>, IntValue<'ctx>, IntValue<'ctx>) -> IntValue<'ctx>,
    ) {
        let i16_type = self.context.i16_type();
        let value = i16_type.const_int(immediate.value as u64, false);
        let rs = registers[immediate.rs as usize];
        let result = f(&self.builder, value, rs);
        registers[immediate.rd as usize] = result;
    }

    fn jit_compile_addi(&self, registers: &mut Registers<'ctx>, immediate: &Immediate) {
        self.jit_compile_immediate(registers, immediate, |builder, a, b| {
            builder.build_int_add(a, b, "addi")
        });
    }

    fn jit_compile_slti(&self, registers: &mut Registers<'ctx>, immediate: &Immediate) {
        self.jit_compile_immediate(registers, immediate, |builder, a, b| {
            builder.build_int_compare(IntPredicate::SLT, b, a, "slti")
        });
    }

    fn jit_compile_andi(&self, registers: &mut Registers<'ctx>, immediate: &Immediate) {
        self.jit_compile_immediate(registers, immediate, |builder, a, b| {
            builder.build_and(b, a, "andi")
        });
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

    fn jit_compile_add(&self, registers: &mut Registers<'ctx>, register: &Register) {
        let rs1 = registers[register.rs1 as usize];
        let rs2 = registers[register.rs2 as usize];
        let sum = self.builder.build_int_add(rs1, rs2, "add");
        registers[register.rd as usize] = sum;
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

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::reglang::{Processor, Program};
    use parameterized::parameterized;

    type Runner = fn(&[Instruction], &mut [u8]);

    use super::*;

    fn create_codegen(context: &Context) -> CodeGen<'_> {
        let module = context.create_module("program");
        let execution_engine = module
            .create_jit_execution_engine(OptimizationLevel::None)
            .expect("Execution engine couldn't be built");
        CodeGen {
            context,
            module,
            builder: context.create_builder(),
            execution_engine,
        }
    }

    fn run_llvm(instructions: &[Instruction], memory: &mut [u8]) {
        let context = Context::create();
        let codegen = create_codegen(&context);
        let program = codegen
            .jit_compile_program(instructions)
            .expect("Unable to JIT compile `program`");

        unsafe {
            program.call(memory.as_mut_ptr());
        }
    }

    fn run_interpreter(instructions: &[Instruction], memory: &mut [u8]) {
        let mut processor = Processor::new();
        Program {
            instructions: instructions.to_vec(),
        }
        .execute(&mut processor, memory);
    }

    #[parameterized(runner={run_llvm, run_interpreter})]
    fn test_add_immediate(runner: Runner) {
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

        let mut memory = [0u8; 64];
        runner(&instructions, &mut memory);
        assert_eq!(memory[10], 33);
    }

    #[parameterized(runner={run_llvm, run_interpreter})]
    fn test_add_immediate_register_has_value(runner: Runner) {
        let instructions = [
            Instruction::AddI(Immediate {
                value: 10,
                rs: 0,
                rd: 0,
            }),
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

        let mut memory = [0u8; 64];
        runner(&instructions, &mut memory);
        assert_eq!(memory[10], 43);
    }

    #[parameterized(runner={run_llvm, run_interpreter})]
    fn test_add_immediate_register_rs_is_rd(runner: Runner) {
        let instructions = [
            Instruction::AddI(Immediate {
                value: 10,
                rs: 0,
                rd: 0,
            }),
            Instruction::AddI(Immediate {
                value: 33,
                rs: 0,
                rd: 0,
            }),
            Instruction::Store(Store {
                offset: 10,
                rs: 0,
                rd: 1, // defaults to 0
            }),
        ];

        let mut memory = [0u8; 64];
        runner(&instructions, &mut memory);
        assert_eq!(memory[10], 43);
    }

    #[parameterized(runner={run_llvm, run_interpreter})]
    fn test_add_immediate_register_dec(runner: Runner) {
        let instructions = [
            Instruction::AddI(Immediate {
                value: 10,
                rs: 0,
                rd: 0,
            }),
            Instruction::AddI(Immediate {
                value: -1,
                rs: 0,
                rd: 0,
            }),
            Instruction::Store(Store {
                offset: 10,
                rs: 0,
                rd: 1, // defaults to 0
            }),
        ];

        let mut memory = [0u8; 64];
        runner(&instructions, &mut memory);
        assert_eq!(memory[10], 9);
    }

    #[parameterized(runner={run_llvm, run_interpreter})]
    fn test_slt_immediate_less(runner: Runner) {
        let instructions = [
            Instruction::SltI(Immediate {
                value: 5,
                rs: 0,
                rd: 1,
            }),
            Instruction::Store(Store {
                offset: 10,
                rs: 1,
                rd: 2, // defaults to 0
            }),
        ];
        let mut memory = [0u8; 64];
        runner(&instructions, &mut memory);
        assert_eq!(memory[10], 1);
    }

    #[parameterized(runner={run_llvm, run_interpreter})]
    fn test_slt_immediate_equal(runner: Runner) {
        let instructions = [
            Instruction::AddI(Immediate {
                value: 5,
                rs: 0,
                rd: 0,
            }),
            Instruction::SltI(Immediate {
                value: 5,
                rs: 0,
                rd: 1,
            }),
            Instruction::Store(Store {
                offset: 10,
                rs: 1,
                rd: 2, // defaults to 0
            }),
        ];
        let mut memory = [0u8; 64];
        runner(&instructions, &mut memory);
        assert_eq!(memory[10], 0);
    }

    #[parameterized(runner={run_llvm, run_interpreter})]
    fn test_slt_immediate_greater(runner: Runner) {
        let instructions = [
            Instruction::AddI(Immediate {
                value: 6,
                rs: 0,
                rd: 0,
            }),
            Instruction::SltI(Immediate {
                value: 5,
                rs: 0,
                rd: 1,
            }),
            Instruction::Store(Store {
                offset: 10,
                rs: 1,
                rd: 2, // defaults to 0
            }),
        ];
        let mut memory = [0u8; 64];
        runner(&instructions, &mut memory);
        assert_eq!(memory[10], 0);
    }

    #[parameterized(runner={run_llvm, run_interpreter})]
    fn test_and_immediate(runner: Runner) {
        let instructions = [
            Instruction::AddI(Immediate {
                value: 0b1010101,
                rs: 0,
                rd: 0,
            }),
            Instruction::AndI(Immediate {
                value: 0b1111110,
                rs: 0,
                rd: 1,
            }),
            Instruction::Store(Store {
                offset: 10,
                rs: 1,
                rd: 2, // defaults to 0
            }),
        ];
        let mut memory = [0u8; 64];
        runner(&instructions, &mut memory);
        assert_eq!(memory[10], 0b1010100);
    }

    #[parameterized(runner={run_llvm, run_interpreter})]
    fn test_add(runner: Runner) {
        let instructions = [
            Instruction::AddI(Immediate {
                value: 33,
                rs: 0,
                rd: 1,
            }),
            Instruction::AddI(Immediate {
                value: 44,
                rs: 0,
                rd: 2,
            }),
            Instruction::Add(Register {
                rs1: 1,
                rs2: 2,
                rd: 3,
            }),
            Instruction::Store(Store {
                offset: 10,
                rs: 3,
                rd: 4, // defaults to 0
            }),
        ];

        let mut memory = [0u8; 64];
        runner(&instructions, &mut memory);
        assert_eq!(memory[10], 77);
    }
}

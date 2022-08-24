use crate::reglang::{Branch, BranchTarget, Immediate, Instruction, Load, Register, Store};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::Module;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
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

type Build2<'ctx> = fn(&Builder<'ctx>, IntValue<'ctx>, IntValue<'ctx>) -> IntValue<'ctx>;

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
                Instruction::Addi(immediate) => self.jit_compile_addi(&mut registers, immediate),
                Instruction::Slti(immediate) => self.jit_compile_slti(&mut registers, immediate),
                Instruction::Sltiu(immediate) => self.jit_compile_sltiu(&mut registers, immediate),
                Instruction::Andi(immediate) => self.jit_compile_andi(&mut registers, immediate),
                Instruction::Ori(immediate) => self.jit_compile_ori(&mut registers, immediate),
                Instruction::Xori(immediate) => self.jit_compile_xori(&mut registers, immediate),
                Instruction::Slli(immediate) => self.jit_compile_slli(&mut registers, immediate),
                Instruction::Srli(immediate) => self.jit_compile_srli(&mut registers, immediate),
                Instruction::Srai(immediate) => self.jit_compile_srai(&mut registers, immediate),
                Instruction::Add(register) => self.jit_compile_add(&mut registers, register),
                Instruction::Sub(register) => self.jit_compile_sub(&mut registers, register),
                Instruction::Slt(register) => self.jit_compile_slt(&mut registers, register),
                Instruction::Sltu(register) => self.jit_compile_sltu(&mut registers, register),
                Instruction::And(register) => self.jit_compile_and(&mut registers, register),
                Instruction::Or(register) => self.jit_compile_or(&mut registers, register),
                Instruction::Xor(register) => self.jit_compile_xor(&mut registers, register),
                Instruction::Sll(register) => self.jit_compile_sll(&mut registers, register),
                Instruction::Srl(register) => self.jit_compile_srl(&mut registers, register),
                Instruction::Sra(register) => self.jit_compile_sra(&mut registers, register),
                Instruction::Lb(load) => {
                    self.jit_compile_lb(&mut registers, ptr, load);
                }
                Instruction::Lbu(load) => {
                    self.jit_compile_lbu(&mut registers, ptr, load);
                }
                Instruction::Sb(store) => {
                    self.jit_compile_sb(&registers, ptr, store);
                }
                Instruction::Lh(load) => {
                    self.jit_compile_lh(&mut registers, ptr, load);
                }
                Instruction::Sh(store) => {
                    self.jit_compile_sh(&registers, ptr, store);
                }
                _ => {}
            }
        }
        self.builder.build_return(None);

        // self.module.print_to_stderr();
        // save_asm(&self.module)

        unsafe { self.execution_engine.get_function("program").ok() }
    }

    fn jit_compile_immediate(
        &self,
        registers: &mut Registers<'ctx>,
        immediate: &Immediate,
        f: Build2<'ctx>,
    ) {
        let i16_type = self.context.i16_type();
        let value = i16_type.const_int(immediate.value as u64, false);
        let rs = registers[immediate.rs as usize];
        let result = f(&self.builder, rs, value);
        registers[immediate.rd as usize] = result;
    }

    fn jit_compile_addi(&self, registers: &mut Registers<'ctx>, immediate: &Immediate) {
        self.jit_compile_immediate(registers, immediate, |builder, a, b| {
            builder.build_int_add(a, b, "addi")
        });
    }

    fn jit_compile_slti(&self, registers: &mut Registers<'ctx>, immediate: &Immediate) {
        self.jit_compile_immediate(registers, immediate, |builder, a, b| {
            builder.build_int_compare(IntPredicate::SLT, a, b, "slti")
        });
    }

    fn jit_compile_sltiu(&self, registers: &mut Registers<'ctx>, immediate: &Immediate) {
        self.jit_compile_immediate(registers, immediate, |builder, a, b| {
            builder.build_int_compare(IntPredicate::ULT, a, b, "sltiu")
        });
    }

    fn jit_compile_andi(&self, registers: &mut Registers<'ctx>, immediate: &Immediate) {
        self.jit_compile_immediate(registers, immediate, |builder, a, b| {
            builder.build_and(a, b, "andi")
        });
    }

    fn jit_compile_ori(&self, registers: &mut Registers<'ctx>, immediate: &Immediate) {
        self.jit_compile_immediate(registers, immediate, |builder, a, b| {
            builder.build_or(a, b, "ori")
        });
    }

    fn jit_compile_xori(&self, registers: &mut Registers<'ctx>, immediate: &Immediate) {
        self.jit_compile_immediate(registers, immediate, |builder, a, b| {
            builder.build_xor(a, b, "xori")
        });
    }

    fn jit_compile_slli(&self, registers: &mut Registers<'ctx>, immediate: &Immediate) {
        self.jit_compile_immediate(registers, immediate, |builder, a, b| {
            builder.build_left_shift(a, b, "slli")
        });
    }

    fn jit_compile_srli(&self, registers: &mut Registers<'ctx>, immediate: &Immediate) {
        self.jit_compile_immediate(registers, immediate, |builder, a, b| {
            builder.build_right_shift(a, b, false, "srli")
        });
    }

    fn jit_compile_srai(&self, registers: &mut Registers<'ctx>, immediate: &Immediate) {
        self.jit_compile_immediate(registers, immediate, |builder, a, b| {
            builder.build_right_shift(a, b, true, "srai")
        });
    }
    fn jit_compile_register(
        &self,
        registers: &mut Registers<'ctx>,
        register: &Register,
        f: Build2<'ctx>,
    ) {
        let rs1 = registers[register.rs1 as usize];
        let rs2 = registers[register.rs2 as usize];
        let result = f(&self.builder, rs1, rs2);
        registers[register.rd as usize] = result;
    }

    fn jit_compile_add(&self, registers: &mut Registers<'ctx>, register: &Register) {
        self.jit_compile_register(registers, register, |builder, a, b| {
            builder.build_int_add(a, b, "add")
        });
    }
    fn jit_compile_sub(&self, registers: &mut Registers<'ctx>, register: &Register) {
        self.jit_compile_register(registers, register, |builder, a, b| {
            builder.build_int_sub(a, b, "sub")
        });
    }
    fn jit_compile_slt(&self, registers: &mut Registers<'ctx>, register: &Register) {
        self.jit_compile_register(registers, register, |builder, a, b| {
            builder.build_int_compare(IntPredicate::SLT, a, b, "slt")
        });
    }
    fn jit_compile_sltu(&self, registers: &mut Registers<'ctx>, register: &Register) {
        self.jit_compile_register(registers, register, |builder, a, b| {
            builder.build_int_compare(IntPredicate::ULT, a, b, "sltu")
        });
    }
    fn jit_compile_and(&self, registers: &mut Registers<'ctx>, register: &Register) {
        self.jit_compile_register(registers, register, |builder, a, b| {
            builder.build_and(a, b, "and")
        });
    }
    fn jit_compile_or(&self, registers: &mut Registers<'ctx>, register: &Register) {
        self.jit_compile_register(registers, register, |builder, a, b| {
            builder.build_or(a, b, "and")
        });
    }
    fn jit_compile_xor(&self, registers: &mut Registers<'ctx>, register: &Register) {
        self.jit_compile_register(registers, register, |builder, a, b| {
            builder.build_xor(a, b, "and")
        });
    }
    fn jit_compile_sll(&self, registers: &mut Registers<'ctx>, register: &Register) {
        self.jit_compile_register(registers, register, |builder, a, b| {
            builder.build_left_shift(a, b, "and")
        });
    }
    fn jit_compile_srl(&self, registers: &mut Registers<'ctx>, register: &Register) {
        self.jit_compile_register(registers, register, |builder, a, b| {
            builder.build_right_shift(a, b, false, "and")
        });
    }
    fn jit_compile_sra(&self, registers: &mut Registers<'ctx>, register: &Register) {
        self.jit_compile_register(registers, register, |builder, a, b| {
            builder.build_right_shift(a, b, true, "and")
        });
    }
    fn jit_compile_lb(
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
        let value = self.builder.build_load(address, "lb");
        let extended_value = self.builder.build_int_s_extend(
            value.into_int_value(),
            self.context.i16_type(),
            "extend",
        );
        registers[load.rd as usize] = extended_value;
    }

    fn jit_compile_lbu(
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
        let value = self.builder.build_load(address, "lb");
        let extended_value = self.builder.build_int_z_extend(
            value.into_int_value(),
            self.context.i16_type(),
            "extend",
        );
        registers[load.rd as usize] = extended_value;
    }

    fn jit_compile_sb(&self, registers: &Registers, ptr: PointerValue, store: &Store) {
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

    fn jit_compile_lh(
        &self,
        registers: &mut Registers<'ctx>,
        ptr: PointerValue<'ctx>,
        load: &Load,
    ) {
        let i16_type = self.context.i16_type();
        let i16_ptr_type = i16_type.ptr_type(AddressSpace::Generic);
        let i16_ptr = self.builder.build_pointer_cast(ptr, i16_ptr_type, "lh_ptr");
        let offset = self.context.i16_type().const_int(load.offset as u64, false);
        let index = self
            .builder
            .build_int_add(offset, registers[load.rs as usize], "index");
        let address = unsafe { self.builder.build_gep(i16_ptr, &[index], "gep index") };
        let value = self.builder.build_load(address, "lh");
        registers[load.rd as usize] = value.into_int_value();
    }

    fn jit_compile_sh(&self, registers: &Registers, ptr: PointerValue, store: &Store) {
        let i16_type = self.context.i16_type();
        let i16_ptr_type = i16_type.ptr_type(AddressSpace::Generic);
        let i16_ptr = self.builder.build_pointer_cast(ptr, i16_ptr_type, "sh_ptr");
        let offset = self
            .context
            .i16_type()
            .const_int(store.offset as u64, false);
        let index = self
            .builder
            .build_int_add(offset, registers[store.rd as usize], "index");
        let address = unsafe { self.builder.build_gep(i16_ptr, &[index], "gep index") };
        self.builder
            .build_store(address, registers[store.rs as usize]);
    }
}

fn save_asm(module: &Module) {
    Target::initialize_native(&InitializationConfig::default())
        .expect("Failed to initialize native target");

    let triple = TargetMachine::get_default_triple();
    let cpu = TargetMachine::get_host_cpu_name().to_string();
    let features = TargetMachine::get_host_cpu_features().to_string();

    let target = Target::from_triple(&triple).unwrap();
    let machine = target
        .create_target_machine(
            &triple,
            &cpu,
            &features,
            OptimizationLevel::Aggressive,
            RelocMode::Default,
            CodeModel::Default,
        )
        .unwrap();
    machine
        .write_to_file(module, FileType::Assembly, "out.asm".as_ref())
        .unwrap();
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
        Instruction::Lb(Load {
            rs: 0,
            rd: 1,
            offset: 0,
        }),
        Instruction::Addi(Immediate {
            value: 44,
            rs: 0,
            rd: 2,
        }),
        Instruction::Add(Register {
            rs1: 1,
            rs2: 2,
            rd: 3,
        }),
        Instruction::Sb(Store {
            offset: 10,
            rs: 3,
            rd: 4, // defaults to 0
        }),
    ];
    // let instructions = [
    //     Instruction::Addi(Immediate {
    //         value: 33,
    //         rs: 0,
    //         rd: 1,
    //     }),
    //     Instruction::Sb(Store {
    //         offset: 10,
    //         rs: 1,
    //         rd: 2, // defaults to 0
    //     }),
    // ];

    println!("Compiling program");
    let program = codegen
        .jit_compile_program(&instructions)
        .ok_or("Unable to JIT compile `program`")?;

    println!("Running program");
    unsafe {
        program.call(memory.as_mut_ptr());
    }
    println!("Memory");
    println!("{:?}", memory);

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::reglang::{Processor, Program};
    use byteorder::{ByteOrder, LittleEndian};
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
            Instruction::Addi(Immediate {
                value: 33,
                rs: 0,
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
        assert_eq!(memory[10], 33);
    }

    #[parameterized(runner={run_llvm, run_interpreter})]
    fn test_add_immediate_register_has_value(runner: Runner) {
        let instructions = [
            Instruction::Addi(Immediate {
                value: 10,
                rs: 0,
                rd: 0,
            }),
            Instruction::Addi(Immediate {
                value: 33,
                rs: 0,
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
    fn test_add_immediate_register_rs_is_rd(runner: Runner) {
        let instructions = [
            Instruction::Addi(Immediate {
                value: 10,
                rs: 0,
                rd: 0,
            }),
            Instruction::Addi(Immediate {
                value: 33,
                rs: 0,
                rd: 0,
            }),
            Instruction::Sb(Store {
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
            Instruction::Addi(Immediate {
                value: 10,
                rs: 0,
                rd: 0,
            }),
            Instruction::Addi(Immediate {
                value: -1,
                rs: 0,
                rd: 0,
            }),
            Instruction::Sb(Store {
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
            Instruction::Slti(Immediate {
                value: 5,
                rs: 0,
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
        assert_eq!(memory[10], 1);
    }

    #[parameterized(runner={run_llvm, run_interpreter})]
    fn test_slt_immediate_less_negative(runner: Runner) {
        let instructions = [
            Instruction::Slti(Immediate {
                value: -4,
                rs: 0,
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
        assert_eq!(memory[10], 0);
    }

    #[parameterized(runner={run_llvm, run_interpreter})]
    fn test_sltu_immediate_less(runner: Runner) {
        let instructions = [
            Instruction::Sltiu(Immediate {
                value: -4, // treated as large number instead
                rs: 0,
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
        assert_eq!(memory[10], 1);
    }

    #[parameterized(runner={run_llvm, run_interpreter})]
    fn test_slt_immediate_equal(runner: Runner) {
        let instructions = [
            Instruction::Addi(Immediate {
                value: 5,
                rs: 0,
                rd: 0,
            }),
            Instruction::Slti(Immediate {
                value: 5,
                rs: 0,
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
        assert_eq!(memory[10], 0);
    }

    #[parameterized(runner={run_llvm, run_interpreter})]
    fn test_slt_immediate_greater(runner: Runner) {
        let instructions = [
            Instruction::Addi(Immediate {
                value: 6,
                rs: 0,
                rd: 0,
            }),
            Instruction::Slti(Immediate {
                value: 5,
                rs: 0,
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
        assert_eq!(memory[10], 0);
    }

    #[parameterized(runner={run_llvm, run_interpreter})]
    fn test_and_immediate(runner: Runner) {
        let instructions = [
            Instruction::Addi(Immediate {
                value: 0b1010101,
                rs: 0,
                rd: 0,
            }),
            Instruction::Andi(Immediate {
                value: 0b1111110,
                rs: 0,
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
        assert_eq!(memory[10], 0b1010100);
    }

    #[parameterized(runner={run_llvm, run_interpreter})]
    fn test_or_immediate(runner: Runner) {
        let instructions = [
            Instruction::Addi(Immediate {
                value: 0b1010100,
                rs: 0,
                rd: 0,
            }),
            Instruction::Ori(Immediate {
                value: 0b1111110,
                rs: 0,
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
        assert_eq!(memory[10], 0b1111110);
    }

    #[parameterized(runner={run_llvm, run_interpreter})]
    fn test_xor_immediate(runner: Runner) {
        let instructions = [
            Instruction::Addi(Immediate {
                value: 0b1010100,
                rs: 0,
                rd: 0,
            }),
            Instruction::Xori(Immediate {
                value: 0b1111010,
                rs: 0,
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
        assert_eq!(memory[10], 0b0101110);
    }

    #[parameterized(runner={run_llvm, run_interpreter})]
    fn test_sll_immediate(runner: Runner) {
        let instructions = [
            Instruction::Addi(Immediate {
                value: 5,
                rs: 0,
                rd: 0,
            }),
            Instruction::Slli(Immediate {
                value: 2,
                rs: 0,
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
        assert_eq!(memory[10], 20);
    }

    #[parameterized(runner={run_llvm, run_interpreter})]
    fn test_sra_immediate(runner: Runner) {
        let instructions = [
            Instruction::Addi(Immediate {
                value: 20,
                rs: 0,
                rd: 0,
            }),
            Instruction::Srai(Immediate {
                value: 2,
                rs: 0,
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
        assert_eq!(memory[10], 5);
    }

    #[parameterized(runner={run_llvm, run_interpreter})]
    fn test_lb(runner: Runner) {
        let instructions = [
            Instruction::Lb(Load {
                offset: 0,
                rs: 0,
                rd: 1,
            }),
            Instruction::Sb(Store {
                offset: 10,
                rs: 1,
                rd: 2, // defaults to 0
            }),
        ];
        let mut memory = [0u8; 64];
        memory[0] = 11;
        runner(&instructions, &mut memory);
        assert_eq!(memory[10], 11);
    }

    #[parameterized(runner={run_llvm, run_interpreter})]
    fn test_lh_sh(runner: Runner) {
        let instructions = [
            Instruction::Lh(Load {
                offset: 0,
                rs: 0,
                rd: 1,
            }),
            Instruction::Sh(Store {
                offset: 10,
                rs: 1,
                rd: 2, // defaults to 0
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
                rs: 0,
                rd: 1,
            }),
            Instruction::Sh(Store {
                offset: 10,
                rs: 1,
                rd: 2, // defaults to 0
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
    fn test_sh_aligns(runner: Runner) {
        let instructions = [
            Instruction::Lh(Load {
                offset: 0,
                rs: 0,
                rd: 1,
            }),
            // it's not possible to misalign this as it's already even
            Instruction::Sh(Store {
                offset: 11,
                rs: 1,
                rd: 2, // defaults to 0
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
    fn test_lb_sign_extends(runner: Runner) {
        let instructions = [
            Instruction::Lb(Load {
                offset: 0,
                rs: 0,
                rd: 1,
            }),
            Instruction::Sh(Store {
                offset: 10,
                rs: 1,
                rd: 2, // defaults to 0
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
                rs: 0,
                rd: 1,
            }),
            Instruction::Sh(Store {
                offset: 10,
                rs: 1,
                rd: 2, // defaults to 0
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
                rs: 0,
                rd: 1,
            }),
            Instruction::Srai(Immediate {
                value: 2,
                rs: 1,
                rd: 1,
            }),
            Instruction::Sh(Store {
                offset: 10,
                rs: 1,
                rd: 2, // defaults to 0
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
                rs: 0,
                rd: 1,
            }),
            Instruction::Srai(Immediate {
                value: 2,
                rs: 1,
                rd: 1,
            }),
            Instruction::Sh(Store {
                offset: 10,
                rs: 1,
                rd: 2, // defaults to 0
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
                rs: 0,
                rd: 1,
            }),
            Instruction::Srli(Immediate {
                value: 2,
                rs: 1,
                rd: 1,
            }),
            Instruction::Sh(Store {
                offset: 10,
                rs: 1,
                rd: 2, // defaults to 0
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
                rs: 0,
                rd: 1,
            }),
            Instruction::Addi(Immediate {
                value: 44,
                rs: 0,
                rd: 2,
            }),
            Instruction::Add(Register {
                rs1: 1,
                rs2: 2,
                rd: 3,
            }),
            Instruction::Sb(Store {
                offset: 10,
                rs: 3,
                rd: 4, // defaults to 0
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
                rs: 0,
                rd: 1,
            }),
            Instruction::Addi(Immediate {
                value: -11,
                rs: 0,
                rd: 2,
            }),
            Instruction::Add(Register {
                rs1: 1,
                rs2: 2,
                rd: 3,
            }),
            Instruction::Sb(Store {
                offset: 10,
                rs: 3,
                rd: 4, // defaults to 0
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
                rs: 0,
                rd: 1,
            }),
            Instruction::Addi(Immediate {
                value: 11,
                rs: 0,
                rd: 2,
            }),
            Instruction::Sub(Register {
                rs1: 1,
                rs2: 2,
                rd: 3,
            }),
            Instruction::Sb(Store {
                offset: 10,
                rs: 3,
                rd: 4, // defaults to 0
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
                rs: 0,
                rd: 1,
            }),
            Instruction::Addi(Immediate {
                value: 1,
                rs: 0,
                rd: 2,
            }),
            Instruction::Add(Register {
                rs1: 1,
                rs2: 2,
                rd: 3,
            }),
            Instruction::Sh(Store {
                offset: 10,
                rs: 3,
                rd: 4, // defaults to 0
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
                rs: 0,
                rd: 1,
            }),
            Instruction::Addi(Immediate {
                value: 255,
                rs: 0,
                rd: 2,
            }),
            Instruction::Add(Register {
                rs1: 1,
                rs2: 2,
                rd: 3,
            }),
            Instruction::Sh(Store {
                offset: 10,
                rs: 3,
                rd: 4, // defaults to 0
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
                rs: 0,
                rd: 1,
            }),
            Instruction::Addi(Immediate {
                value: 44,
                rs: 0,
                rd: 2,
            }),
            Instruction::Slt(Register {
                rs1: 1,
                rs2: 2,
                rd: 3,
            }),
            Instruction::Sb(Store {
                offset: 10,
                rs: 3,
                rd: 4, // defaults to 0
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
                rs: 0,
                rd: 1,
            }),
            Instruction::Addi(Immediate {
                value: 44,
                rs: 0,
                rd: 2,
            }),
            Instruction::Slt(Register {
                rs1: 1,
                rs2: 2,
                rd: 3,
            }),
            Instruction::Sb(Store {
                offset: 10,
                rs: 3,
                rd: 4, // defaults to 0
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
                rs: 0,
                rd: 1,
            }),
            Instruction::Addi(Immediate {
                value: 44,
                rs: 0,
                rd: 2,
            }),
            Instruction::Slt(Register {
                rs1: 1,
                rs2: 2,
                rd: 3,
            }),
            Instruction::Sb(Store {
                offset: 10,
                rs: 3,
                rd: 4, // defaults to 0
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
                rs: 0,
                rd: 1,
            }),
            Instruction::Addi(Immediate {
                value: 33,
                rs: 0,
                rd: 2,
            }),
            Instruction::Slt(Register {
                rs1: 1,
                rs2: 2,
                rd: 3,
            }),
            Instruction::Sb(Store {
                offset: 10,
                rs: 3,
                rd: 4, // defaults to 0
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
                rs: 0,
                rd: 1,
            }),
            Instruction::Addi(Immediate {
                value: 44,
                rs: 0,
                rd: 2,
            }),
            Instruction::Sltu(Register {
                rs1: 1,
                rs2: 2,
                rd: 3,
            }),
            Instruction::Sb(Store {
                offset: 10,
                rs: 3,
                rd: 4, // defaults to 0
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
                rs: 0,
                rd: 1,
            }),
            Instruction::Addi(Immediate {
                value: 44,
                rs: 0,
                rd: 2,
            }),
            Instruction::Sltu(Register {
                rs1: 1,
                rs2: 2,
                rd: 3,
            }),
            Instruction::Sb(Store {
                offset: 10,
                rs: 3,
                rd: 4, // defaults to 0
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
                rs: 0,
                rd: 1,
            }),
            Instruction::Addi(Immediate {
                value: 44,
                rs: 0,
                rd: 2,
            }),
            Instruction::Sltu(Register {
                rs1: 1,
                rs2: 2,
                rd: 3,
            }),
            Instruction::Sb(Store {
                offset: 10,
                rs: 3,
                rd: 4, // defaults to 0
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
                rs: 0,
                rd: 1,
            }),
            Instruction::Addi(Immediate {
                value: 33,
                rs: 0,
                rd: 2,
            }),
            Instruction::Sltu(Register {
                rs1: 1,
                rs2: 2,
                rd: 3,
            }),
            Instruction::Sb(Store {
                offset: 10,
                rs: 3,
                rd: 4, // defaults to 0
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
                rs: 0,
                rd: 1,
            }),
            Instruction::Addi(Immediate {
                value: 0b1111110,
                rs: 0,
                rd: 2,
            }),
            Instruction::And(Register {
                rs1: 1,
                rs2: 2,
                rd: 3,
            }),
            Instruction::Sb(Store {
                offset: 10,
                rs: 3,
                rd: 4, // defaults to 0
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
                rs: 0,
                rd: 1,
            }),
            Instruction::Addi(Immediate {
                value: 0b1111110,
                rs: 0,
                rd: 2,
            }),
            Instruction::Or(Register {
                rs1: 1,
                rs2: 2,
                rd: 3,
            }),
            Instruction::Sb(Store {
                offset: 10,
                rs: 3,
                rd: 4, // defaults to 0
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
                rs: 0,
                rd: 1,
            }),
            Instruction::Addi(Immediate {
                value: 0b1010100,
                rs: 0,
                rd: 2,
            }),
            Instruction::Xor(Register {
                rs1: 1,
                rs2: 2,
                rd: 3,
            }),
            Instruction::Sb(Store {
                offset: 10,
                rs: 3,
                rd: 4, // defaults to 0
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
                rs: 0,
                rd: 1,
            }),
            Instruction::Addi(Immediate {
                value: 2,
                rs: 0,
                rd: 2,
            }),
            Instruction::Sll(Register {
                rs1: 1,
                rs2: 2,
                rd: 3,
            }),
            Instruction::Sb(Store {
                offset: 10,
                rs: 3,
                rd: 4, // defaults to 0
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
                rs: 0,
                rd: 1,
            }),
            Instruction::Addi(Immediate {
                value: 100,
                rs: 0,
                rd: 2,
            }),
            Instruction::Sll(Register {
                rs1: 1,
                rs2: 2,
                rd: 3,
            }),
            Instruction::Sb(Store {
                offset: 10,
                rs: 3,
                rd: 4, // defaults to 0
            }),
        ];

        let mut memory = [0u8; 64];
        runner(&instructions, &mut memory);
        assert_eq!(memory[10], 0);
    }

    #[parameterized(runner={run_llvm, run_interpreter})]
    fn test_srl(runner: Runner) {
        let instructions = [
            Instruction::Addi(Immediate {
                value: 0b10100,
                rs: 0,
                rd: 1,
            }),
            Instruction::Addi(Immediate {
                value: 2,
                rs: 0,
                rd: 2,
            }),
            Instruction::Srl(Register {
                rs1: 1,
                rs2: 2,
                rd: 3,
            }),
            Instruction::Sb(Store {
                offset: 10,
                rs: 3,
                rd: 4, // defaults to 0
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
                rs: 0,
                rd: 1,
            }),
            Instruction::Addi(Immediate {
                value: 100,
                rs: 0,
                rd: 2,
            }),
            Instruction::Srl(Register {
                rs1: 1,
                rs2: 2,
                rd: 3,
            }),
            Instruction::Sb(Store {
                offset: 10,
                rs: 3,
                rd: 4, // defaults to 0
            }),
        ];

        let mut memory = [0u8; 64];
        runner(&instructions, &mut memory);
        assert_eq!(memory[10], 0);
    }

    #[parameterized(runner={run_llvm, run_interpreter})]
    fn test_srl_negative(runner: Runner) {
        let instructions = [
            Instruction::Addi(Immediate {
                value: -20,
                rs: 0,
                rd: 1,
            }),
            Instruction::Addi(Immediate {
                value: 2,
                rs: 0,
                rd: 2,
            }),
            Instruction::Srl(Register {
                rs1: 1,
                rs2: 2,
                rd: 3,
            }),
            Instruction::Sh(Store {
                offset: 10,
                rs: 3,
                rd: 4, // defaults to 0
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
                rs: 0,
                rd: 1,
            }),
            Instruction::Addi(Immediate {
                value: 2,
                rs: 0,
                rd: 2,
            }),
            Instruction::Sra(Register {
                rs1: 1,
                rs2: 2,
                rd: 3,
            }),
            Instruction::Sb(Store {
                offset: 10,
                rs: 3,
                rd: 4, // defaults to 0
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
                rs: 0,
                rd: 1,
            }),
            Instruction::Addi(Immediate {
                value: 2,
                rs: 0,
                rd: 2,
            }),
            Instruction::Sra(Register {
                rs1: 1,
                rs2: 2,
                rd: 3,
            }),
            Instruction::Sh(Store {
                offset: 10,
                rs: 3,
                rd: 4, // defaults to 0
            }),
        ];

        let mut memory = [0u8; 64];
        runner(&instructions, &mut memory);
        let value = LittleEndian::read_i16(&memory[20..]);
        assert_eq!(value, -5);
    }

    #[parameterized(runner={run_interpreter})]
    fn test_beq(runner: Runner) {
        let instructions = [
            Instruction::Lb(Load {
                offset: 0,
                rs: 0,
                rd: 1,
            }),
            Instruction::Lb(Load {
                offset: 1,
                rs: 0,
                rd: 2,
            }),
            Instruction::Beq(Branch {
                rs1: 1,
                rs2: 2,
                target: 1,
            }),
            Instruction::Lb(Load {
                offset: 2,
                rs: 0,
                rd: 3,
            }),
            Instruction::Sb(Store {
                offset: 10,
                rs: 3,
                rd: 4, // defaults to 0
            }),
            Instruction::Target(BranchTarget::new(1)),
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
}

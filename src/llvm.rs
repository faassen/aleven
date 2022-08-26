use crate::lang::{Branch, Immediate, Instruction, Load, Register, Store};
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::Module;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::values::{FunctionValue, IntValue, PointerValue};
use inkwell::{AddressSpace, IntPredicate, OptimizationLevel};
use rustc_hash::FxHashMap;
use std::error::Error;

type ProgramFunc = unsafe extern "C" fn(*mut u8) -> ();

pub struct CodeGen<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub execution_engine: ExecutionEngine<'ctx>,
}

type Registers<'a> = [IntValue<'a>; 32];

type Build2<'ctx> = fn(&Builder<'ctx>, IntValue<'ctx>, IntValue<'ctx>) -> IntValue<'ctx>;
type LoadValue<'ctx> = fn(&Builder<'ctx>, &'ctx Context, PointerValue<'ctx>) -> IntValue<'ctx>;
type StoreValue<'ctx> = fn(&Builder<'ctx>, &Context, PointerValue<'ctx>, IntValue<'ctx>);

impl<'ctx> CodeGen<'ctx> {
    pub fn compile_program(
        &self,
        instructions: &[Instruction],
        memory_size: u16,
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

        let (blocks, targets) = self.get_blocks(function, instructions);

        let mut blocks_iter = blocks.iter();
        let mut block = blocks_iter.next().unwrap().1;
        self.builder.build_unconditional_branch(block);
        self.builder.position_at_end(block);

        for instruction in instructions {
            match instruction {
                Instruction::Addi(immediate) => self.compile_addi(&mut registers, immediate),
                Instruction::Slti(immediate) => self.compile_slti(&mut registers, immediate),
                Instruction::Sltiu(immediate) => self.compile_sltiu(&mut registers, immediate),
                Instruction::Andi(immediate) => self.compile_andi(&mut registers, immediate),
                Instruction::Ori(immediate) => self.compile_ori(&mut registers, immediate),
                Instruction::Xori(immediate) => self.compile_xori(&mut registers, immediate),
                Instruction::Slli(immediate) => self.compile_slli(&mut registers, immediate),
                Instruction::Srli(immediate) => self.compile_srli(&mut registers, immediate),
                Instruction::Srai(immediate) => self.compile_srai(&mut registers, immediate),
                Instruction::Add(register) => self.compile_add(&mut registers, register),
                Instruction::Sub(register) => self.compile_sub(&mut registers, register),
                Instruction::Slt(register) => self.compile_slt(&mut registers, register),
                Instruction::Sltu(register) => self.compile_sltu(&mut registers, register),
                Instruction::And(register) => self.compile_and(&mut registers, register),
                Instruction::Or(register) => self.compile_or(&mut registers, register),
                Instruction::Xor(register) => self.compile_xor(&mut registers, register),
                Instruction::Sll(register) => self.compile_sll(&mut registers, register),
                Instruction::Srl(register) => self.compile_srl(&mut registers, register),
                Instruction::Sra(register) => self.compile_sra(&mut registers, register),
                Instruction::Lb(load) => {
                    self.compile_lb(&mut registers, ptr, load, memory_size, function);
                }
                Instruction::Lbu(load) => {
                    self.compile_lbu(&mut registers, ptr, load, memory_size, function);
                }
                Instruction::Sb(store) => {
                    self.compile_sb(&registers, ptr, store, memory_size, function);
                }
                Instruction::Lh(load) => {
                    self.compile_lh(&mut registers, ptr, load, memory_size, function);
                }
                Instruction::Sh(store) => {
                    self.compile_sh(&registers, ptr, store, memory_size, function);
                }
                Instruction::Beq(branch) => {
                    block = blocks_iter.next().unwrap().1;
                    self.compile_beq(&registers, branch, block, &targets);
                    self.builder.position_at_end(block);
                }
                Instruction::Target(_target) => {
                    block = blocks_iter.next().unwrap().1;
                    self.builder.build_unconditional_branch(block);
                    self.builder.position_at_end(block);
                }
            }
        }
        self.builder.build_return(None);

        // self.module.print_to_stderr();
        // save_asm(&self.module);

        unsafe { self.execution_engine.get_function("program").ok() }
    }

    fn get_blocks(
        &self,
        parent: FunctionValue,
        instructions: &[Instruction],
    ) -> (Vec<(usize, BasicBlock)>, FxHashMap<u8, BasicBlock>) {
        let mut blocks = Vec::new();
        let mut targets = FxHashMap::default();
        let mut block_id: u16 = 0;
        blocks.push((
            0,
            self.context
                .append_basic_block(parent, &format!("block{}", block_id)),
        ));
        block_id += 1;
        for (index, instruction) in instructions.iter().enumerate() {
            match instruction {
                Instruction::Beq(_branch) => {
                    blocks.push((
                        index,
                        self.context
                            .append_basic_block(parent, &format!("block{}", block_id)),
                    ));
                    block_id += 1;
                }
                Instruction::Target(target) => {
                    let block = self
                        .context
                        .append_basic_block(parent, &format!("block{}", block_id));
                    blocks.push((index, block));
                    targets.insert(target.identifier, block);
                    block_id += 1;
                }
                _ => {}
            }
        }
        (blocks, targets)
    }

    fn compile_immediate(
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

    fn compile_immediate_shift(
        &self,
        registers: &mut Registers<'ctx>,
        immediate: &Immediate,
        f: Build2<'ctx>,
    ) {
        let i16_type = self.context.i16_type();
        let value = i16_type.const_int(immediate.value as u64, false);
        let max = i16_type.const_int(16, false);
        let zero = i16_type.const_int(0, false);
        let mvalue = self
            .builder
            .build_select(
                self.builder
                    .build_int_compare(IntPredicate::UGE, value, max, "cmp max"),
                zero,
                value,
                "max shift",
            )
            .into_int_value();
        let rs = registers[immediate.rs as usize];
        let result = f(&self.builder, rs, mvalue);
        registers[immediate.rd as usize] = result;
    }

    fn compile_addi(&self, registers: &mut Registers<'ctx>, immediate: &Immediate) {
        self.compile_immediate(registers, immediate, |builder, a, b| {
            builder.build_int_add(a, b, "addi")
        });
    }

    fn compile_slti(&self, registers: &mut Registers<'ctx>, immediate: &Immediate) {
        self.compile_immediate(registers, immediate, |builder, a, b| {
            builder.build_int_compare(IntPredicate::SLT, a, b, "slti")
        });
    }

    fn compile_sltiu(&self, registers: &mut Registers<'ctx>, immediate: &Immediate) {
        self.compile_immediate(registers, immediate, |builder, a, b| {
            builder.build_int_compare(IntPredicate::ULT, a, b, "sltiu")
        });
    }

    fn compile_andi(&self, registers: &mut Registers<'ctx>, immediate: &Immediate) {
        self.compile_immediate(registers, immediate, |builder, a, b| {
            builder.build_and(a, b, "andi")
        });
    }

    fn compile_ori(&self, registers: &mut Registers<'ctx>, immediate: &Immediate) {
        self.compile_immediate(registers, immediate, |builder, a, b| {
            builder.build_or(a, b, "ori")
        });
    }

    fn compile_xori(&self, registers: &mut Registers<'ctx>, immediate: &Immediate) {
        self.compile_immediate(registers, immediate, |builder, a, b| {
            builder.build_xor(a, b, "xori")
        });
    }

    fn compile_slli(&self, registers: &mut Registers<'ctx>, immediate: &Immediate) {
        self.compile_immediate_shift(registers, immediate, |builder, a, b| {
            builder.build_left_shift(a, b, "slli")
        });
    }

    fn compile_srli(&self, registers: &mut Registers<'ctx>, immediate: &Immediate) {
        self.compile_immediate_shift(registers, immediate, |builder, a, b| {
            builder.build_right_shift(a, b, false, "srli")
        });
    }

    fn compile_srai(&self, registers: &mut Registers<'ctx>, immediate: &Immediate) {
        self.compile_immediate_shift(registers, immediate, |builder, a, b| {
            builder.build_right_shift(a, b, true, "srai")
        });
    }

    fn compile_register(
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

    fn compile_register_shift(
        &self,
        registers: &mut Registers<'ctx>,
        register: &Register,
        f: Build2<'ctx>,
    ) {
        let i16_type = self.context.i16_type();
        let max = i16_type.const_int(16, false);
        let zero = i16_type.const_int(0, false);
        let rs1 = registers[register.rs1 as usize];
        let rs2 = registers[register.rs2 as usize];
        let mvalue = self
            .builder
            .build_select(
                self.builder
                    .build_int_compare(IntPredicate::UGE, rs2, max, "cmp max"),
                zero,
                rs2,
                "max shift",
            )
            .into_int_value();
        let result = f(&self.builder, rs1, mvalue);
        registers[register.rd as usize] = result;
    }

    fn compile_add(&self, registers: &mut Registers<'ctx>, register: &Register) {
        self.compile_register(registers, register, |builder, a, b| {
            builder.build_int_add(a, b, "add")
        });
    }
    fn compile_sub(&self, registers: &mut Registers<'ctx>, register: &Register) {
        self.compile_register(registers, register, |builder, a, b| {
            builder.build_int_sub(a, b, "sub")
        });
    }
    fn compile_slt(&self, registers: &mut Registers<'ctx>, register: &Register) {
        self.compile_register(registers, register, |builder, a, b| {
            builder.build_int_compare(IntPredicate::SLT, a, b, "slt")
        });
    }
    fn compile_sltu(&self, registers: &mut Registers<'ctx>, register: &Register) {
        self.compile_register(registers, register, |builder, a, b| {
            builder.build_int_compare(IntPredicate::ULT, a, b, "sltu")
        });
    }
    fn compile_and(&self, registers: &mut Registers<'ctx>, register: &Register) {
        self.compile_register(registers, register, |builder, a, b| {
            builder.build_and(a, b, "and")
        });
    }
    fn compile_or(&self, registers: &mut Registers<'ctx>, register: &Register) {
        self.compile_register(registers, register, |builder, a, b| {
            builder.build_or(a, b, "and")
        });
    }
    fn compile_xor(&self, registers: &mut Registers<'ctx>, register: &Register) {
        self.compile_register(registers, register, |builder, a, b| {
            builder.build_xor(a, b, "and")
        });
    }
    fn compile_sll(&self, registers: &mut Registers<'ctx>, register: &Register) {
        self.compile_register_shift(registers, register, |builder, a, b| {
            builder.build_left_shift(a, b, "and")
        });
    }
    fn compile_srl(&self, registers: &mut Registers<'ctx>, register: &Register) {
        self.compile_register_shift(registers, register, |builder, a, b| {
            builder.build_right_shift(a, b, false, "and")
        });
    }
    fn compile_sra(&self, registers: &mut Registers<'ctx>, register: &Register) {
        self.compile_register_shift(registers, register, |builder, a, b| {
            builder.build_right_shift(a, b, true, "and")
        });
    }

    fn compile_load_in_bounds(
        &self,
        registers: &mut Registers<'ctx>,
        ptr: PointerValue<'ctx>,
        load: &Load,
        memory_size: u16,
        function: FunctionValue,
        load_branch: LoadValue<'ctx>,
    ) {
        let load_block = self.context.append_basic_block(function, "load");
        self.builder.build_unconditional_branch(load_block);
        self.builder.position_at_end(load_block);

        let offset = self.context.i16_type().const_int(load.offset as u64, false);
        let index = self
            .builder
            .build_int_add(offset, registers[load.rs as usize], "index");

        let then_block = self.context.append_basic_block(function, "load");
        let else_block = self.context.append_basic_block(function, "else");
        let end_block = self.context.append_basic_block(function, "end_load");

        let memory_size = self.context.i16_type().const_int(memory_size as u64, false);
        let in_bounds =
            self.builder
                .build_int_compare(IntPredicate::ULT, index, memory_size, "in_bounds");
        self.builder
            .build_conditional_branch(in_bounds, then_block, else_block);

        self.builder.position_at_end(then_block);
        let address = unsafe { self.builder.build_gep(ptr, &[index], "gep index") };

        let load_value = load_branch(&self.builder, self.context, address);

        self.builder.build_unconditional_branch(end_block);

        self.builder.position_at_end(else_block);
        let else_value = self.context.i16_type().const_int(0, false);
        self.builder.build_unconditional_branch(end_block);

        self.builder.position_at_end(end_block);
        let phi = self
            .builder
            .build_phi(self.context.i16_type(), "load_result");

        phi.add_incoming(&[(&load_value, then_block), (&else_value, else_block)]);

        registers[load.rd as usize] = phi.as_basic_value().into_int_value();
    }

    fn compile_store_in_bounds(
        &self,
        registers: &Registers<'ctx>,
        ptr: PointerValue<'ctx>,
        store: &Store,
        memory_size: u16,
        function: FunctionValue,
        store_branch: StoreValue<'ctx>,
    ) {
        let store_block = self.context.append_basic_block(function, "store");
        self.builder.build_unconditional_branch(store_block);
        self.builder.position_at_end(store_block);

        let offset = self
            .context
            .i16_type()
            .const_int(store.offset as u64, false);
        let index = self
            .builder
            .build_int_add(offset, registers[store.rd as usize], "index");

        let then_block = self.context.append_basic_block(function, "store");
        let end_block = self.context.append_basic_block(function, "end_store");

        let memory_size = self.context.i16_type().const_int(memory_size as u64, false);
        let in_bounds =
            self.builder
                .build_int_compare(IntPredicate::ULT, index, memory_size, "in_bounds");
        self.builder
            .build_conditional_branch(in_bounds, then_block, end_block);

        self.builder.position_at_end(then_block);
        let address = unsafe { self.builder.build_gep(ptr, &[index], "gep index") };

        store_branch(
            &self.builder,
            self.context,
            address,
            registers[store.rs as usize],
        );

        self.builder.build_unconditional_branch(end_block);

        self.builder.position_at_end(end_block);
    }

    fn compile_lb(
        &self,
        registers: &mut Registers<'ctx>,
        ptr: PointerValue<'ctx>,
        load: &Load,
        memory_size: u16,
        function: FunctionValue,
    ) {
        self.compile_load_in_bounds(
            registers,
            ptr,
            load,
            memory_size,
            function,
            |builder, context, address| {
                let load_value = builder.build_load(address, "lb");
                builder.build_int_s_extend(
                    load_value.into_int_value(),
                    context.i16_type(),
                    "extended",
                )
            },
        );
    }

    fn compile_lbu(
        &self,
        registers: &mut Registers<'ctx>,
        ptr: PointerValue<'ctx>,
        load: &Load,
        memory_size: u16,
        function: FunctionValue,
    ) {
        self.compile_load_in_bounds(
            registers,
            ptr,
            load,
            memory_size,
            function,
            |builder, context, address| {
                let load_value = builder.build_load(address, "lb");
                builder.build_int_z_extend(
                    load_value.into_int_value(),
                    context.i16_type(),
                    "extended",
                )
            },
        );
    }

    fn compile_sb(
        &self,
        registers: &Registers<'ctx>,
        ptr: PointerValue<'ctx>,
        store: &Store,
        memory_size: u16,
        function: FunctionValue,
    ) {
        self.compile_store_in_bounds(
            registers,
            ptr,
            store,
            memory_size,
            function,
            |builder, context, address, value| {
                let truncated = builder.build_int_truncate(value, context.i8_type(), "truncated");
                builder.build_store(address, truncated);
            },
        );
    }

    fn compile_lh(
        &self,
        registers: &mut Registers<'ctx>,
        ptr: PointerValue<'ctx>,
        load: &Load,
        memory_size: u16,
        function: FunctionValue,
    ) {
        let i16_type = self.context.i16_type();
        let i16_ptr_type = i16_type.ptr_type(AddressSpace::Generic);
        let ptr = self.builder.build_pointer_cast(ptr, i16_ptr_type, "lh_ptr");

        self.compile_load_in_bounds(
            registers,
            ptr,
            load,
            memory_size / 2,
            function,
            |builder, _i16_type, address| builder.build_load(address, "lh").into_int_value(),
        );
    }

    fn compile_sh(
        &self,
        registers: &Registers<'ctx>,
        ptr: PointerValue<'ctx>,
        store: &Store,
        memory_size: u16,
        function: FunctionValue,
    ) {
        let i16_type = self.context.i16_type();
        let i16_ptr_type = i16_type.ptr_type(AddressSpace::Generic);
        let i16_ptr = self.builder.build_pointer_cast(ptr, i16_ptr_type, "sh_ptr");

        self.compile_store_in_bounds(
            registers,
            i16_ptr,
            store,
            memory_size / 2,
            function,
            |builder, _context, address, value| {
                builder.build_store(address, value);
            },
        );
    }

    fn compile_beq(
        &self,
        registers: &Registers,
        branch: &Branch,
        next_block: BasicBlock,
        targets: &FxHashMap<u8, BasicBlock>,
    ) {
        let cond = self.builder.build_int_compare(
            IntPredicate::EQ,
            registers[branch.rs1 as usize],
            registers[branch.rs2 as usize],
            "beq",
        );
        if let Some(target) = targets.get(&branch.target) {
            self.builder
                .build_conditional_branch(cond, *target, next_block);
        } else {
            self.builder.build_unconditional_branch(next_block);
        }
    }

    fn max_shift(
        builder: Builder<'ctx>,
        value: IntValue<'ctx>,
        max: IntValue<'ctx>,
    ) -> IntValue<'ctx> {
        builder
            .build_select(
                builder.build_int_compare(IntPredicate::UGE, value, max, ">= max"),
                value,
                max,
                "max_shift",
            )
            .into_int_value()
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
            rs: 1,
            rd: 2,
            offset: 0,
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
        .compile_program(&instructions, 64)
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
    use crate::assemble::Assembler;
    use crate::lang::BranchTarget;
    use crate::lang::{Processor, Program};
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
        let instructions = Program::cleanup(instructions);
        let context = Context::create();
        let codegen = create_codegen(&context);
        let program = codegen
            .compile_program(&instructions, memory.len() as u16)
            .expect("Unable to JIT compile `program`");
        codegen.module.verify().unwrap();
        unsafe {
            program.call(memory.as_mut_ptr());
        }
    }

    fn run_interpreter(instructions: &[Instruction], memory: &mut [u8]) {
        let mut processor = Processor::new();
        Program::new(instructions).execute(&mut processor, memory);
    }

    #[parameterized(runner={run_llvm, run_interpreter})]
    fn test_add_immediate(runner: Runner) {
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
    fn test_beq(runner: Runner) {
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
}

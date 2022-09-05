use crate::cache::FunctionValueCache;
use crate::function::Function;
use crate::lang::{
    Branch, BranchOpcode, BranchTarget, BranchTargetOpcode, CallId, CallIdOpcode, Immediate,
    ImmediateOpcode, Instruction, Load, LoadOpcode, Register, RegisterOpcode, Store, StoreOpcode,
};
use crate::llvmasm::save_asm;
use crate::program::Program;
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::{Linkage, Module};
use inkwell::passes::{PassManager, PassManagerBuilder};
use inkwell::targets::{Target, TargetMachine, TargetTriple};
use inkwell::types::FunctionType;
use inkwell::values::{FunctionValue, IntValue, PointerValue};
use inkwell::{AddressSpace, IntPredicate, OptimizationLevel};
use rustc_hash::FxHashMap;
use std::error::Error;

pub type ProgramFunc = unsafe extern "C" fn(*mut u8) -> ();

pub struct CodeGen<'ctx> {
    context: &'ctx Context,
    pub module: Module<'ctx>,
    builder: Builder<'ctx>,
    execution_engine: ExecutionEngine<'ctx>,
}

struct Registers<'a>(Vec<PointerValue<'a>>);

impl<'a> Registers<'a> {
    fn new(codegen: &CodeGen<'a>, function: FunctionValue<'a>) -> Self {
        let registers_ptr = function.get_nth_param(1).unwrap().into_pointer_value();

        let mut registers = Vec::new();
        for i in 0..32 {
            let register_ptr = unsafe {
                codegen.builder.build_gep(
                    registers_ptr,
                    &[codegen.context.i16_type().const_int(i, false)],
                    "register",
                )
            };
            registers.push(register_ptr);
        }
        Registers(registers)
    }

    fn get(&self, index: u8) -> PointerValue<'a> {
        self.0[index as usize]
    }
}

type Build2<'ctx> =
    fn(&Builder<'ctx>, &'ctx Context, IntValue<'ctx>, IntValue<'ctx>) -> IntValue<'ctx>;
type LoadValue<'ctx> = fn(&Builder<'ctx>, &'ctx Context, PointerValue<'ctx>) -> IntValue<'ctx>;
type StoreValue<'ctx> = fn(&Builder<'ctx>, &Context, PointerValue<'ctx>, IntValue<'ctx>);

impl<'ctx> CodeGen<'ctx> {
    pub fn new(context: &'ctx Context) -> CodeGen<'ctx> {
        let module = context.create_module("program");

        let execution_engine = module
            .create_jit_execution_engine(OptimizationLevel::Default)
            .expect("Execution engine couldn't be built");

        // set data layout for performance reasons
        // https://llvm.org/docs/Frontend/PerformanceTips.html#the-basics
        let target_data = execution_engine.get_target_data();
        let data_layout = target_data.get_data_layout();
        module.set_data_layout(&data_layout);

        // set triple for performance reasons
        let triple = TargetMachine::get_default_triple();
        module.set_triple(&triple);

        CodeGen {
            context,
            module,
            builder: context.create_builder(),
            execution_engine,
        }
    }

    pub fn compile_program(
        &self,
        program_id: usize,
        functions: &FxHashMap<u16, FunctionValue>,
    ) -> Option<JitFunction<ProgramFunc>> {
        let i8_type = self.context.i8_type();
        let void_type = self.context.void_type();
        let memory_ptr_type = i8_type.ptr_type(AddressSpace::Generic);
        let fn_type = void_type.fn_type(&[memory_ptr_type.into()], false);

        let function_name = format!("func-{}", program_id);
        let function = self.module.add_function(&function_name, fn_type, None);
        let basic_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(basic_block);

        let memory_ptr = function.get_nth_param(0).unwrap().into_pointer_value();
        let registers_ptr = self.builder.build_array_alloca(
            self.context.i16_type(),
            self.context.i16_type().const_int(32, false),
            "registers",
        );
        for i in 0..32 {
            let register_ptr = unsafe {
                self.builder.build_gep(
                    registers_ptr,
                    &[self.context.i16_type().const_int(i, false)],
                    "register",
                )
            };
            self.builder
                .build_store(register_ptr, self.context.i16_type().const_int(0, false));
        }

        let inner_function = functions.get(&0).unwrap();

        self.builder.position_at_end(basic_block);
        self.builder.build_call(
            *inner_function,
            &[memory_ptr.into(), registers_ptr.into()],
            "call",
        );
        self.builder.build_return(None);

        // let pass_manager_builder = PassManagerBuilder::create();
        // pass_manager_builder.set_optimization_level(OptimizationLevel::Aggressive);
        // let fpm = PassManager::create(&self.module);
        // pass_manager_builder.populate_function_pass_manager(&fpm);

        // let mut curr_function = self.module.get_first_function();
        // while let Some(func_value) = curr_function {
        //     let fpm_result = fpm.run_on(&func_value);
        //     println!("fpm did something: {:?}", fpm_result);
        //     curr_function = func_value.get_next_function();
        // }

        // let fpm_result = fpm.run_on(self.module);

        // self.module.print_to_stderr();

        // it claims it does something but on the other hand the assembly output is
        // the same?

        // save_asm(&self.module);

        unsafe { self.execution_engine.get_function("func-0").ok() }
    }

    fn get_function_type(&self) -> FunctionType<'ctx> {
        let void_type = self.context.void_type();
        let memory_ptr_type = self.context.i8_type().ptr_type(AddressSpace::Generic);
        let registers_ptr_type = self.context.i16_type().ptr_type(AddressSpace::Generic);

        void_type.fn_type(&[memory_ptr_type.into(), registers_ptr_type.into()], false)
    }

    pub fn compile_function(
        &self,
        id: usize,
        instructions: &[Instruction],
        memory_size: u16,
        functions: &FxHashMap<u16, FunctionValue>,
    ) -> FunctionValue<'ctx> {
        let function = self.module.add_function(
            format!("inner-{}", id).as_str(),
            self.get_function_type(),
            Some(Linkage::LinkerPrivate),
        );
        let basic_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(basic_block);

        let memory_ptr = function.get_nth_param(0).unwrap().into_pointer_value();
        let registers_ptr = function.get_nth_param(1).unwrap().into_pointer_value();

        let registers = &Registers::new(self, function);

        let (blocks, targets) = self.get_blocks(function, instructions);

        let mut blocks_iter = blocks.iter();

        // this is safe as there's always at least 1 block, even if there are no instruction
        let mut next_instr_block = blocks_iter.next().unwrap().1;
        self.builder.build_unconditional_branch(next_instr_block);

        for instruction in instructions {
            let instr_block = next_instr_block;
            self.builder.position_at_end(instr_block);
            // there is safe as there's always more more block than instructions
            next_instr_block = blocks_iter.next().unwrap().1;
            let mut branched = false;
            match instruction {
                Instruction::Immediate(immediate) => {
                    use ImmediateOpcode::*;
                    match immediate.opcode {
                        Addi => self.compile_addi(registers, immediate),
                        Slti => self.compile_slti(registers, immediate),
                        Sltiu => self.compile_sltiu(registers, immediate),
                        Andi => self.compile_andi(registers, immediate),
                        Ori => self.compile_ori(registers, immediate),
                        Xori => self.compile_xori(registers, immediate),
                        Slli => self.compile_slli(registers, immediate),
                        Srli => self.compile_srli(registers, immediate),
                        Srai => self.compile_srai(registers, immediate),
                    }
                }
                Instruction::Register(register) => {
                    use RegisterOpcode::*;
                    match register.opcode {
                        Add => self.compile_add(registers, register),
                        Sub => self.compile_sub(registers, register),
                        Slt => self.compile_slt(registers, register),
                        Sltu => self.compile_sltu(registers, register),
                        And => self.compile_and(registers, register),
                        Or => self.compile_or(registers, register),
                        Xor => self.compile_xor(registers, register),
                        Sll => self.compile_sll(registers, register),
                        Srl => self.compile_srl(registers, register),
                        Sra => self.compile_sra(registers, register),
                    }
                }
                Instruction::Load(load) => {
                    use LoadOpcode::*;
                    match load.opcode {
                        Lb => {
                            self.compile_lb(registers, memory_ptr, load, memory_size, function);
                        }
                        Lbu => {
                            self.compile_lbu(registers, memory_ptr, load, memory_size, function);
                        }
                        Lh => {
                            self.compile_lh(registers, memory_ptr, load, memory_size, function);
                        }
                    }
                }
                Instruction::Store(store) => {
                    use StoreOpcode::*;
                    match store.opcode {
                        Sb => {
                            self.compile_sb(registers, memory_ptr, store, memory_size, function);
                        }
                        Sh => {
                            self.compile_sh(registers, memory_ptr, store, memory_size, function);
                        }
                    }
                }
                Instruction::Branch(branch) => {
                    use BranchOpcode::*;
                    match branch.opcode {
                        Beq => {
                            self.compile_beq(registers, branch, next_instr_block, &targets);
                            branched = true;
                        }
                        Bne => {
                            self.compile_bne(registers, branch, next_instr_block, &targets);
                            branched = true;
                        }
                        Blt => {
                            self.compile_blt(registers, branch, next_instr_block, &targets);
                            branched = true;
                        }
                        Bltu => {
                            self.compile_bltu(registers, branch, next_instr_block, &targets);
                            branched = true;
                        }
                    }
                }
                Instruction::BranchTarget(branch_target) => {
                    use BranchTargetOpcode::*;
                    match branch_target.opcode {
                        Target => {
                            // do nothing
                        }
                    }
                }
                Instruction::CallId(call_id) => {
                    use CallIdOpcode::*;
                    match call_id.opcode {
                        Call => {
                            self.compile_call(call_id, memory_ptr, registers_ptr, functions);
                        }
                    }
                }
            }
            if !branched {
                self.builder.build_unconditional_branch(next_instr_block);
            }
        }
        self.builder.position_at_end(next_instr_block);
        self.builder.build_return(None);
        function
    }

    fn get_blocks(
        &self,
        parent: FunctionValue,
        instructions: &[Instruction],
    ) -> (Vec<(usize, BasicBlock)>, FxHashMap<u8, BasicBlock>) {
        let mut blocks = Vec::new();
        let mut targets = FxHashMap::default();
        for (index, instruction) in instructions.iter().enumerate() {
            let instr_block = self
                .context
                .append_basic_block(parent, &format!("instr-{}", index));

            blocks.push((index, instr_block));

            if let Instruction::BranchTarget(BranchTarget {
                opcode: _,
                identifier,
            }) = instruction
            {
                targets.insert(*identifier, instr_block);
            }
        }
        // add one more block for the end of the program
        blocks.push((
            instructions.len(),
            self.context.append_basic_block(parent, "instr-end"),
        ));
        (blocks, targets)
    }

    fn compile_immediate(
        &self,
        registers: &Registers<'ctx>,
        immediate: &Immediate,
        f: Build2<'ctx>,
    ) {
        let i16_type = self.context.i16_type();
        let value = i16_type.const_int(immediate.value as u64, false);
        let rs = registers.get(immediate.rs);
        let rs_value = self.builder.build_load(rs, "rs_value");
        let result = f(
            &self.builder,
            self.context,
            rs_value.into_int_value(),
            value,
        );
        self.builder
            .build_store(registers.get(immediate.rd), result);
    }

    fn compile_immediate_shift(
        &self,
        registers: &Registers<'ctx>,
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
        let rs = registers.get(immediate.rs);
        let rs_value = self.builder.build_load(rs, "rs_value");
        let result = f(
            &self.builder,
            self.context,
            rs_value.into_int_value(),
            mvalue,
        );
        self.builder
            .build_store(registers.get(immediate.rd), result);
    }

    fn compile_addi(&self, registers: &Registers<'ctx>, immediate: &Immediate) {
        self.compile_immediate(registers, immediate, |builder, _context, a, b| {
            builder.build_int_add(a, b, "addi")
        });
    }

    fn compile_slti(&self, registers: &Registers<'ctx>, immediate: &Immediate) {
        self.compile_immediate(registers, immediate, |builder, context, a, b| {
            let cmp = builder.build_int_compare(IntPredicate::SLT, a, b, "slti");
            builder.build_int_z_extend(cmp, context.i16_type(), "sltz")
        });
    }

    fn compile_sltiu(&self, registers: &Registers<'ctx>, immediate: &Immediate) {
        self.compile_immediate(registers, immediate, |builder, context, a, b| {
            let cmp = builder.build_int_compare(IntPredicate::ULT, a, b, "sltiu");
            builder.build_int_z_extend(cmp, context.i16_type(), "sltz")
        });
    }

    fn compile_andi(&self, registers: &Registers<'ctx>, immediate: &Immediate) {
        self.compile_immediate(registers, immediate, |builder, _context, a, b| {
            builder.build_and(a, b, "andi")
        });
    }

    fn compile_ori(&self, registers: &Registers<'ctx>, immediate: &Immediate) {
        self.compile_immediate(registers, immediate, |builder, _context, a, b| {
            builder.build_or(a, b, "ori")
        });
    }

    fn compile_xori(&self, registers: &Registers<'ctx>, immediate: &Immediate) {
        self.compile_immediate(registers, immediate, |builder, _context, a, b| {
            builder.build_xor(a, b, "xori")
        });
    }

    fn compile_slli(&self, registers: &Registers<'ctx>, immediate: &Immediate) {
        self.compile_immediate_shift(registers, immediate, |builder, _context, a, b| {
            builder.build_left_shift(a, b, "slli")
        });
    }

    fn compile_srli(&self, registers: &Registers<'ctx>, immediate: &Immediate) {
        self.compile_immediate_shift(registers, immediate, |builder, _context, a, b| {
            builder.build_right_shift(a, b, false, "srli")
        });
    }

    fn compile_srai(&self, registers: &Registers<'ctx>, immediate: &Immediate) {
        self.compile_immediate_shift(registers, immediate, |builder, _context, a, b| {
            builder.build_right_shift(a, b, true, "srai")
        });
    }

    fn compile_register(&self, registers: &Registers<'ctx>, register: &Register, f: Build2<'ctx>) {
        let rs1 = registers.get(register.rs1);
        let rs1_value = self.builder.build_load(rs1, "rs1_value");
        let rs2 = registers.get(register.rs2);
        let rs2_value = self.builder.build_load(rs2, "rs2_value");
        let result = f(
            &self.builder,
            self.context,
            rs1_value.into_int_value(),
            rs2_value.into_int_value(),
        );
        self.builder.build_store(registers.get(register.rd), result);
    }

    fn compile_register_shift(
        &self,
        registers: &Registers<'ctx>,
        register: &Register,
        f: Build2<'ctx>,
    ) {
        let i16_type = self.context.i16_type();
        let max = i16_type.const_int(16, false);
        let zero = i16_type.const_int(0, false);
        let rs1 = registers.get(register.rs1);
        let rs1_value = self.builder.build_load(rs1, "rs1_value");
        let rs2 = registers.get(register.rs2);
        let rs2_value = self.builder.build_load(rs2, "rs2_value");
        let mvalue = self
            .builder
            .build_select(
                self.builder.build_int_compare(
                    IntPredicate::UGE,
                    rs2_value.into_int_value(),
                    max,
                    "cmp max",
                ),
                zero,
                rs2_value.into_int_value(),
                "max shift",
            )
            .into_int_value();
        let result = f(
            &self.builder,
            self.context,
            rs1_value.into_int_value(),
            mvalue,
        );
        self.builder.build_store(registers.get(register.rd), result);
    }

    fn compile_add(&self, registers: &Registers<'ctx>, register: &Register) {
        self.compile_register(registers, register, |builder, _context, a, b| {
            builder.build_int_add(a, b, "add")
        });
    }
    fn compile_sub(&self, registers: &Registers<'ctx>, register: &Register) {
        self.compile_register(registers, register, |builder, _context, a, b| {
            builder.build_int_sub(a, b, "sub")
        });
    }
    fn compile_slt(&self, registers: &Registers<'ctx>, register: &Register) {
        self.compile_register(registers, register, |builder, context, a, b| {
            let cmp = builder.build_int_compare(IntPredicate::SLT, a, b, "slt");
            builder.build_int_z_extend(cmp, context.i16_type(), "sltz")
        });
    }
    fn compile_sltu(&self, registers: &Registers<'ctx>, register: &Register) {
        self.compile_register(registers, register, |builder, context, a, b| {
            let cmp = builder.build_int_compare(IntPredicate::ULT, a, b, "sltu");
            builder.build_int_z_extend(cmp, context.i16_type(), "sltz")
        });
    }
    fn compile_and(&self, registers: &Registers<'ctx>, register: &Register) {
        self.compile_register(registers, register, |builder, _context, a, b| {
            builder.build_and(a, b, "and")
        });
    }
    fn compile_or(&self, registers: &Registers<'ctx>, register: &Register) {
        self.compile_register(registers, register, |builder, _context, a, b| {
            builder.build_or(a, b, "and")
        });
    }
    fn compile_xor(&self, registers: &Registers<'ctx>, register: &Register) {
        self.compile_register(registers, register, |builder, _context, a, b| {
            builder.build_xor(a, b, "and")
        });
    }
    fn compile_sll(&self, registers: &Registers<'ctx>, register: &Register) {
        self.compile_register_shift(registers, register, |builder, _context, a, b| {
            builder.build_left_shift(a, b, "and")
        });
    }
    fn compile_srl(&self, registers: &Registers<'ctx>, register: &Register) {
        self.compile_register_shift(registers, register, |builder, _context, a, b| {
            builder.build_right_shift(a, b, false, "and")
        });
    }
    fn compile_sra(&self, registers: &Registers<'ctx>, register: &Register) {
        self.compile_register_shift(registers, register, |builder, _context, a, b| {
            builder.build_right_shift(a, b, true, "and")
        });
    }

    fn compile_load_in_bounds(
        &self,
        registers: &Registers<'ctx>,
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
        let rs = registers.get(load.rs);
        let rs_value = self.builder.build_load(rs, "rs_value");
        let index = self
            .builder
            .build_int_add(offset, rs_value.into_int_value(), "index");

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

        self.builder.build_store(
            registers.get(load.rd),
            phi.as_basic_value().into_int_value(),
        );
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
        let rd = registers.get(store.rd);
        let rd_value = self.builder.build_load(rd, "rs_value");

        let index = self
            .builder
            .build_int_add(offset, rd_value.into_int_value(), "index");

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

        let rs = registers.get(store.rs);
        let rs_value = self.builder.build_load(rs, "rs_value");
        store_branch(
            &self.builder,
            self.context,
            address,
            rs_value.into_int_value(),
        );

        self.builder.build_unconditional_branch(end_block);

        self.builder.position_at_end(end_block);
    }

    fn compile_lb(
        &self,
        registers: &Registers<'ctx>,
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
        registers: &Registers<'ctx>,
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
        registers: &Registers<'ctx>,
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

    fn compile_compare(
        &self,
        registers: &Registers,
        branch: &Branch,
        next_block: BasicBlock,
        targets: &FxHashMap<u8, BasicBlock>,
        predicate: IntPredicate,
    ) {
        let rs1 = registers.get(branch.rs1);
        let rs1_value = self.builder.build_load(rs1, "rs1_value");
        let rs2 = registers.get(branch.rs2);
        let rs2_value = self.builder.build_load(rs2, "rs2_value");

        let cond = self.builder.build_int_compare(
            predicate,
            rs1_value.into_int_value(),
            rs2_value.into_int_value(),
            "beq",
        );
        if let Some(target) = targets.get(&branch.target) {
            self.builder
                .build_conditional_branch(cond, *target, next_block);
        } else {
            self.builder.build_unconditional_branch(next_block);
        }
    }

    fn compile_beq(
        &self,
        registers: &Registers,
        branch: &Branch,
        next_block: BasicBlock,
        targets: &FxHashMap<u8, BasicBlock>,
    ) {
        self.compile_compare(registers, branch, next_block, targets, IntPredicate::EQ);
    }

    fn compile_bne(
        &self,
        registers: &Registers,
        branch: &Branch,
        next_block: BasicBlock,
        targets: &FxHashMap<u8, BasicBlock>,
    ) {
        self.compile_compare(registers, branch, next_block, targets, IntPredicate::NE);
    }

    fn compile_blt(
        &self,
        registers: &Registers,
        branch: &Branch,
        next_block: BasicBlock,
        targets: &FxHashMap<u8, BasicBlock>,
    ) {
        self.compile_compare(registers, branch, next_block, targets, IntPredicate::SLT);
    }

    fn compile_bltu(
        &self,
        registers: &Registers,
        branch: &Branch,
        next_block: BasicBlock,
        targets: &FxHashMap<u8, BasicBlock>,
    ) {
        self.compile_compare(registers, branch, next_block, targets, IntPredicate::ULT);
    }

    fn compile_call(
        &self,
        call: &CallId,
        memory_ptr: PointerValue,
        registers_ptr: PointerValue,
        functions: &FxHashMap<u16, FunctionValue>,
    ) {
        let identifier = call.identifier;
        self.builder.build_call(
            *functions.get(&identifier).unwrap(),
            &[memory_ptr.into(), registers_ptr.into()],
            "call",
        );
    }
}

pub fn main() -> Result<(), Box<dyn Error>> {
    let mut memory = [0u8; 64];
    memory[0] = 11;

    let main_instructions = vec![
        Instruction::Load(Load {
            opcode: LoadOpcode::Lb,
            offset: 0,
            rs: 31,
            rd: 1,
        }),
        Instruction::Load(Load {
            opcode: LoadOpcode::Lb,
            offset: 1,
            rs: 31,
            rd: 2,
        }),
        Instruction::Load(Load {
            opcode: LoadOpcode::Lb,
            offset: 2,
            rs: 31,
            rd: 3,
        }),
        Instruction::Load(Load {
            opcode: LoadOpcode::Lb,
            offset: 3,
            rs: 31,
            rd: 4,
        }),
        Instruction::CallId(CallId {
            opcode: CallIdOpcode::Call,
            identifier: 1,
        }),
        Instruction::Store(Store {
            opcode: StoreOpcode::Sb,
            offset: 13,
            rs: 4,
            rd: 31,
        }),
    ];

    let sub_instructions = vec![
        Instruction::Store(Store {
            opcode: StoreOpcode::Sb,
            offset: 10,
            rs: 1,
            rd: 31,
        }),
        Instruction::CallId(CallId {
            opcode: CallIdOpcode::Call,
            identifier: 2,
        }),
        Instruction::Store(Store {
            opcode: StoreOpcode::Sb,
            offset: 12,
            rs: 3,
            rd: 31,
        }),
    ];

    let sub_sub_instructions = vec![Instruction::Store(Store {
        opcode: StoreOpcode::Sb,
        offset: 11,
        rs: 2,
        rd: 31,
    })];

    let program = Program::new(&[&main_instructions, &sub_instructions, &sub_sub_instructions]);
    let context = Context::create();
    let codegen = CodeGen::new(&context);
    let mut cache = FunctionValueCache::new();
    println!("Compiling program");
    let func = program.compile(0, &codegen, memory.len() as u16, &mut cache);
    codegen.module.verify().unwrap();

    save_asm(&codegen.module);

    println!("Running program");
    Function::run(&func, &mut memory);
    println!("Memory");
    println!("{:?}", memory);

    Ok(())
}

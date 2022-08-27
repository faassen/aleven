use crate::lang::{Branch, BranchTarget, Immediate, Instruction, Load, Register, Store};
use crate::program::Program;
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::Module;
use inkwell::passes::{PassManager, PassManagerBuilder};
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
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

type Registers<'a> = [PointerValue<'a>];

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
        CodeGen {
            context,
            module,
            builder: context.create_builder(),
            execution_engine,
        }
    }

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

        let mut registers = Vec::new();
        for _i in 0..32 {
            let alloc_a = self.builder.build_alloca(i16_type, "memory");
            self.builder
                .build_store(alloc_a, i16_type.const_int(0, false));
            registers.push(alloc_a);
        }
        let registers = registers.as_mut_slice();

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
                Instruction::Addi(immediate) => self.compile_addi(registers, immediate),
                Instruction::Slti(immediate) => self.compile_slti(registers, immediate),
                Instruction::Sltiu(immediate) => self.compile_sltiu(registers, immediate),
                Instruction::Andi(immediate) => self.compile_andi(registers, immediate),
                Instruction::Ori(immediate) => self.compile_ori(registers, immediate),
                Instruction::Xori(immediate) => self.compile_xori(registers, immediate),
                Instruction::Slli(immediate) => self.compile_slli(registers, immediate),
                Instruction::Srli(immediate) => self.compile_srli(registers, immediate),
                Instruction::Srai(immediate) => self.compile_srai(registers, immediate),
                Instruction::Add(register) => self.compile_add(registers, register),
                Instruction::Sub(register) => self.compile_sub(registers, register),
                Instruction::Slt(register) => self.compile_slt(registers, register),
                Instruction::Sltu(register) => self.compile_sltu(registers, register),
                Instruction::And(register) => self.compile_and(registers, register),
                Instruction::Or(register) => self.compile_or(registers, register),
                Instruction::Xor(register) => self.compile_xor(registers, register),
                Instruction::Sll(register) => self.compile_sll(registers, register),
                Instruction::Srl(register) => self.compile_srl(registers, register),
                Instruction::Sra(register) => self.compile_sra(registers, register),
                Instruction::Lb(load) => {
                    self.compile_lb(registers, ptr, load, memory_size, function);
                }
                Instruction::Lbu(load) => {
                    self.compile_lbu(registers, ptr, load, memory_size, function);
                }
                Instruction::Sb(store) => {
                    self.compile_sb(registers, ptr, store, memory_size, function);
                }
                Instruction::Lh(load) => {
                    self.compile_lh(registers, ptr, load, memory_size, function);
                }
                Instruction::Sh(store) => {
                    self.compile_sh(registers, ptr, store, memory_size, function);
                }
                Instruction::Beq(branch) => {
                    self.compile_beq(registers, branch, next_instr_block, &targets);
                    branched = true;
                }
                Instruction::Target(_target) => {
                    // do nothing
                }
            }
            if !branched {
                self.builder.build_unconditional_branch(next_instr_block);
            }
        }
        self.builder.position_at_end(next_instr_block);
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
        for (index, instruction) in instructions.iter().enumerate() {
            let instr_block = self
                .context
                .append_basic_block(parent, &format!("instr-{}", index));

            blocks.push((index, instr_block));

            if let Instruction::Target(target) = instruction {
                targets.insert(target.identifier, instr_block);
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
        registers: &mut Registers<'ctx>,
        immediate: &Immediate,
        f: Build2<'ctx>,
    ) {
        let i16_type = self.context.i16_type();
        let value = i16_type.const_int(immediate.value as u64, false);
        let rs = registers[immediate.rs as usize];
        let rs_value = self.builder.build_load(rs, "rs_value");
        let result = f(
            &self.builder,
            self.context,
            rs_value.into_int_value(),
            value,
        );
        self.builder
            .build_store(registers[immediate.rd as usize], result);
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
        let rs_value = self.builder.build_load(rs, "rs_value");
        let result = f(
            &self.builder,
            self.context,
            rs_value.into_int_value(),
            mvalue,
        );
        self.builder
            .build_store(registers[immediate.rd as usize], result);
    }

    fn compile_addi(&self, registers: &mut Registers<'ctx>, immediate: &Immediate) {
        self.compile_immediate(registers, immediate, |builder, _context, a, b| {
            builder.build_int_add(a, b, "addi")
        });
    }

    fn compile_slti(&self, registers: &mut Registers<'ctx>, immediate: &Immediate) {
        self.compile_immediate(registers, immediate, |builder, context, a, b| {
            let cmp = builder.build_int_compare(IntPredicate::SLT, a, b, "slti");
            builder.build_int_z_extend(cmp, context.i16_type(), "sltz")
        });
    }

    fn compile_sltiu(&self, registers: &mut Registers<'ctx>, immediate: &Immediate) {
        self.compile_immediate(registers, immediate, |builder, context, a, b| {
            let cmp = builder.build_int_compare(IntPredicate::ULT, a, b, "sltiu");
            builder.build_int_z_extend(cmp, context.i16_type(), "sltz")
        });
    }

    fn compile_andi(&self, registers: &mut Registers<'ctx>, immediate: &Immediate) {
        self.compile_immediate(registers, immediate, |builder, _context, a, b| {
            builder.build_and(a, b, "andi")
        });
    }

    fn compile_ori(&self, registers: &mut Registers<'ctx>, immediate: &Immediate) {
        self.compile_immediate(registers, immediate, |builder, _context, a, b| {
            builder.build_or(a, b, "ori")
        });
    }

    fn compile_xori(&self, registers: &mut Registers<'ctx>, immediate: &Immediate) {
        self.compile_immediate(registers, immediate, |builder, _context, a, b| {
            builder.build_xor(a, b, "xori")
        });
    }

    fn compile_slli(&self, registers: &mut Registers<'ctx>, immediate: &Immediate) {
        self.compile_immediate_shift(registers, immediate, |builder, _context, a, b| {
            builder.build_left_shift(a, b, "slli")
        });
    }

    fn compile_srli(&self, registers: &mut Registers<'ctx>, immediate: &Immediate) {
        self.compile_immediate_shift(registers, immediate, |builder, _context, a, b| {
            builder.build_right_shift(a, b, false, "srli")
        });
    }

    fn compile_srai(&self, registers: &mut Registers<'ctx>, immediate: &Immediate) {
        self.compile_immediate_shift(registers, immediate, |builder, _context, a, b| {
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
        let rs1_value = self.builder.build_load(rs1, "rs1_value");
        let rs2 = registers[register.rs2 as usize];
        let rs2_value = self.builder.build_load(rs2, "rs2_value");
        let result = f(
            &self.builder,
            self.context,
            rs1_value.into_int_value(),
            rs2_value.into_int_value(),
        );
        self.builder
            .build_store(registers[register.rd as usize], result);
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
        let rs1_value = self.builder.build_load(rs1, "rs1_value");
        let rs2 = registers[register.rs2 as usize];
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
        self.builder
            .build_store(registers[register.rd as usize], result);
    }

    fn compile_add(&self, registers: &mut Registers<'ctx>, register: &Register) {
        self.compile_register(registers, register, |builder, _context, a, b| {
            builder.build_int_add(a, b, "add")
        });
    }
    fn compile_sub(&self, registers: &mut Registers<'ctx>, register: &Register) {
        self.compile_register(registers, register, |builder, _context, a, b| {
            builder.build_int_sub(a, b, "sub")
        });
    }
    fn compile_slt(&self, registers: &mut Registers<'ctx>, register: &Register) {
        self.compile_register(registers, register, |builder, context, a, b| {
            let cmp = builder.build_int_compare(IntPredicate::SLT, a, b, "slt");
            builder.build_int_z_extend(cmp, context.i16_type(), "sltz")
        });
    }
    fn compile_sltu(&self, registers: &mut Registers<'ctx>, register: &Register) {
        self.compile_register(registers, register, |builder, context, a, b| {
            let cmp = builder.build_int_compare(IntPredicate::ULT, a, b, "sltu");
            builder.build_int_z_extend(cmp, context.i16_type(), "sltz")
        });
    }
    fn compile_and(&self, registers: &mut Registers<'ctx>, register: &Register) {
        self.compile_register(registers, register, |builder, _context, a, b| {
            builder.build_and(a, b, "and")
        });
    }
    fn compile_or(&self, registers: &mut Registers<'ctx>, register: &Register) {
        self.compile_register(registers, register, |builder, _context, a, b| {
            builder.build_or(a, b, "and")
        });
    }
    fn compile_xor(&self, registers: &mut Registers<'ctx>, register: &Register) {
        self.compile_register(registers, register, |builder, _context, a, b| {
            builder.build_xor(a, b, "and")
        });
    }
    fn compile_sll(&self, registers: &mut Registers<'ctx>, register: &Register) {
        self.compile_register_shift(registers, register, |builder, _context, a, b| {
            builder.build_left_shift(a, b, "and")
        });
    }
    fn compile_srl(&self, registers: &mut Registers<'ctx>, register: &Register) {
        self.compile_register_shift(registers, register, |builder, _context, a, b| {
            builder.build_right_shift(a, b, false, "and")
        });
    }
    fn compile_sra(&self, registers: &mut Registers<'ctx>, register: &Register) {
        self.compile_register_shift(registers, register, |builder, _context, a, b| {
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
        let rs = registers[load.rs as usize];
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
            registers[load.rd as usize],
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
        let rd = registers[store.rd as usize];
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

        let rs = registers[store.rs as usize];
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
        let rs1 = registers[branch.rs1 as usize];
        let rs1_value = self.builder.build_load(rs1, "rs1_value");
        let rs2 = registers[branch.rs2 as usize];
        let rs2_value = self.builder.build_load(rs2, "rs2_value");

        let cond = self.builder.build_int_compare(
            IntPredicate::EQ,
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
}

fn save_asm(module: &Module) {
    Target::initialize_native(&InitializationConfig::default())
        .expect("Failed to initialize native target");

    let triple = TargetMachine::get_default_triple();
    let cpu = TargetMachine::get_host_cpu_name().to_string();
    let features = TargetMachine::get_host_cpu_features().to_string();

    // let pass_manager_builder = PassManagerBuilder::create();
    // pass_manager_builder.set_optimization_level(OptimizationLevel::Aggressive);

    // let pass_manager = PassManager::create(module);

    // pass_manager_builder.populate_function_pass_manager(&pass_manager);
    // pass_manager.add_demote_memory_to_register_pass();

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
    // machine.add_analysis_passes(&pass_manager);

    machine
        .write_to_file(module, FileType::Assembly, "out.asm".as_ref())
        .unwrap();
}

pub fn main() -> Result<(), Box<dyn Error>> {
    let context = Context::create();
    let codegen = CodeGen::new(&context);

    let mut memory = [0u8; 64];
    memory[0] = 11;

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
            rs1: 24,
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
    let program = Program::new(&instructions);

    println!("Compiling program");
    let func = program.compile(&codegen, memory.len() as u16);
    save_asm(&codegen.module);

    println!("Running program");
    Program::run(func, &mut memory);
    println!("Memory");
    println!("{:?}", memory);

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::assemble::Assembler;
    use crate::lang::BranchTarget;
    use crate::lang::Processor;
    use crate::program::Program;
    use byteorder::{ByteOrder, LittleEndian};
    use parameterized::parameterized;

    type Runner = fn(&[Instruction], &mut [u8]);

    use super::*;

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
            5, 234, 234, 234, 234, 234, 234, 234, 234, 29, 21, 234, 234, 234, 234, 32, 10, 32, 6,
            10,
        ]);
        let mut memory = [0u8; 64];
        runner(&instructions, &mut memory);
    }

    #[parameterized(runner={run_llvm, run_interpreter})]
    fn test_bug8(runner: Runner) {
        let assembler = Assembler::new();
        let instructions = assembler.disassemble(&[
            0, 0, 234, 249, 185, 255, 230, 5, 191, 150, 150, 150, 150, 150, 150, 150, 150, 150,
            150, 150, 150, 150, 150, 150, 150, 150, 22, 6, 70, 0, 22,
        ]);
        let mut memory = [0u8; 64];
        runner(&instructions, &mut memory);
    }

    #[parameterized(runner={run_llvm, run_interpreter})]
    fn test_bug9(runner: Runner) {
        let assembler = Assembler::new();
        let data = [
            20, 77, 22, 146, 146, 146, 146, 146, 146, 146, 146, 146, 146, 146, 146, 146, 146, 0,
            146, 146, 146, 146, 146, 146, 146, 146, 146, 146, 146, 146, 146, 146, 146, 146, 146,
            146, 22, 22, 0, 0, 0, 0, 0, 233, 0,
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
            19, 25, 176, 25, 255, 25, 255, 255, 255, 255, 25, 25, 255, 12, 255, 25, 255, 12, 25,
            255, 255, 25, 25,
        ];
        let instructions = assembler.disassemble(&data);
        let mut memory = data.to_vec();
        runner(&instructions, &mut memory);
    }

    #[parameterized(runner={run_llvm, run_interpreter})]
    fn test_bug12(runner: Runner) {
        let assembler = Assembler::new();
        let data = [
            25, 176, 19, 24, 34, 24, 24, 24, 255, 255, 255, 255, 24, 24, 24, 24, 24, 24, 24, 24,
            24, 24, 24, 24, 24, 24, 24, 24, 24, 9, 9, 235, 24, 90, 0, 0, 0, 24, 24, 24, 24, 235,
            176, 25, 255, 25, 19, 25, 126, 25, 176, 25, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
            24, 24, 24, 24, 235, 176, 25, 255, 25, 19, 25, 126, 25, 176, 25, 255, 25, 19, 25, 25,
            25, 0, 0, 0, 0, 24, 24, 24, 24, 24, 24, 24, 25, 126,
        ];
        let instructions = assembler.disassemble(&data);
        let mut memory = data.to_vec();
        runner(&instructions, &mut memory);
    }

    #[test]
    fn test_bug13() {
        use Instruction::*;
        let data = [
            23, 81, 23, 255, 255, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 44, 23,
            23, 23, 23, 255, 255, 37, 20, 1, 0, 23, 23, 23, 23, 23, 255, 255, 255, 255, 23, 23, 23,
            23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 255, 255, 23, 23, 23, 0, 0, 23, 23,
            23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 44, 23, 23, 23, 23,
            255, 255, 161, 23, 23, 23, 23, 23, 255, 255, 0, 0, 0, 0, 0, 112, 0, 0, 255, 255, 37,
            23, 23, 23, 23, 23, 23, 23, 23, 23, 20, 1, 0, 44, 23, 23, 23, 23, 255, 255, 23, 23, 23,
            23,
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
            37, 37, 19, 16, 16, 244, 16, 16, 16, 153, 16, 16, 153, 16, 16, 1, 0, 10, 16, 244, 16,
            16, 19, 16, 16, 244, 16, 16, 16, 1, 0, 0, 0, 0, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22,
            22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 170, 22, 22, 22, 22, 22, 22, 22, 22, 22,
            22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 255, 255, 255, 255, 6, 6, 0, 0, 25, 14,
            11, 255, 6, 14, 96, 23, 49, 176, 14, 0, 6, 25, 14, 59, 11, 255, 6, 255, 22, 22, 22, 22,
            22, 22, 22, 22, 153, 10, 16, 22, 22, 234, 233, 233, 232, 22, 22, 22, 22, 22, 22, 22,
            22, 0, 16, 16, 16, 1, 244, 16, 16, 153, 193, 16, 16, 1, 0, 10, 16, 244, 16, 16, 19, 16,
            16, 244, 6, 6, 0, 0, 25, 14, 11, 255, 6, 14, 96, 23, 49, 176, 14, 0, 6, 25, 14, 59, 11,
            255, 6, 255, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22,
            22, 22, 0, 16, 16, 22, 22, 22, 255, 255, 255, 255, 22, 22, 22, 22, 22, 22, 22, 22, 22,
            22, 22, 22, 22, 16, 1, 244, 16, 16, 16, 153, 22, 22, 22, 224, 22, 22, 16, 16, 1, 22,
            22, 22, 22, 22, 22, 0, 25, 227, 1, 254, 23, 0, 0,
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
            1, 0, 0, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22,
            22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22,
            22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22,
            22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22,
            22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 128, 128,
            128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
            128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
            128, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22,
            22, 22, 22, 22, 22, 22, 22, 22, 22, 255, 255, 255, 255, 22, 22, 22, 22, 22, 22, 22, 22,
            22, 22, 49, 54, 98, 105, 116, 45, 109, 111, 100, 101, 22, 22, 22, 22, 22, 22, 22, 22,
            22, 22, 22, 22, 22, 22, 22, 0, 25, 227, 254, 23, 0,
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
}

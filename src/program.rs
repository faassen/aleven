use crate::cache::FunctionValueCache;
use crate::function::Function;
use crate::lang::{Instruction, Processor};
use crate::llvm::CodeGen;
use crate::llvm::ProgramFunc;
use inkwell::execution_engine::JitFunction;
use rustc_hash::{FxHashMap, FxHashSet};

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Program {
    functions: Vec<Function>,
}

impl Program {
    pub fn new(functions: &[(u8, &[Instruction])]) -> Program {
        Program::from_functions(
            functions
                .iter()
                .map(|(repeat, instructions)| {
                    Function::new("unknown".to_string(), instructions, *repeat)
                })
                .collect(),
        )
    }

    pub fn from_functions(functions: Vec<Function>) -> Program {
        let mut result = Program { functions };
        result.cleanup_calls();
        result
    }

    pub fn from_instructions(instructions: &[Instruction]) -> Program {
        Program::new(&[(0, instructions)])
    }

    pub fn cleanup_calls(&mut self) {
        let mut seen = FxHashSet::default();
        seen.insert(0);
        self.clean_calls_helper(0, &seen);
    }

    fn clean_calls_helper(&mut self, call_id: u16, seen: &FxHashSet<u16>) {
        let function = &self.functions[call_id as usize];
        let converted_function = function.cleanup_calls(&self.functions, seen);

        let mut seen = seen.clone();
        seen.insert(call_id);
        for sub_call_id in converted_function.get_call_id_set() {
            self.clean_calls_helper(sub_call_id, &seen);
        }
        self.functions[call_id as usize] = converted_function;
    }

    pub fn restrict_call_budget(&mut self, budget: u64) {
        let mut budget_total = budget;
        let function = &self.functions[0];
        let cost = function.get_repeat() as u64;
        let mut restricted_functions = FxHashMap::default();
        self.restrict_call_budget_helper(0, cost, &mut budget_total, &mut restricted_functions);
        for (id, function) in restricted_functions {
            self.functions[id as usize] = function;
        }
    }

    fn restrict_call_budget_helper(
        &self,
        call_id: u16,
        cost: u64,
        budget: &mut u64,
        restricted_functions: &mut FxHashMap<u16, Function>,
    ) {
        let restricted_function =
            self.functions[call_id as usize].restrict_call_budget(&mut |call_id| {
                let called = &self.functions[call_id as usize];
                let call_cost = cost * called.get_repeat() as u64;
                if call_cost > *budget {
                    return false;
                }
                *budget -= call_cost;
                self.restrict_call_budget_helper(call_id, call_cost, budget, restricted_functions);
                true
            });
        restricted_functions.insert(call_id, restricted_function);
    }

    pub fn interpret(&self, memory: &mut [u8]) {
        let mut processor = Processor::new();
        self.interpret_with_processor(memory, &mut processor);
    }

    pub fn interpret_with_processor(&self, memory: &mut [u8], processor: &mut Processor) {
        self.call(memory, processor, 0);
    }

    pub fn get_function(&self, id: u16) -> &Function {
        &self.functions[id as usize]
    }

    pub fn call(&self, memory: &mut [u8], processor: &mut Processor, id: usize) {
        self.functions[id].interpret(memory, processor, &self.functions);
    }

    pub fn compile<'ctx>(
        &'ctx self,
        program_id: usize,
        codegen: &'ctx CodeGen,
        memory_size: u16,
        cache: &mut FunctionValueCache<'ctx>,
    ) -> JitFunction<ProgramFunc> {
        let dependency_map = cache.compile(0, self, codegen, memory_size);
        codegen
            .compile_program(program_id, &dependency_map)
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assembler::parse_program;
    use crate::disassembler::disassemble;
    use crate::lang::{CallId, CallIdOpcode, Immediate, ImmediateOpcode};

    #[test]
    fn test_call_ids_no_recursion() {
        let program = Program::new(&[(
            0,
            &[Instruction::CallId(CallId {
                opcode: CallIdOpcode::Call,
                identifier: 0,
            })],
        )]);

        assert_eq!(
            program.functions,
            vec![Function::new("unknown".to_string(), &[], 0)]
        );
    }

    #[test]
    fn test_call_ids_no_indirect_recursion() {
        let program = Program::new(&[
            (
                0,
                &[Instruction::CallId(CallId {
                    opcode: CallIdOpcode::Call,
                    identifier: 1,
                })],
            ),
            (
                0,
                &[Instruction::CallId(CallId {
                    opcode: CallIdOpcode::Call,
                    identifier: 0,
                })],
            ),
        ]);

        assert_eq!(
            program.functions,
            vec![
                Function::new(
                    "unknown".to_string(),
                    &[Instruction::CallId(CallId {
                        opcode: CallIdOpcode::Call,
                        identifier: 1
                    }),],
                    0
                ),
                Function::new("unknown".to_string(), &[], 0),
            ]
        );
    }

    #[test]
    fn test_call_ids_no_indirect_multiple_calls() {
        let program = Program::new(&[
            (
                0,
                &[
                    Instruction::CallId(CallId {
                        opcode: CallIdOpcode::Call,
                        identifier: 1,
                    }),
                    Instruction::CallId(CallId {
                        opcode: CallIdOpcode::Call,
                        identifier: 2,
                    }),
                ],
            ),
            (
                0,
                &[Instruction::CallId(CallId {
                    opcode: CallIdOpcode::Call,
                    identifier: 0,
                })],
            ),
            (
                0,
                &[Instruction::Immediate(Immediate {
                    opcode: ImmediateOpcode::Addi,
                    rs: 0,
                    rd: 0,
                    value: 1,
                })],
            ),
        ]);

        assert_eq!(
            program.functions,
            vec![
                Function::new(
                    "unknown".to_string(),
                    &[
                        Instruction::CallId(CallId {
                            opcode: CallIdOpcode::Call,
                            identifier: 1
                        }),
                        Instruction::CallId(CallId {
                            opcode: CallIdOpcode::Call,
                            identifier: 2
                        }),
                    ],
                    0
                ),
                Function::new("unknown".to_string(), &[], 0),
                Function::new(
                    "unknown".to_string(),
                    &[Instruction::Immediate(Immediate {
                        opcode: ImmediateOpcode::Addi,
                        rs: 0,
                        rd: 0,
                        value: 1,
                    },),],
                    0
                )
            ]
        );
    }

    #[test]
    fn test_call_ids_no_unknown_target() {
        let program = Program::new(&[(
            0,
            &[Instruction::CallId(CallId {
                opcode: CallIdOpcode::Call,
                identifier: 100,
            })],
        )]);

        assert_eq!(
            program.functions,
            vec![Function::new("unknown".to_string(), &[], 0)]
        );
    }

    #[test]
    fn test_restrict_costs_no_repeat() {
        let program = parse_program(
            "
        func main {
            call alpha
            call beta
        }

        func alpha {
            r1 = addi r0 1
        }

        func beta {
            r1 = addi r0 1
        }
        ",
        )
        .unwrap();

        let mut budget_1 = program.clone();
        budget_1.restrict_call_budget(1);
        assert_eq!(
            disassemble(budget_1.functions[0].get_instructions()),
            "call f1\ntarget t0"
        );

        let mut budget_2 = program;
        budget_2.restrict_call_budget(2);
        assert_eq!(
            disassemble(budget_2.functions[0].get_instructions()),
            "call f1\ncall f2\ntarget t0"
        );
    }

    #[test]
    fn test_restrict_costs_with_repeat_simple() {
        let program = parse_program(
            "
        func main {
            call alpha
            call beta
        }

        repeat alpha 3 {
            r1 = addi r0 1
        }

        repeat beta 4 {
            r1 = addi r0 1
        }
        ",
        )
        .unwrap();

        let mut budget_1 = program.clone();
        budget_1.restrict_call_budget(1);
        assert_eq!(
            disassemble(budget_1.functions[0].get_instructions()),
            "target t0"
        );

        let mut budget_2 = program.clone();
        budget_2.restrict_call_budget(2);
        assert_eq!(
            disassemble(budget_2.functions[0].get_instructions()),
            "target t0"
        );

        let mut budget_3 = program.clone();
        budget_3.restrict_call_budget(3);
        assert_eq!(
            disassemble(budget_3.functions[0].get_instructions()),
            "call f1\ntarget t0"
        );

        let mut budget_4 = program.clone();
        budget_4.restrict_call_budget(4);
        assert_eq!(
            disassemble(budget_4.functions[0].get_instructions()),
            "call f1\ntarget t0"
        );

        let mut budget_5 = program;
        budget_5.restrict_call_budget(7);
        assert_eq!(
            disassemble(budget_5.functions[0].get_instructions()),
            "call f1\ncall f2\ntarget t0"
        );
    }

    #[test]
    fn test_restrict_costs_with_repeat_nested() {
        let program = parse_program(
            "
        func main {
            call alpha
            call beta
        }

        repeat alpha 3 {
            call gamma
        }

        repeat beta 4 {
            r1 = addi r0 1
        }

        repeat gamma 2 {
            r1 = addi r0 1
        }
        ",
        )
        .unwrap();

        let mut budget_1 = program.clone();
        budget_1.restrict_call_budget(1);
        assert_eq!(
            disassemble(budget_1.functions[0].get_instructions()),
            "target t0"
        );

        let mut budget_2 = program.clone();
        budget_2.restrict_call_budget(3);
        assert_eq!(
            disassemble(budget_2.functions[0].get_instructions()),
            "call f1\ntarget t0"
        );
        assert_eq!(
            disassemble(budget_2.functions[1].get_instructions()),
            "target t0"
        );

        let mut budget_3 = program;
        budget_3.restrict_call_budget(9);
        assert_eq!(
            disassemble(budget_3.functions[0].get_instructions()),
            "call f1\ntarget t0"
        );
        assert_eq!(
            disassemble(budget_3.functions[1].get_instructions()),
            "call f3\ntarget t0"
        );
    }

    #[test]
    fn test_restrict_costs_multiple_calls_same_function() {
        let program = parse_program(
            "
        func main {
            call alpha
            call alpha
        }

        repeat alpha 3 {
            call gamma
        }

        repeat gamma 2 {
            r1 = addi r0 1
        }
        ",
        )
        .unwrap();

        // we can call the first alpha, and the second alpha, but not the gamma
        // in the second alpha. in this case we cannot call gamma at all, as
        // the least budget when we last call it determines whether that call succeeds
        // 3 + 6 + 3 = 12
        let mut budget_1 = program.clone();
        budget_1.restrict_call_budget(12);
        assert_eq!(
            disassemble(budget_1.functions[0].get_instructions()),
            "call f1\ncall f1\ntarget t0"
        );
        assert_eq!(
            disassemble(budget_1.functions[1].get_instructions()),
            "target t0"
        );

        // now we give it sufficient budget to call the gamma in the second alpha
        let mut budget_2 = program;
        budget_2.restrict_call_budget(18);
        assert_eq!(
            disassemble(budget_2.functions[0].get_instructions()),
            "call f1\ncall f1\ntarget t0"
        );
        assert_eq!(
            disassemble(budget_2.functions[1].get_instructions()),
            "call f2\ntarget t0"
        );
    }
}

use crate::assembler::parse_program;
use crate::cache::FunctionValueCache;
use crate::function::Function;
use crate::lang::{Instruction, Processor};
use crate::llvm::CodeGen;
use crate::llvm::ProgramFunc;
use inkwell::execution_engine::JitFunction;
use rustc_hash::FxHashSet;

#[derive(Debug, Eq, PartialEq)]
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

    pub fn clean_calls_helper(&mut self, call_id: u16, seen: &FxHashSet<u16>) {
        let function = &self.functions[call_id as usize];
        let converted_function = function.cleanup_calls(&self.functions, seen);

        let mut seen = seen.clone();
        seen.insert(call_id);
        for sub_call_id in converted_function.get_call_id_set() {
            self.clean_calls_helper(sub_call_id, &seen);
        }
        self.functions[call_id as usize] = converted_function;
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

    pub fn get_function_cost(&self, id: u16) -> u64 {
        let function = &self.functions[id as usize];

        let call_cost = function
            .get_call_ids()
            .map(|id| self.get_function_cost(id))
            .sum::<u64>();
        if call_cost > 0 {
            call_cost * function.get_repeat() as u64
        } else {
            function.get_repeat() as u64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::{
        BranchTarget, BranchTargetOpcode, CallId, CallIdOpcode, Immediate, ImmediateOpcode,
    };

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
            vec![Function::new(
                "unknown".to_string(),
                &[Instruction::BranchTarget(BranchTarget {
                    opcode: BranchTargetOpcode::Target,
                    identifier: 0
                })],
                0
            )]
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
                    &[
                        Instruction::CallId(CallId {
                            opcode: CallIdOpcode::Call,
                            identifier: 1
                        }),
                        Instruction::BranchTarget(BranchTarget {
                            opcode: BranchTargetOpcode::Target,
                            identifier: 0
                        })
                    ],
                    0
                ),
                Function::new(
                    "unknown".to_string(),
                    &[Instruction::BranchTarget(BranchTarget {
                        opcode: BranchTargetOpcode::Target,
                        identifier: 0
                    })],
                    0
                ),
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
                        Instruction::BranchTarget(BranchTarget {
                            opcode: BranchTargetOpcode::Target,
                            identifier: 0
                        })
                    ],
                    0
                ),
                Function::new(
                    "unknown".to_string(),
                    &[Instruction::BranchTarget(BranchTarget {
                        opcode: BranchTargetOpcode::Target,
                        identifier: 0
                    })],
                    0
                ),
                Function::new(
                    "unknown".to_string(),
                    &[
                        Instruction::Immediate(Immediate {
                            opcode: ImmediateOpcode::Addi,
                            rs: 0,
                            rd: 0,
                            value: 1,
                        },),
                        Instruction::BranchTarget(BranchTarget {
                            opcode: BranchTargetOpcode::Target,
                            identifier: 0
                        })
                    ],
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
            vec![Function::new(
                "unknown".to_string(),
                &[Instruction::BranchTarget(BranchTarget {
                    opcode: BranchTargetOpcode::Target,
                    identifier: 0
                })],
                0
            )]
        );
    }

    #[test]
    fn test_function_costs_no_repeat() {
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

        assert_eq!(program.get_function_cost(0), 2);
        assert_eq!(program.get_function_cost(1), 1);
        assert_eq!(program.get_function_cost(2), 1);
    }

    #[test]
    fn test_function_costs_with_repeat_simple() {
        let program = parse_program(
            "
        func main {
            call alpha
            call beta
        }

        repeat alpha 5 {
            r1 = addi r0 1
        }

        repeat beta 6 {
            r1 = addi r0 1
        }
        ",
        )
        .unwrap();

        assert_eq!(program.get_function_cost(0), 11);
        assert_eq!(program.get_function_cost(1), 5);
        assert_eq!(program.get_function_cost(2), 6);
    }

    #[test]
    fn test_function_costs_with_repeat_nested() {
        let program = parse_program(
            "
        repeat main 10 {
            call alpha
            call beta
        }

        repeat alpha 5 {
            r1 = addi r0 1
        }

        repeat beta 6 {
            r1 = addi r0 1
        }
        ",
        )
        .unwrap();

        assert_eq!(program.get_function_cost(0), 110);
        assert_eq!(program.get_function_cost(1), 5);
        assert_eq!(program.get_function_cost(2), 6);
    }
}

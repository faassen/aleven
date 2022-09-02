use crate::cache::FunctionValueCache;
use crate::function::Function;
use crate::lang::{BranchTarget, Instruction, Processor};
use crate::llvm::CodeGen;
use crate::llvm::ProgramFunc;
use inkwell::execution_engine::JitFunction;
use rustc_hash::FxHashSet;

#[derive(Debug)]
pub struct Program {
    functions: Vec<Function>,
}

impl Program {
    pub fn new(functions: &[&[Instruction]]) -> Program {
        let mut result = Program {
            functions: functions
                .iter()
                .map(|instructions| Function::new(instructions))
                .collect(),
        };
        result.cleanup_calls();
        result
    }

    pub fn from_instructions(instructions: &[Instruction]) -> Program {
        Program::new(&[instructions])
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
        for sub_call_id in converted_function.get_call_ids() {
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
        let dependency_map = cache.compile(0, &self, codegen, memory_size);
        codegen
            .compile_program(program_id, &dependency_map)
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::{CallId, Immediate};

    #[test]
    fn test_call_ids_no_recursion() {
        let program = Program::new(&[&[Instruction::Call(CallId { identifier: 0 })]]);

        assert_eq!(
            program.functions,
            vec![Function::new(&[Instruction::Target(BranchTarget {
                identifier: 0
            })])]
        );
    }

    #[test]
    fn test_call_ids_no_indirect_recursion() {
        let program = Program::new(&[
            &[Instruction::Call(CallId { identifier: 1 })],
            &[Instruction::Call(CallId { identifier: 0 })],
        ]);

        assert_eq!(
            program.functions,
            vec![
                Function::new(&[
                    Instruction::Call(CallId { identifier: 1 }),
                    Instruction::Target(BranchTarget { identifier: 0 })
                ]),
                Function::new(&[Instruction::Target(BranchTarget { identifier: 0 })]),
            ]
        );
    }

    #[test]
    fn test_call_ids_no_indirect_multiple_calls() {
        let program = Program::new(&[
            &[
                Instruction::Call(CallId { identifier: 1 }),
                Instruction::Call(CallId { identifier: 2 }),
            ],
            &[Instruction::Call(CallId { identifier: 0 })],
            &[Instruction::Addi(Immediate {
                rs: 0,
                rd: 0,
                value: 1,
            })],
        ]);

        assert_eq!(
            program.functions,
            vec![
                Function::new(&[
                    Instruction::Call(CallId { identifier: 1 }),
                    Instruction::Call(CallId { identifier: 2 }),
                    Instruction::Target(BranchTarget { identifier: 0 })
                ]),
                Function::new(&[Instruction::Target(BranchTarget { identifier: 0 })]),
                Function::new(&[
                    Instruction::Addi(Immediate {
                        rs: 0,
                        rd: 0,
                        value: 1,
                    },),
                    Instruction::Target(BranchTarget { identifier: 0 })
                ])
            ]
        );
    }

    #[test]
    fn test_call_ids_no_unknown_target() {
        let program = Program::new(&[&[Instruction::Call(CallId { identifier: 100 })]]);

        assert_eq!(
            program.functions,
            vec![Function::new(&[Instruction::Target(BranchTarget {
                identifier: 0
            })])]
        );
    }
}

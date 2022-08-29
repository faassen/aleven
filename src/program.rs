use crate::function::Function;
use crate::lang::Instruction;

pub struct Program {
    functions: Vec<Function>,
    main_id: usize,
}

impl Program {
    pub fn from_instructions(instructions: &[Instruction]) -> Program {
        Program {
            functions: vec![Function::new(instructions)],
            main_id: 0,
        }
    }

    pub fn interpret(&self, memory: &mut [u8]) {
        self.functions[self.main_id].interpret(memory);
    }
}

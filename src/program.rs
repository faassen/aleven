use crate::function::Function;
use crate::lang::Instruction;

pub struct Program {
    functions: Vec<Function>,
    main_id: usize,
}

impl Program {
    pub fn new(functions: Vec<Function>) -> Program {
        Program {
            functions,
            main_id: 0,
        }
    }

    pub fn from_instructions(instructions: &[Instruction]) -> Program {
        Program::new(vec![Function::new(instructions)])
    }

    pub fn interpret(&self, memory: &mut [u8]) {
        self.functions[self.main_id].interpret(memory);
    }
}

use crate::function::Function;
use crate::lang::{Instruction, Processor};

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
        let mut processor = Processor::new();
        self.interpret_with_processor(memory, &mut processor);
    }

    pub fn interpret_with_processor(&self, memory: &mut [u8], processor: &mut Processor) {
        self.functions[self.main_id].interpret(memory, processor);
    }
}

// a simple stack language like interface on top of reglang
use crate::reglang::{Immediate, Instruction, Load, Program, Register, Store};
use rustc_hash::FxHashMap;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

// we have memory with i8s. We assemble instructions to stack instructions,
// then from there into memory. We can take the memory and disassemble it and
// turn it back into stack instructions. We can take a vec of stack
// instructions and create instructions out of it to display or compile

const INSTRUCTION_INDEX: isize = 128 - 32;

#[allow(non_camel_case_types)]
#[derive(EnumIter, Debug, Display, Eq, PartialEq, Copy, Clone, FromPrimitive, ToPrimitive)]
pub enum StackInstr {
    ADDI = INSTRUCTION_INDEX,
    SLTI,
    ANDI,
    ORI,
    XORI,
    SLLI,
    SRAI,
    ADD,
    SLT,
    AND,
    OR,
    XOR,
    SLL,
    SRA,
    LOAD,
    STORE,
}

pub struct StackAssembler {
    instructions: FxHashMap<String, StackInstr>,
}

pub struct Assembler;

pub struct Stack(Vec<i8>);

enum Entry {
    Instr(StackInstr),
    Value(i8),
}

impl StackAssembler {
    pub fn new() -> StackAssembler {
        let mut instructions = FxHashMap::default();
        for instruction in StackInstr::iter() {
            instructions.insert(instruction.to_string(), instruction);
        }
        StackAssembler { instructions }
    }

    pub fn assemble_words(&self, words: Vec<&str>) -> Vec<i8> {
        let mut result = Vec::new();

        for word in words {
            match self.instructions.get(word) {
                // we can find the instruction, so we encode it as i8
                Some(instruction) => {
                    result.push(instruction.encode());
                }
                // we can't find an instruction, so we store the i8 directly if
                // it's < INSTRUCTION_INDEX, if larger we ignore it
                None => {
                    // parse as number
                    match word.parse::<i8>() {
                        Ok(value) => {
                            if isize::from(value) < INSTRUCTION_INDEX {
                                result.push(value)
                            }
                        }
                        Err(_) => panic!("Not a number: {}", word),
                    }
                }
            };
        }
        result
    }

    pub fn assemble(&self, text: &str) -> Vec<i8> {
        self.assemble_words(text.split_whitespace().collect())
    }

    pub fn line_assemble(&self, text: &str) -> Vec<i8> {
        let words = text_to_words(text);
        self.assemble_words(words)
    }

    pub fn disassemble_to_entries(&self, values: &[i8]) -> Vec<Entry> {
        let mut result = Vec::new();
        for value in values {
            match *value {
                v if isize::from(v) >= INSTRUCTION_INDEX => {
                    let stack_instr = StackInstr::decode(v);
                    if let Some(stack_instr) = stack_instr {
                        result.push(Entry::Instr(stack_instr))
                    } else {
                        // ignore unknown instructions
                    }
                }
                v => result.push(Entry::Value(v)),
            }
        }
        result
    }

    pub fn disassemble_to_words(&self, values: &[i8]) -> Vec<String> {
        let mut words: Vec<String> = Vec::new();
        for entry in self.disassemble_to_entries(values) {
            match entry {
                Entry::Instr(instruction) => words.push(instruction.to_string()),
                Entry::Value(value) => words.push(value.to_string()),
            }
        }
        words
    }

    pub fn line_disassemble(&self, values: &[i8]) -> String {
        self.disassemble_to_words(values).join("\n")
    }
}

impl StackInstr {
    pub fn to_instruction(&self, stack: &mut Stack) -> Instruction {
        match self {
            StackInstr::ADDI => Instruction::AddI(stack.pop_immediate()),
            StackInstr::SLTI => Instruction::SltI(stack.pop_immediate()),
            StackInstr::ANDI => Instruction::AndI(stack.pop_immediate()),
            StackInstr::ORI => Instruction::OrI(stack.pop_immediate()),
            StackInstr::XORI => Instruction::XorI(stack.pop_immediate()),
            StackInstr::SLLI => Instruction::SllI(stack.pop_immediate()),
            StackInstr::SRAI => Instruction::SraI(stack.pop_immediate()),
            StackInstr::ADD => Instruction::Add(stack.pop_register()),
            StackInstr::SLT => Instruction::Slt(stack.pop_register()),
            StackInstr::AND => Instruction::And(stack.pop_register()),
            StackInstr::OR => Instruction::Or(stack.pop_register()),
            StackInstr::XOR => Instruction::Xor(stack.pop_register()),
            StackInstr::SLL => Instruction::Sll(stack.pop_register()),
            StackInstr::SRA => Instruction::Sra(stack.pop_register()),
            StackInstr::LOAD => Instruction::Load(stack.pop_load()),
            StackInstr::STORE => Instruction::Store(stack.pop_store()),
        }
    }

    pub fn encode(&self) -> i8 {
        num::ToPrimitive::to_i8(self).unwrap()
    }

    pub fn decode(value: i8) -> Option<StackInstr> {
        num::FromPrimitive::from_i8(value)
    }

    pub fn from_instruction(instruction: &Instruction, stack: &mut Stack) {
        match instruction {
            Instruction::AddI(immediate) => {
                stack.push_immediate(immediate);
                stack.push_instr(StackInstr::ADDI);
            }
            Instruction::SltI(immediate) => {
                stack.push_immediate(immediate);
                stack.push_instr(StackInstr::SLTI);
            }
            Instruction::AndI(immediate) => {
                stack.push_immediate(immediate);
                stack.push_instr(StackInstr::ANDI);
            }
            Instruction::OrI(immediate) => {
                stack.push_immediate(immediate);
                stack.push_instr(StackInstr::ORI);
            }
            Instruction::XorI(immediate) => {
                stack.push_immediate(immediate);
                stack.push_instr(StackInstr::XORI);
            }
            Instruction::SllI(immediate) => {
                stack.push_immediate(immediate);
                stack.push_instr(StackInstr::SLLI);
            }
            Instruction::SraI(immediate) => {
                stack.push_immediate(immediate);
                stack.push_instr(StackInstr::SRAI);
            }
            Instruction::Add(register) => {
                stack.push_register(register);
                stack.push_instr(StackInstr::ADD);
            }
            Instruction::Slt(register) => {
                stack.push_register(register);
                stack.push_instr(StackInstr::SLT);
            }
            Instruction::And(register) => {
                stack.push_register(register);
                stack.push_instr(StackInstr::AND);
            }
            Instruction::Or(register) => {
                stack.push_register(register);
                stack.push_instr(StackInstr::OR);
            }
            Instruction::Xor(register) => {
                stack.push_register(register);
                stack.push_instr(StackInstr::XOR)
            }
            Instruction::Sll(register) => {
                stack.push_register(register);
                stack.push_instr(StackInstr::SLL)
            }
            Instruction::Sra(register) => {
                stack.push_register(register);
                stack.push_instr(StackInstr::SRA)
            }
            Instruction::Load(load) => {
                stack.push_load(load);
                stack.push_instr(StackInstr::LOAD)
            }
            Instruction::Store(store) => {
                stack.push_store(store);
                stack.push_instr(StackInstr::STORE)
            }
        }
    }
}

impl Stack {
    pub fn new() -> Stack {
        Stack(Vec::new())
    }

    fn pop(&mut self) -> i8 {
        self.0.pop().unwrap_or(0)
    }

    fn pop_immediate(&mut self) -> Immediate {
        Immediate {
            rd: self.pop(),
            rs: self.pop(),
            value: self.pop(),
        }
    }

    fn pop_register(&mut self) -> Register {
        Register {
            rd: self.pop(),
            rs2: self.pop(),
            rs1: self.pop(),
        }
    }

    fn pop_load(&mut self) -> Load {
        Load {
            rd: self.pop(),
            rs: self.pop(),
            offset: self.pop(),
        }
    }

    fn pop_store(&mut self) -> Store {
        Store {
            rd: self.pop(),
            rs: self.pop(),
            offset: self.pop(),
        }
    }

    fn push_immediate(&mut self, immediate: &Immediate) {
        self.0.push(immediate.value);
        self.0.push(immediate.rs);
        self.0.push(immediate.rd);
    }

    fn push_register(&mut self, register: &Register) {
        self.0.push(register.rs1);
        self.0.push(register.rs2);
        self.0.push(register.rd);
    }

    fn push_load(&mut self, load: &Load) {
        self.0.push(load.offset);
        self.0.push(load.rs);
        self.0.push(load.rd);
    }

    fn push_store(&mut self, store: &Store) {
        self.0.push(store.offset);
        self.0.push(store.rs);
        self.0.push(store.rd);
    }

    fn push_instr(&mut self, instr: StackInstr) {
        self.push(instr.encode())
    }

    fn push(&mut self, value: i8) {
        self.0.push(value)
    }

    fn to_vec(&self) -> Vec<i8> {
        self.0.clone()
    }
}

impl Assembler {
    pub fn assemble_from_instructions(&self, instructions: &[Instruction]) -> Vec<i8> {
        let mut result = Stack::new();
        for instruction in instructions {
            StackInstr::from_instruction(instruction, &mut result);
        }
        result.to_vec()
    }

    pub fn disassemble_to_instructions(
        &self,
        stack_assembler: &StackAssembler,
        values: &[i8],
    ) -> Vec<Instruction> {
        let mut stack = Stack::new();
        let mut result = Vec::new();
        for entry in stack_assembler.disassemble_to_entries(values) {
            match entry {
                Entry::Instr(stack_instr) => {
                    result.push(stack_instr.to_instruction(&mut stack));
                }
                Entry::Value(value) => {
                    stack.0.push(value);
                }
            }
        }
        result
    }
}

pub fn text_to_words(text: &str) -> Vec<&str> {
    text.split('\n')
        .map(|line| line.split('#').collect::<Vec<&str>>()[0].trim())
        .filter(|line| !line.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regstack_assemble() {
        let assembler = StackAssembler::new();
        let words = assembler.assemble("1 2 ADD");
        assert_eq!(words, vec![1, 2, StackInstr::ADD as i8]);
    }

    #[test]
    fn test_regstack_disassemble() {
        let assembler = StackAssembler::new();
        let words = assembler.assemble("1 2 ADD");
        let text = assembler.line_disassemble(words.as_slice());
        assert_eq!(text, "1\n2\nADD");
    }
}

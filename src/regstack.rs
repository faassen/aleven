// a simple stack language like interface on top of reglang
use crate::reglang::{Immediate, Instruction, Load, Program, Register, Store};
use byteorder::{ByteOrder, LittleEndian};
use rustc_hash::FxHashMap;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

#[allow(non_camel_case_types)]
#[derive(EnumIter, Debug, Display, Eq, PartialEq, Copy, Clone, FromPrimitive, ToPrimitive)]
pub enum StackInstr {
    NOP,
    ADDI,
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

pub struct Stack(Vec<Entry>);

pub enum Entry {
    Instr(StackInstr),
    Value(i16),
}

impl StackAssembler {
    pub fn new() -> StackAssembler {
        let mut instructions = FxHashMap::default();
        for instruction in StackInstr::iter() {
            instructions.insert(instruction.to_string(), instruction);
        }
        StackAssembler { instructions }
    }

    pub fn assemble_words(&self, words: Vec<&str>) -> Vec<u8> {
        let mut result = Vec::new();

        for entry in self.assemble_entries(words) {
            match entry {
                Entry::Instr(stack_instr) => result.push(stack_instr.encode()),
                Entry::Value(value) => {
                    let mut buf: [u8; 2] = [0; 2];
                    LittleEndian::write_i16(&mut buf, value);
                    result.extend(buf);
                }
            }
        }
        result
    }

    pub fn assemble_entries(&self, words: Vec<&str>) -> Vec<Entry> {
        let mut result = Vec::new();

        for word in words {
            match self.instructions.get(word) {
                Some(instruction) => {
                    result.push(Entry::Instr(*instruction));
                }
                None => {
                    // parse as number
                    match word.parse::<i16>() {
                        Ok(value) => {
                            result.push(Entry::Value(value));
                        }
                        Err(_) => panic!("Not a number: {}", word),
                    }
                }
            };
        }
        result
    }

    pub fn assemble(&self, text: &str) -> Vec<u8> {
        self.assemble_words(text.split_whitespace().collect())
    }

    pub fn line_assemble(&self, text: &str) -> Vec<u8> {
        let words = text_to_words(text);
        self.assemble_words(words)
    }

    pub fn disassemble_to_entries(&self, values: &[u8]) -> Vec<Entry> {
        let mut result = Vec::new();
        let mut index: usize = 0;
        while index < values.len() {
            let instruction_values = &values[index..index + 7];
            let instruction = StackInstr::decode(instruction_values[6]);
            if let Some(instruction) = instruction {
                result.push(Entry::Value(to_value(instruction_values, 0, 2)));
                result.push(Entry::Value(to_value(instruction_values, 2, 4)));
                result.push(Entry::Value(to_value(instruction_values, 4, 6)));
                result.push(Entry::Instr(instruction));
            } else {
                continue;
            }
            index += 7;
        }
        result
    }

    pub fn disassemble_to_words(&self, values: &[u8]) -> Vec<String> {
        let mut words: Vec<String> = Vec::new();
        for entry in self.disassemble_to_entries(values) {
            match entry {
                Entry::Instr(instruction) => words.push(instruction.to_string()),
                Entry::Value(value) => words.push(value.to_string()),
            }
        }
        words
    }

    pub fn line_disassemble(&self, values: &[u8]) -> String {
        self.disassemble_to_words(values).join("\n")
    }
}

impl StackInstr {
    pub fn to_instruction(&self, stack: &mut Stack) -> Instruction {
        match self {
            StackInstr::NOP => Instruction::Nop(stack.pop_immediate()),
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

    pub fn encode(&self) -> u8 {
        num::ToPrimitive::to_u8(self).unwrap()
    }

    pub fn decode(value: u8) -> Option<StackInstr> {
        num::FromPrimitive::from_u8(value)
    }

    pub fn from_instruction(instruction: &Instruction, stack: &mut Stack) {
        match instruction {
            Instruction::Nop(immediate) => {
                stack.push_immediate(immediate);
                stack.push_instr(StackInstr::NOP);
            }
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

    fn pop_i16(&mut self) -> i16 {
        match self.0.pop() {
            Some(value) => match value {
                Entry::Value(value) => value,
                Entry::Instr(_) => {
                    panic!("Unexpected instruction");
                }
            },
            None => 0,
        }
    }

    fn pop_instr(&mut self) -> StackInstr {
        match self.0.pop() {
            Some(value) => match value {
                Entry::Instr(instruction) => instruction,
                Entry::Value(_) => {
                    panic!("Unexpected value");
                }
            },
            None => StackInstr::NOP,
        }
    }

    fn pop_immediate(&mut self) -> Immediate {
        Immediate {
            rd: self.pop_i16(),
            rs: self.pop_i16(),
            value: self.pop_i16(),
        }
    }

    fn pop_register(&mut self) -> Register {
        Register {
            rd: self.pop_i16(),
            rs2: self.pop_i16(),
            rs1: self.pop_i16(),
        }
    }

    fn pop_load(&mut self) -> Load {
        Load {
            rd: self.pop_i16(),
            rs: self.pop_i16(),
            offset: self.pop_i16(),
        }
    }

    fn pop_store(&mut self) -> Store {
        Store {
            rd: self.pop_i16(),
            rs: self.pop_i16(),
            offset: self.pop_i16(),
        }
    }

    fn push_immediate(&mut self, immediate: &Immediate) {
        self.push_i16(immediate.value);
        self.push_i16(immediate.rs);
        self.push_i16(immediate.rd);
    }

    fn push_register(&mut self, register: &Register) {
        self.push_i16(register.rs1);
        self.push_i16(register.rs2);
        self.push_i16(register.rd);
    }

    fn push_load(&mut self, load: &Load) {
        self.push_i16(load.offset);
        self.push_i16(load.rs);
        self.push_i16(load.rd);
    }

    fn push_store(&mut self, store: &Store) {
        self.push_i16(store.offset);
        self.push_i16(store.rs);
        self.push_i16(store.rd);
    }

    fn push_instr(&mut self, instr: StackInstr) {
        self.0.push(Entry::Instr(instr));
    }

    fn push_i16(&mut self, value: i16) {
        self.0.push(Entry::Value(value))
    }

    fn to_vec(&self) -> Vec<u8> {
        let mut result = Vec::new();
        for entry in self.0.iter() {
            match entry {
                Entry::Value(value) => {
                    let mut buf: [u8; 2] = [0; 2];
                    LittleEndian::write_i16(&mut buf, *value);
                    result.extend(buf);
                }
                Entry::Instr(instr) => result.push(instr.encode()),
            }
        }

        result
    }
}

impl Assembler {
    pub fn new() -> Assembler {
        Assembler {}
    }

    pub fn assemble_from_instructions(&self, instructions: &[Instruction]) -> Vec<u8> {
        let mut result = Stack::new();
        for instruction in instructions {
            StackInstr::from_instruction(instruction, &mut result);
        }
        result.to_vec()
    }

    pub fn disassemble_to_instructions(
        &self,
        stack_assembler: &StackAssembler,
        values: &[u8],
    ) -> Vec<Instruction> {
        let mut stack = Stack::new();
        let mut result = Vec::new();
        for entry in stack_assembler.disassemble_to_entries(values) {
            match entry {
                Entry::Instr(stack_instr) => {
                    result.push(stack_instr.to_instruction(&mut stack));
                }
                Entry::Value(value) => {
                    stack.push_i16(value);
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

fn to_value(values: &[u8], start: usize, end: usize) -> i16 {
    LittleEndian::read_i16(values[start..end].as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regstack_assemble() {
        let assembler = StackAssembler::new();
        let words = assembler.assemble("1 2 3 ADD");
        assert_eq!(words, vec![1, 0, 2, 0, 3, 0, StackInstr::ADD.encode()]);
    }

    #[test]
    fn test_regstack_disassemble() {
        let assembler = StackAssembler::new();
        let words = assembler.assemble("1 2 3 ADD");
        let text = assembler.line_disassemble(words.as_slice());
        assert_eq!(text, "1\n2\n3\nADD");
    }

    #[test]
    fn test_assemble_from_instructions() {
        let assembler = Assembler::new();
        let instructions = assembler.assemble_from_instructions(&[Instruction::AddI(Immediate {
            value: 3,
            rs: 2,
            rd: 1,
        })]);
        assert_eq!(
            instructions,
            vec![3, 0, 2, 0, 1, 0, StackInstr::ADDI.encode()]
        );
    }
}

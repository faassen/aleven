// a simple stack language like interface on top of reglang
use crate::reglang::{Instruction, Program};
use rustc_hash::FxHashMap;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

// we have memory with i8s. We assemble instructions to stack instructions,
// then from there into memory. We can take the memory and disassemble it and
// turn it back into stack instructions. We can take a vec of stack
// instructions and create instructions out of it to display or compile

const INSTRUCTION_INDEX: isize = 96;

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
                    let v = num::ToPrimitive::to_i8(instruction);
                    if let Some(value) = v {
                        result.push(value)
                    } else {
                        panic!("Should never happen {:?}", instruction);
                    }
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

    pub fn disassemble_to_words(&self, values: &[i8]) -> Vec<String> {
        let mut words: Vec<String> = Vec::new();
        for value in values {
            let decoded: Option<StackInstr> = num::FromPrimitive::from_i8(*value);
            match decoded {
                Some(instruction) => {
                    words.push(instruction.to_string());
                }
                None => {
                    words.push(format!("{}", *value));
                }
            }
        }
        words
    }

    pub fn line_disassemble(&self, values: &[i8]) -> String {
        self.disassemble_to_words(values).join("\n")
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

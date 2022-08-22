pub struct Immediate {
    value: i8,
    rs: i8,
    rd: i8,
}

pub struct Load {
    offset: i8,
    rs: i8,
    rd: i8,
}

pub struct Store {
    offset: i8,
    rs: i8,
    rd: i8,
}

pub struct Register {
    rs1: i8,
    rs2: i8,
    rd: i8,
}

pub enum Instruction {
    AddI(Immediate),
    SltI(Immediate),
    AndI(Immediate),
    OrI(Immediate),
    XorI(Immediate),
    SllI(Immediate),
    SraI(Immediate),
    Add(Register),
    Slt(Register),
    And(Register),
    Or(Register),
    Xor(Register),
    Sll(Register),
    Sra(Register),
    Load(Load),
    Store(Store),
}

pub struct Memory {
    values: [i8; 64],
}

pub struct Processor {
    registers: [i8; 32],
}

impl Memory {
    pub fn new() -> Memory {
        Memory { values: [0; 64] }
    }
}

impl Processor {
    pub fn new() -> Processor {
        Processor { registers: [0; 32] }
    }
}

impl Instruction {
    pub fn execute(&self, processor: &mut Processor, memory: &mut Memory) {
        match self {
            Instruction::AddI(immediate) => {
                let rs = immediate.rs;
                let rd = immediate.rd;
                let value = immediate.value;
                let result = processor.registers[rs as usize] + value;
                processor.registers[rd as usize] = result;
            }
            Instruction::SltI(immediate) => {
                let rs = immediate.rs;
                let rd = immediate.rd;
                let value = immediate.value;
                let result = if processor.registers[rs as usize] < value {
                    1
                } else {
                    0
                };
                processor.registers[rd as usize] = result;
            }
            Instruction::AndI(immediate) => {
                let rs = immediate.rs;
                let rd = immediate.rd;
                let value = immediate.value;
                let result = processor.registers[rs as usize] & value;
                processor.registers[rd as usize] = result;
            }
            Instruction::OrI(immediate) => {
                let rs = immediate.rs;
                let rd = immediate.rd;
                let value = immediate.value;
                let result = processor.registers[rs as usize] | value;
                processor.registers[rd as usize] = result;
            }
            Instruction::XorI(immediate) => {
                let rs = immediate.rs;
                let rd = immediate.rd;
                let value = immediate.value;
                let result = processor.registers[rs as usize] ^ value;
                processor.registers[rd as usize] = result;
            }
            Instruction::SllI(immediate) => {
                let rs = immediate.rs;
                let rd = immediate.rd;
                let value = immediate.value;
                let result = processor.registers[rs as usize] << value;
                processor.registers[rd as usize] = result;
            }
            Instruction::SraI(immediate) => {
                let rs = immediate.rs;
                let rd = immediate.rd;
                let value = immediate.value;
                let result = processor.registers[rs as usize] >> value;
                processor.registers[rd as usize] = result;
            }
            Instruction::Add(register) => {
                let rs1 = register.rs1;
                let rs2 = register.rs2;
                let rd = register.rd;
                let result = processor.registers[rs1 as usize] + processor.registers[rs2 as usize];
                processor.registers[rd as usize] = result;
            }
            Instruction::Slt(register) => {
                let rs1 = register.rs1;
                let rs2 = register.rs2;
                let rd = register.rd;
                let result =
                    if processor.registers[rs1 as usize] < processor.registers[rs2 as usize] {
                        1
                    } else {
                        0
                    };
                processor.registers[rd as usize] = result;
            }
            Instruction::And(register) => {
                let rs1 = register.rs1;
                let rs2 = register.rs2;
                let rd = register.rd;
                let result = processor.registers[rs1 as usize] & processor.registers[rs2 as usize];
                processor.registers[rd as usize] = result;
            }
            Instruction::Or(register) => {
                let rs1 = register.rs1;
                let rs2 = register.rs2;
                let rd = register.rd;
                let result = processor.registers[rs1 as usize] | processor.registers[rs2 as usize];
                processor.registers[rd as usize] = result;
            }
            Instruction::Xor(register) => {
                let rs1 = register.rs1;
                let rs2 = register.rs2;
                let rd = register.rd;
                let result = processor.registers[rs1 as usize] ^ processor.registers[rs2 as usize];
                processor.registers[rd as usize] = result;
            }
            Instruction::Sll(register) => {
                let rs1 = register.rs1;
                let rs2 = register.rs2;
                let rd = register.rd;
                let result = processor.registers[rs1 as usize] << processor.registers[rs2 as usize];
                processor.registers[rd as usize] = result;
            }
            Instruction::Sra(register) => {
                let rs1 = register.rs1;
                let rs2 = register.rs2;
                let rd = register.rd;
                let result = processor.registers[rs1 as usize] >> processor.registers[rs2 as usize];
                processor.registers[rd as usize] = result;
            }
            Instruction::Load(load) => {
                let offset = load.offset;
                let rs = load.rs;
                let rd = load.rd;
                let result = memory.values[(processor.registers[rs as usize] + offset) as usize];
                processor.registers[rd as usize] = result;
            }
            Instruction::Store(store) => {
                let offset = store.offset;
                let rs = store.rs;
                let address = store.rd;
                memory.values[(processor.registers[address as usize] + offset) as usize] =
                    processor.registers[rs as usize];
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_immediate() {
        let instruction = Instruction::AddI(Immediate {
            value: 5,
            rs: 1,
            rd: 2,
        });

        let mut processor = Processor::new();
        let mut memory = Memory::new();
        instruction.execute(&mut processor, &mut memory);

        assert_eq!(processor.registers[2], 5);
    }

    #[test]
    fn test_add_immediate_register_has_value() {
        let instruction = Instruction::AddI(Immediate {
            value: 5,
            rs: 1,
            rd: 2,
        });

        let mut processor = Processor::new();
        processor.registers[1] = 10;
        let mut memory = Memory::new();
        instruction.execute(&mut processor, &mut memory);

        assert_eq!(processor.registers[2], 15);
    }

    #[test]
    fn test_add_immediate_register_rs_is_rd() {
        let instruction = Instruction::AddI(Immediate {
            value: 5,
            rs: 1,
            rd: 1,
        });

        let mut processor = Processor::new();
        processor.registers[1] = 10;
        let mut memory = Memory::new();
        instruction.execute(&mut processor, &mut memory);

        assert_eq!(processor.registers[1], 15);
    }

    #[test]
    fn test_add_immediate_register_dec() {
        let instruction = Instruction::AddI(Immediate {
            value: -1,
            rs: 1,
            rd: 1,
        });

        let mut processor = Processor::new();
        processor.registers[1] = 10;
        let mut memory = Memory::new();
        instruction.execute(&mut processor, &mut memory);

        assert_eq!(processor.registers[1], 9);
    }

    #[test]
    fn test_slt_immediate() {
        let instruction = Instruction::SltI(Immediate {
            value: 5,
            rs: 1,
            rd: 2,
        });

        let mut processor = Processor::new();
        let mut memory = Memory::new();
        instruction.execute(&mut processor, &mut memory);

        assert_eq!(processor.registers[2], 1);
    }

    #[test]
    fn test_slt_immediate_equal() {
        let instruction = Instruction::SltI(Immediate {
            value: 5,
            rs: 1,
            rd: 2,
        });

        let mut processor = Processor::new();
        processor.registers[1] = 5;
        let mut memory = Memory::new();
        instruction.execute(&mut processor, &mut memory);

        assert_eq!(processor.registers[2], 0);
    }

    #[test]
    fn test_slt_immediate_greater() {
        let instruction = Instruction::SltI(Immediate {
            value: 5,
            rs: 1,
            rd: 2,
        });

        let mut processor = Processor::new();
        processor.registers[1] = 6;
        let mut memory = Memory::new();
        instruction.execute(&mut processor, &mut memory);

        assert_eq!(processor.registers[2], 0);
    }

    #[test]
    fn test_and_immediate() {
        let instruction = Instruction::AndI(Immediate {
            value: 0b1111110,
            rs: 1,
            rd: 2,
        });

        let mut processor = Processor::new();
        processor.registers[1] = 0b1010101;
        let mut memory = Memory::new();
        instruction.execute(&mut processor, &mut memory);

        assert_eq!(processor.registers[2], 0b1010100);
    }

    #[test]
    fn test_or_immediate() {
        let instruction = Instruction::OrI(Immediate {
            value: 0b1111110,
            rs: 1,
            rd: 2,
        });

        let mut processor = Processor::new();
        processor.registers[1] = 0b1010100;
        let mut memory = Memory::new();
        instruction.execute(&mut processor, &mut memory);

        assert_eq!(processor.registers[2], 0b1111110);
    }

    #[test]
    fn test_xor_immediate() {
        let instruction = Instruction::XorI(Immediate {
            value: 0b1111010,
            rs: 1,
            rd: 2,
        });

        let mut processor = Processor::new();
        processor.registers[1] = 0b1010100;
        let mut memory = Memory::new();
        instruction.execute(&mut processor, &mut memory);

        assert_eq!(processor.registers[2], 0b0101110);
    }

    #[test]
    fn test_sll_immediate() {
        let instruction = Instruction::SllI(Immediate {
            value: 2,
            rs: 1,
            rd: 2,
        });

        let mut processor = Processor::new();
        processor.registers[1] = 5;
        let mut memory = Memory::new();
        instruction.execute(&mut processor, &mut memory);

        assert_eq!(processor.registers[2], 20);
    }

    #[test]
    fn test_sra_immediate() {
        let instruction = Instruction::SraI(Immediate {
            value: 2,
            rs: 1,
            rd: 2,
        });

        let mut processor = Processor::new();
        processor.registers[1] = 20;
        let mut memory = Memory::new();
        instruction.execute(&mut processor, &mut memory);

        assert_eq!(processor.registers[2], 5);
    }

    #[test]
    fn test_add() {
        let instruction = Instruction::Add(Register {
            rs1: 1,
            rs2: 2,
            rd: 3,
        });

        let mut processor = Processor::new();
        processor.registers[1] = 10;
        processor.registers[2] = 20;
        let mut memory = Memory::new();
        instruction.execute(&mut processor, &mut memory);

        assert_eq!(processor.registers[3], 30);
    }

    #[test]
    fn test_sub() {
        let instruction = Instruction::Add(Register {
            rs1: 1,
            rs2: 2,
            rd: 3,
        });

        let mut processor = Processor::new();
        processor.registers[1] = 10;
        processor.registers[2] = -5;
        let mut memory = Memory::new();
        instruction.execute(&mut processor, &mut memory);

        assert_eq!(processor.registers[3], 5);
    }

    #[test]
    fn test_slt() {
        let instruction = Instruction::Slt(Register {
            rs1: 1,
            rs2: 2,
            rd: 3,
        });
        let mut processor = Processor::new();
        processor.registers[1] = 10;
        processor.registers[2] = 20;
        let mut memory = Memory::new();
        instruction.execute(&mut processor, &mut memory);

        assert_eq!(processor.registers[3], 1);
    }
    #[test]
    fn test_slt_greater() {
        let instruction = Instruction::Slt(Register {
            rs1: 1,
            rs2: 2,
            rd: 3,
        });
        let mut processor = Processor::new();
        processor.registers[1] = 21;
        processor.registers[2] = 20;
        let mut memory = Memory::new();
        instruction.execute(&mut processor, &mut memory);

        assert_eq!(processor.registers[3], 0);
    }

    #[test]
    fn test_and() {
        let instruction = Instruction::And(Register {
            rs1: 1,
            rs2: 2,
            rd: 3,
        });
        let mut processor = Processor::new();
        processor.registers[1] = 0b1010101;
        processor.registers[2] = 0b1111110;
        let mut memory = Memory::new();
        instruction.execute(&mut processor, &mut memory);

        assert_eq!(processor.registers[3], 0b1010100);
    }

    #[test]
    fn test_or() {
        let instruction = Instruction::Or(Register {
            rs1: 1,
            rs2: 2,
            rd: 3,
        });
        let mut processor = Processor::new();
        processor.registers[1] = 0b1010100;
        processor.registers[2] = 0b1111110;
        let mut memory = Memory::new();
        instruction.execute(&mut processor, &mut memory);

        assert_eq!(processor.registers[3], 0b1111110);
    }

    #[test]
    fn test_xor() {
        let instruction = Instruction::Xor(Register {
            rs1: 1,
            rs2: 2,
            rd: 3,
        });
        let mut processor = Processor::new();
        processor.registers[1] = 0b1111010;
        processor.registers[2] = 0b1010100;
        let mut memory = Memory::new();
        instruction.execute(&mut processor, &mut memory);
        assert_eq!(processor.registers[3], 0b0101110);
    }

    #[test]
    fn test_sll() {
        let instruction = Instruction::Sll(Register {
            rs1: 1,
            rs2: 2,
            rd: 3,
        });
        let mut processor = Processor::new();
        processor.registers[1] = 0b101;
        processor.registers[2] = 2;
        let mut memory = Memory::new();
        instruction.execute(&mut processor, &mut memory);
        assert_eq!(processor.registers[3], 0b10100);
    }

    #[test]
    fn test_sll_decimals() {
        let instruction = Instruction::Sll(Register {
            rs1: 1,
            rs2: 2,
            rd: 3,
        });
        let mut processor = Processor::new();
        processor.registers[1] = 5;
        processor.registers[2] = 2;
        let mut memory = Memory::new();
        instruction.execute(&mut processor, &mut memory);
        assert_eq!(processor.registers[3], 20);
    }

    #[test]
    fn test_srl() {
        let instruction = Instruction::Sra(Register {
            rs1: 1,
            rs2: 2,
            rd: 3,
        });
        let mut processor = Processor::new();
        processor.registers[1] = 0b10100;
        processor.registers[2] = 2;
        let mut memory = Memory::new();
        instruction.execute(&mut processor, &mut memory);
        assert_eq!(processor.registers[3], 0b101);
    }

    #[test]
    fn test_sra_decimals() {
        let instruction = Instruction::Sra(Register {
            rs1: 1,
            rs2: 2,
            rd: 3,
        });
        let mut processor = Processor::new();
        processor.registers[1] = 20;
        processor.registers[2] = 2;
        let mut memory = Memory::new();
        instruction.execute(&mut processor, &mut memory);
        assert_eq!(processor.registers[3], 5);
    }

    #[test]
    fn test_sra_decimals_negative() {
        let instruction = Instruction::Sra(Register {
            rs1: 1,
            rs2: 2,
            rd: 3,
        });
        let mut processor = Processor::new();
        processor.registers[1] = -20;
        processor.registers[2] = 2;
        let mut memory = Memory::new();
        instruction.execute(&mut processor, &mut memory);
        assert_eq!(processor.registers[3], -5);
    }

    #[test]
    fn test_load() {
        let instruction = Instruction::Load(Load {
            offset: 0,
            rs: 1,
            rd: 2,
        });
        let mut processor = Processor::new();
        processor.registers[1] = 10;
        let mut memory = Memory::new();
        memory.values[10] = 20;
        instruction.execute(&mut processor, &mut memory);

        assert_eq!(processor.registers[2], 20);
    }

    #[test]
    fn test_load_offset() {
        let instruction = Instruction::Load(Load {
            offset: 2,
            rs: 1,
            rd: 2,
        });
        let mut processor = Processor::new();
        processor.registers[1] = 10;
        let mut memory = Memory::new();
        memory.values[10] = 20;
        memory.values[11] = 30;
        memory.values[12] = 40;

        instruction.execute(&mut processor, &mut memory);

        assert_eq!(processor.registers[2], 40);
    }

    #[test]
    fn test_store() {
        let instruction = Instruction::Store(Store {
            offset: 0,
            rs: 1,
            rd: 2,
        });

        let mut processor = Processor::new();
        processor.registers[1] = 20;
        processor.registers[2] = 10;
        let mut memory = Memory::new();

        instruction.execute(&mut processor, &mut memory);

        assert_eq!(memory.values[10], 20);
    }

    #[test]
    fn test_store_offset() {
        let instruction = Instruction::Store(Store {
            offset: 2,
            rs: 1,
            rd: 2,
        });

        let mut processor = Processor::new();
        processor.registers[1] = 20;
        processor.registers[2] = 10;
        let mut memory = Memory::new();

        instruction.execute(&mut processor, &mut memory);

        assert_eq!(memory.values[12], 20);
    }
}

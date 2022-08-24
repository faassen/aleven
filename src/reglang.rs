use byteorder::{ByteOrder, LittleEndian};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Immediate {
    pub value: i16,
    pub rs: i16,
    pub rd: i16,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Load {
    pub offset: i16,
    pub rs: i16,
    pub rd: i16,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Store {
    pub offset: i16,
    pub rs: i16,
    pub rd: i16,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Register {
    pub rs1: i16,
    pub rs2: i16,
    pub rd: i16,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Instruction {
    Addi(Immediate),
    Slti(Immediate),
    Sltiu(Immediate),
    Andi(Immediate),
    Ori(Immediate),
    Xori(Immediate),
    Slli(Immediate),
    Srli(Immediate),
    Srai(Immediate),
    Add(Register),
    Sub(Register),
    Slt(Register),
    Sltu(Register),
    And(Register),
    Or(Register),
    Xor(Register),
    Sll(Register),
    Srl(Register),
    Sra(Register),
    Lh(Load),
    Lb(Load),
    Lbu(Load),
    Sh(Store),
    Sb(Store),
}

pub struct Processor {
    registers: [i16; 32],
}

pub struct Program {
    pub instructions: Vec<Instruction>,
}

impl Processor {
    pub fn new() -> Processor {
        Processor { registers: [0; 32] }
    }
}

impl Instruction {
    pub fn execute(&self, processor: &mut Processor, memory: &mut [u8]) {
        match self {
            Instruction::Addi(immediate) => {
                let rs = immediate.rs;
                let rd = immediate.rd;
                let value = immediate.value;
                let result = processor.registers[rs as usize] + value;
                processor.registers[rd as usize] = result;
            }
            Instruction::Slti(immediate) => {
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
            Instruction::Sltiu(immediate) => {
                let rs = immediate.rs;
                let rd = immediate.rd;
                let value = immediate.value as u16;
                let result = if (processor.registers[rs as usize] as u16) < value {
                    1
                } else {
                    0
                };
                processor.registers[rd as usize] = result;
            }
            Instruction::Andi(immediate) => {
                let rs = immediate.rs;
                let rd = immediate.rd;
                let value = immediate.value;
                let result = processor.registers[rs as usize] & value;
                processor.registers[rd as usize] = result;
            }
            Instruction::Ori(immediate) => {
                let rs = immediate.rs;
                let rd = immediate.rd;
                let value = immediate.value;
                let result = processor.registers[rs as usize] | value;
                processor.registers[rd as usize] = result;
            }
            Instruction::Xori(immediate) => {
                let rs = immediate.rs;
                let rd = immediate.rd;
                let value = immediate.value;
                let result = processor.registers[rs as usize] ^ value;
                processor.registers[rd as usize] = result;
            }
            Instruction::Slli(immediate) => {
                let rs = immediate.rs;
                let rd = immediate.rd;
                let value = immediate.value;
                let result = if value < 16 {
                    processor.registers[rs as usize] << value
                } else {
                    0
                };
                processor.registers[rd as usize] = result;
            }
            Instruction::Srli(immediate) => {
                let rs = immediate.rs;
                let rd = immediate.rd;
                let value = immediate.value;
                let result = if value < 16 {
                    (processor.registers[rs as usize] as u16) >> value
                } else {
                    0
                };
                processor.registers[rd as usize] = result as i16;
            }
            Instruction::Srai(immediate) => {
                let rs = immediate.rs;
                let rd = immediate.rd;
                let value = immediate.value;
                let result = if value < 16 {
                    processor.registers[rs as usize] >> value
                } else {
                    0
                };
                processor.registers[rd as usize] = result;
            }
            Instruction::Add(register) => {
                let rs1 = register.rs1;
                let rs2 = register.rs2;
                let rd = register.rd;
                let result = processor.registers[rs1 as usize]
                    .wrapping_add(processor.registers[rs2 as usize]);
                processor.registers[rd as usize] = result;
            }
            Instruction::Sub(register) => {
                let rs1 = register.rs1;
                let rs2 = register.rs2;
                let rd = register.rd;
                let result = processor.registers[rs1 as usize]
                    .wrapping_sub(processor.registers[rs2 as usize]);
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
            Instruction::Sltu(register) => {
                let rs1 = register.rs1;
                let rs2 = register.rs2;
                let rd = register.rd;
                let result = if (processor.registers[rs1 as usize] as u16)
                    < (processor.registers[rs2 as usize] as u16)
                {
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
                let result = if processor.registers[rs2 as usize] < 16 {
                    processor.registers[rs1 as usize] << processor.registers[rs2 as usize]
                } else {
                    0
                };
                processor.registers[rd as usize] = result;
            }
            Instruction::Srl(register) => {
                let rs1 = register.rs1;
                let rs2 = register.rs2;
                let rd = register.rd;
                let result = if processor.registers[rs2 as usize] < 16 {
                    (processor.registers[rs1 as usize] as u16) >> processor.registers[rs2 as usize]
                } else {
                    0
                };
                processor.registers[rd as usize] = result as i16;
            }
            Instruction::Sra(register) => {
                let rs1 = register.rs1;
                let rs2 = register.rs2;
                let rd = register.rd;
                let result = if processor.registers[rs2 as usize] < 16 {
                    processor.registers[rs1 as usize] >> processor.registers[rs2 as usize]
                } else {
                    0
                };
                processor.registers[rd as usize] = result;
            }
            Instruction::Lh(load) => {
                let offset = load.offset;
                let rs = load.rs;
                let rd = load.rd;
                let address = ((processor.registers[rs as usize] + offset) * 2) as usize & 0xfffe;
                let value = LittleEndian::read_i16(&memory[address..]);
                processor.registers[rd as usize] = value;
            }
            Instruction::Lb(load) => {
                let offset = load.offset;
                let rs = load.rs;
                let rd = load.rd;
                let result = memory[(processor.registers[rs as usize] + offset) as usize];
                processor.registers[rd as usize] = result as i8 as i16;
            }
            Instruction::Lbu(load) => {
                let offset = load.offset;
                let rs = load.rs;
                let rd = load.rd;
                let result = memory[(processor.registers[rs as usize] + offset) as usize];
                processor.registers[rd as usize] = result as u16 as i16;
            }
            Instruction::Sh(store) => {
                let offset = store.offset;
                let rs = store.rs;
                let rd = store.rd;
                let address = ((processor.registers[rd as usize] + offset) * 2) as usize & 0xfffe;
                LittleEndian::write_i16(&mut memory[address..], processor.registers[rs as usize]);
            }
            Instruction::Sb(store) => {
                let offset = store.offset;
                let rs = store.rs;
                let rd = store.rd;
                let address = (processor.registers[rd as usize] + offset) as usize;
                memory[address] = processor.registers[rs as usize] as u8;
            }
        }
    }
}

impl Program {
    pub fn execute(&self, processor: &mut Processor, memory: &mut [u8]) {
        for instruction in self.instructions.iter() {
            instruction.execute(processor, memory);
        }
    }
}

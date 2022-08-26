use byteorder::{ByteOrder, LittleEndian};
use rustc_hash::FxHashMap;
use strum_macros::EnumDiscriminants;

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Immediate {
    pub value: i16,
    pub rs: u8,
    pub rd: u8,
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Load {
    pub offset: u16,
    pub rs: u8,
    pub rd: u8,
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Store {
    pub offset: u16,
    pub rs: u8,
    pub rd: u8,
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Register {
    pub rs1: u8,
    pub rs2: u8,
    pub rd: u8,
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Branch {
    pub target: u8,
    pub rs1: u8,
    pub rs2: u8,
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct BranchTarget {
    pub identifier: u8,
}

#[derive(EnumDiscriminants, Debug, PartialEq, Eq, Clone)]
#[strum_discriminants(derive(FromPrimitive, ToPrimitive))]
#[strum_discriminants(name(Opcode))]
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
    Beq(Branch),
    Target(BranchTarget),
}

pub struct Processor {
    registers: [i16; 32],
    pc: usize,
    jumped: bool,
}

pub struct Program {
    instructions: Vec<Instruction>,
}

impl Processor {
    pub fn new() -> Processor {
        Processor {
            registers: [0; 32],
            pc: 0,
            jumped: false,
        }
    }
}

impl Default for Processor {
    fn default() -> Self {
        Self::new()
    }
}

impl Instruction {
    pub fn execute(
        &self,
        processor: &mut Processor,
        memory: &mut [u8],
        targets: &FxHashMap<u8, usize>,
    ) {
        match self {
            Instruction::Addi(immediate) => {
                let rs = immediate.rs;
                let rd = immediate.rd;
                let value = immediate.value;
                let result = processor.registers[rs as usize].wrapping_add(value);
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
                let value = immediate.value as u16;
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
                let value = immediate.value as u16;
                let result = if value < 16 {
                    (processor.registers[rs as usize] as u16) >> value
                } else {
                    processor.registers[rs as usize] as u16
                };
                processor.registers[rd as usize] = result as i16;
            }
            Instruction::Srai(immediate) => {
                let rs = immediate.rs;
                let rd = immediate.rd;
                let value = immediate.value as u16;
                let result = if value < 16 {
                    processor.registers[rs as usize] >> value
                } else {
                    processor.registers[rs as usize]
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
                let result = if (processor.registers[rs2 as usize] as u16) < 16 {
                    processor.registers[rs1 as usize] << processor.registers[rs2 as usize]
                } else {
                    processor.registers[rs1 as usize]
                };
                processor.registers[rd as usize] = result;
            }
            Instruction::Srl(register) => {
                let rs1 = register.rs1;
                let rs2 = register.rs2;
                let rd = register.rd;
                let result = if (processor.registers[rs2 as usize] as u16) < 16 {
                    (processor.registers[rs1 as usize] as u16) >> processor.registers[rs2 as usize]
                } else {
                    processor.registers[rs1 as usize] as u16
                };
                processor.registers[rd as usize] = result as i16;
            }
            Instruction::Sra(register) => {
                let rs1 = register.rs1;
                let rs2 = register.rs2;
                let rd = register.rd;
                let result = if (processor.registers[rs2 as usize] as u16) < 16 {
                    processor.registers[rs1 as usize] >> processor.registers[rs2 as usize]
                } else {
                    processor.registers[rs1 as usize]
                };
                processor.registers[rd as usize] = result;
            }
            Instruction::Lh(load) => {
                let offset = load.offset;
                let rs = load.rs;
                let rd = load.rd;
                let address = address_h(processor, rs, offset);
                let result = if address < (memory.len() - 1) {
                    LittleEndian::read_i16(&memory[address..])
                } else {
                    0
                };
                processor.registers[rd as usize] = result;
            }
            Instruction::Lb(load) => {
                let offset = load.offset;
                let rs = load.rs;
                let rd = load.rd;
                let address = address_b(processor, rs, offset);
                let result = if address < memory.len() {
                    memory[address]
                } else {
                    0
                };
                processor.registers[rd as usize] = result as i8 as i16;
            }
            Instruction::Lbu(load) => {
                let offset = load.offset;
                let rs = load.rs;
                let rd = load.rd;
                let address = address_b(processor, rs, offset);
                let result = if address < memory.len() {
                    memory[address]
                } else {
                    0
                };
                processor.registers[rd as usize] = result as u16 as i16;
            }
            Instruction::Sh(store) => {
                let offset = store.offset;
                let rs = store.rs;
                let rd = store.rd;
                let address = address_h(processor, rd, offset);
                if address < (memory.len() - 1) {
                    LittleEndian::write_i16(
                        &mut memory[address..],
                        processor.registers[rs as usize],
                    );
                }
            }
            Instruction::Sb(store) => {
                let offset = store.offset;
                let rs = store.rs;
                let rd = store.rd;
                let address = address_b(processor, rd, offset);
                if address < memory.len() {
                    memory[address] = processor.registers[rs as usize] as u8;
                }
            }
            Instruction::Beq(branch) => {
                let rs1 = branch.rs1;
                let rs2 = branch.rs2;
                let target = branch.target;
                let index = targets.get(&target);
                if let Some(index) = index {
                    if processor.registers[rs1 as usize] == processor.registers[rs2 as usize] {
                        processor.pc = *index;
                        processor.jumped = true;
                    }
                }
            }
            Instruction::Target(_target) => {
                // this is a no-op, as targets are only used for branches
            }
        }
    }
}

fn address_b(processor: &Processor, rs: u8, offset: u16) -> usize {
    let start_address = processor.registers[rs as usize] as u16 as usize;
    start_address.wrapping_add(offset as usize)
}

fn address_h(processor: &Processor, rs: u8, offset: u16) -> usize {
    let start_address = processor.registers[rs as usize] as u16 as usize;
    start_address.wrapping_add(offset as usize) * 2
}

impl Program {
    pub fn new(instructions: &[Instruction]) -> Program {
        Program {
            instructions: Program::cleanup(instructions),
        }
    }

    pub fn execute(&self, processor: &mut Processor, memory: &mut [u8]) {
        let targets = Program::targets(&self.instructions);
        while processor.pc < self.instructions.len() {
            let instruction = &self.instructions[processor.pc];
            instruction.execute(processor, memory, &targets);
            if processor.jumped {
                processor.jumped = false;
            } else {
                processor.pc += 1;
            }
        }
    }

    pub fn cleanup(instructions: &[Instruction]) -> Vec<Instruction> {
        // clean up program by removing branching instructions that don't have
        // targets or point to a target that's earlier
        let targets = Program::targets(instructions);
        let mut result = Vec::new();

        use Instruction::*;

        for (index, instruction) in instructions.iter().enumerate() {
            match instruction {
                Beq(branch) => {
                    let target = branch.target;
                    let target_index = targets.get(&target);
                    if let Some(target_index) = target_index {
                        if *target_index > index {
                            result.push(instruction.clone());
                        }
                    }
                }
                _ => {
                    result.push(instruction.clone());
                }
            }
        }
        result
    }

    fn targets(instructions: &[Instruction]) -> FxHashMap<u8, usize> {
        let mut targets = FxHashMap::default();
        for (index, instruction) in instructions.iter().enumerate() {
            if let Instruction::Target(target) = instruction {
                targets.insert(target.identifier, index);
            }
        }
        targets
    }
}

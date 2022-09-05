use crate::function::Function;
use byteorder::{ByteOrder, LittleEndian};
use rustc_hash::FxHashMap;
use strum::EnumCount;
use strum_macros::{Display, EnumCount as EnumCountMacro, EnumIter};

#[derive(
    Debug,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    Display,
    EnumIter,
    EnumCountMacro,
    FromPrimitive,
    ToPrimitive,
)]
pub enum ImmediateOpcode {
    Addi,
    Slti,
    Sltiu,
    Andi,
    Ori,
    Xori,
    Slli,
    Srli,
    Srai,
}

const REGISTER_OPCODE_START: usize = ImmediateOpcode::COUNT;
#[derive(
    Debug,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    Display,
    EnumIter,
    EnumCountMacro,
    FromPrimitive,
    ToPrimitive,
)]
pub enum RegisterOpcode {
    Add = REGISTER_OPCODE_START as isize,
    Sub,
    Slt,
    Sltu,
    And,
    Or,
    Xor,
    Sll,
    Srl,
    Sra,
}

const LOAD_OPCODE_START: usize = REGISTER_OPCODE_START + RegisterOpcode::COUNT;
#[derive(
    Debug,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    Display,
    EnumIter,
    EnumCountMacro,
    FromPrimitive,
    ToPrimitive,
)]
pub enum LoadOpcode {
    Lh = LOAD_OPCODE_START as isize,
    Lb,
    Lbu,
}

const STORE_OPCODE_START: usize = LOAD_OPCODE_START + LoadOpcode::COUNT;
#[derive(
    Debug,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    Display,
    EnumIter,
    EnumCountMacro,
    FromPrimitive,
    ToPrimitive,
)]
pub enum StoreOpcode {
    Sh = STORE_OPCODE_START as isize,
    Sb,
}

const BRANCH_OPCODE_START: usize = STORE_OPCODE_START + StoreOpcode::COUNT;
#[derive(
    Debug,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    Display,
    EnumIter,
    EnumCountMacro,
    FromPrimitive,
    ToPrimitive,
)]
pub enum BranchOpcode {
    Beq = BRANCH_OPCODE_START as isize,
    Bne,
    Blt,
    Bltu,
    Bge,
    Bgeu,
}

const BRANCH_TARGET_OPCODE_START: usize = BRANCH_OPCODE_START + BranchOpcode::COUNT;
#[derive(
    Debug,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    Display,
    EnumIter,
    EnumCountMacro,
    FromPrimitive,
    ToPrimitive,
)]
pub enum BranchTargetOpcode {
    Target = BRANCH_TARGET_OPCODE_START as isize,
}

const CALL_OPCODE_START: usize = BRANCH_TARGET_OPCODE_START + BranchTargetOpcode::COUNT;
#[derive(
    Debug,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    Display,
    EnumIter,
    EnumCountMacro,
    FromPrimitive,
    ToPrimitive,
)]
pub enum CallIdOpcode {
    Call = CALL_OPCODE_START as isize,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Immediate {
    pub opcode: ImmediateOpcode,
    pub value: i16,
    pub rs: u8,
    pub rd: u8,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Load {
    pub opcode: LoadOpcode,
    pub offset: u16,
    pub rs: u8,
    pub rd: u8,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Store {
    pub opcode: StoreOpcode,
    pub offset: u16,
    pub rs: u8,
    pub rd: u8,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Register {
    pub opcode: RegisterOpcode,
    pub rs1: u8,
    pub rs2: u8,
    pub rd: u8,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Branch {
    pub opcode: BranchOpcode,
    pub target: u8,
    pub rs1: u8,
    pub rs2: u8,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct BranchTarget {
    pub opcode: BranchTargetOpcode,
    pub identifier: u8,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct CallId {
    pub opcode: CallIdOpcode,
    pub identifier: u16,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Instruction {
    Immediate(Immediate),
    Load(Load),
    Store(Store),
    Register(Register),
    Branch(Branch),
    BranchTarget(BranchTarget),
    CallId(CallId),
}

#[derive(Debug)]
pub struct Processor {
    registers: [i16; 32],
    pc: usize,
    jumped: bool,
    call_stack: Vec<usize>,
}

impl Processor {
    pub fn new() -> Processor {
        Processor {
            registers: [0; 32],
            pc: 0,
            jumped: false,
            call_stack: Vec::new(),
        }
    }

    pub fn execute(
        &mut self,
        instructions: &[Instruction],
        memory: &mut [u8],
        targets: &FxHashMap<u8, usize>,
        functions: &[Function],
    ) {
        while self.pc < instructions.len() {
            let instruction = &instructions[self.pc];
            instruction.execute(self, memory, targets, functions);
            if self.jumped {
                self.jumped = false;
            } else {
                self.pc += 1;
            }
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
        functions: &[Function],
    ) {
        match self {
            Instruction::Immediate(immediate) => {
                use ImmediateOpcode::*;
                let rs = immediate.rs;
                let rd = immediate.rd;
                let value = immediate.value;
                match immediate.opcode {
                    Addi => {
                        let result = processor.registers[rs as usize].wrapping_add(value);
                        processor.registers[rd as usize] = result;
                    }
                    Slti => {
                        let result = if processor.registers[rs as usize] < value {
                            1
                        } else {
                            0
                        };
                        processor.registers[rd as usize] = result;
                    }
                    Sltiu => {
                        let result = if (processor.registers[rs as usize] as u16) < (value as u16) {
                            1
                        } else {
                            0
                        };
                        processor.registers[rd as usize] = result;
                    }
                    Andi => {
                        let result = processor.registers[rs as usize] & value;
                        processor.registers[rd as usize] = result;
                    }
                    Ori => {
                        let result = processor.registers[rs as usize] | value;
                        processor.registers[rd as usize] = result;
                    }
                    Xori => {
                        let result = processor.registers[rs as usize] ^ value;
                        processor.registers[rd as usize] = result;
                    }
                    Slli => {
                        let result = if value < 16 {
                            processor.registers[rs as usize] << (value as u16)
                        } else {
                            processor.registers[rs as usize]
                        };
                        processor.registers[rd as usize] = result;
                    }
                    Srli => {
                        let value = value as u16;
                        let result = if value < 16 {
                            (processor.registers[rs as usize] as u16) >> value
                        } else {
                            processor.registers[rs as usize] as u16
                        };
                        processor.registers[rd as usize] = result as i16;
                    }
                    Srai => {
                        let value = value as u16;
                        let result = if value < 16 {
                            processor.registers[rs as usize] >> value
                        } else {
                            processor.registers[rs as usize]
                        };
                        processor.registers[rd as usize] = result;
                    }
                }
            }
            Instruction::Register(register) => {
                use RegisterOpcode::*;
                let rs1 = register.rs1;
                let rs2 = register.rs2;
                let rd = register.rd;
                match register.opcode {
                    Add => {
                        let result = processor.registers[rs1 as usize]
                            .wrapping_add(processor.registers[rs2 as usize]);
                        processor.registers[rd as usize] = result;
                    }
                    Sub => {
                        let result = processor.registers[rs1 as usize]
                            .wrapping_sub(processor.registers[rs2 as usize]);
                        processor.registers[rd as usize] = result;
                    }
                    Slt => {
                        let result = if processor.registers[rs1 as usize]
                            < processor.registers[rs2 as usize]
                        {
                            1
                        } else {
                            0
                        };
                        processor.registers[rd as usize] = result;
                    }
                    Sltu => {
                        let result = if (processor.registers[rs1 as usize] as u16)
                            < (processor.registers[rs2 as usize] as u16)
                        {
                            1
                        } else {
                            0
                        };
                        processor.registers[rd as usize] = result;
                    }
                    And => {
                        let result =
                            processor.registers[rs1 as usize] & processor.registers[rs2 as usize];
                        processor.registers[rd as usize] = result;
                    }
                    Or => {
                        let result =
                            processor.registers[rs1 as usize] | processor.registers[rs2 as usize];
                        processor.registers[rd as usize] = result;
                    }
                    Xor => {
                        let result =
                            processor.registers[rs1 as usize] ^ processor.registers[rs2 as usize];
                        processor.registers[rd as usize] = result;
                    }
                    Sll => {
                        let result = if (processor.registers[rs2 as usize] as u16) < 16 {
                            processor.registers[rs1 as usize] << processor.registers[rs2 as usize]
                        } else {
                            processor.registers[rs1 as usize]
                        };
                        processor.registers[rd as usize] = result;
                    }
                    Srl => {
                        let result = if (processor.registers[rs2 as usize] as u16) < 16 {
                            (processor.registers[rs1 as usize] as u16)
                                >> processor.registers[rs2 as usize]
                        } else {
                            processor.registers[rs1 as usize] as u16
                        };
                        processor.registers[rd as usize] = result as i16;
                    }
                    Sra => {
                        let result = if (processor.registers[rs2 as usize] as u16) < 16 {
                            processor.registers[rs1 as usize] >> processor.registers[rs2 as usize]
                        } else {
                            processor.registers[rs1 as usize]
                        };
                        processor.registers[rd as usize] = result;
                    }
                }
            }
            Instruction::Load(load) => {
                use LoadOpcode::*;
                let offset = load.offset;
                let rs = load.rs;
                let rd = load.rd;
                match load.opcode {
                    Lh => {
                        let address = address_h(processor, rs, offset);
                        let result = if let Some(address) = address {
                            if address < (memory.len() - 1) {
                                LittleEndian::read_i16(&memory[address..])
                            } else {
                                0
                            }
                        } else {
                            0
                        };
                        processor.registers[rd as usize] = result;
                    }
                    Lb => {
                        let address = address_b(processor, rs, offset);
                        let result = if address < memory.len() {
                            memory[address]
                        } else {
                            0
                        };
                        processor.registers[rd as usize] = result as i8 as i16;
                    }
                    Lbu => {
                        let address = address_b(processor, rs, offset);
                        let result = if address < memory.len() {
                            memory[address]
                        } else {
                            0
                        };
                        processor.registers[rd as usize] = result as u16 as i16;
                    }
                }
            }
            Instruction::Store(store) => {
                use StoreOpcode::*;
                let offset = store.offset;
                let rs = store.rs;
                let rd = store.rd;
                match store.opcode {
                    Sh => {
                        let address = address_h(processor, rd, offset);
                        if let Some(address) = address {
                            if address < (memory.len() - 1) {
                                LittleEndian::write_i16(
                                    &mut memory[address..],
                                    processor.registers[rs as usize],
                                );
                            }
                        }
                    }
                    Sb => {
                        let address = address_b(processor, rd, offset);
                        if address < memory.len() {
                            memory[address] = processor.registers[rs as usize] as u8;
                        }
                    }
                }
            }
            Instruction::Branch(branch) => {
                use BranchOpcode::*;
                let rs1 = branch.rs1;
                let rs2 = branch.rs2;
                let target = branch.target;
                match branch.opcode {
                    Beq => {
                        let index = targets.get(&target);
                        if let Some(index) = index {
                            if processor.registers[rs1 as usize]
                                == processor.registers[rs2 as usize]
                            {
                                processor.pc = *index;
                                processor.jumped = true;
                            }
                        }
                    }
                    Bne => {
                        let index = targets.get(&target);
                        if let Some(index) = index {
                            if processor.registers[rs1 as usize]
                                != processor.registers[rs2 as usize]
                            {
                                processor.pc = *index;
                                processor.jumped = true;
                            }
                        }
                    }
                    Blt => {
                        let index = targets.get(&target);
                        if let Some(index) = index {
                            if processor.registers[rs1 as usize] < processor.registers[rs2 as usize]
                            {
                                processor.pc = *index;
                                processor.jumped = true;
                            }
                        }
                    }
                    Bltu => {
                        let index = targets.get(&target);
                        if let Some(index) = index {
                            if (processor.registers[rs1 as usize] as u16)
                                < (processor.registers[rs2 as usize] as u16)
                            {
                                processor.pc = *index;
                                processor.jumped = true;
                            }
                        }
                    }
                    Bge => {
                        let index = targets.get(&target);
                        if let Some(index) = index {
                            if processor.registers[rs1 as usize]
                                >= processor.registers[rs2 as usize]
                            {
                                processor.pc = *index;
                                processor.jumped = true;
                            }
                        }
                    }
                    Bgeu => {
                        let index = targets.get(&target);
                        if let Some(index) = index {
                            if (processor.registers[rs1 as usize] as u16)
                                >= (processor.registers[rs2 as usize] as u16)
                            {
                                processor.pc = *index;
                                processor.jumped = true;
                            }
                        }
                    }
                }
            }
            Instruction::BranchTarget(target) => {
                use BranchTargetOpcode::*;
                match target.opcode {
                    Target => {
                        // this is a no-op, as targets are only used for branches
                    }
                }
            }
            Instruction::CallId(call_id) => {
                use CallIdOpcode::*;
                match call_id.opcode {
                    Call => {
                        let identifier = call_id.identifier as usize;
                        let function = &functions[identifier];
                        processor.call_stack.push(processor.pc);
                        processor.pc = 0;
                        function.interpret(memory, processor, functions);
                        processor.pc = processor.call_stack.pop().unwrap();
                    }
                }
            }
        }
    }

    pub fn opcode_str(&self) -> String {
        use Instruction::*;
        match self {
            Register(register) => register.opcode.to_string(),
            Immediate(immediate) => immediate.opcode.to_string(),
            Load(load) => load.opcode.to_string(),
            Store(store) => store.opcode.to_string(),
            Branch(branch) => branch.opcode.to_string(),
            BranchTarget(target) => target.opcode.to_string(),
            CallId(call_id) => call_id.opcode.to_string(),
        }
    }
}

fn address_b(processor: &Processor, rs: u8, offset: u16) -> usize {
    let start_address = processor.registers[rs as usize] as u16;
    start_address.wrapping_add(offset) as usize
}

fn address_h(processor: &Processor, rs: u8, offset: u16) -> Option<usize> {
    let start_address = processor.registers[rs as usize] as u16;
    start_address
        .wrapping_add(offset)
        .checked_mul(2)
        .map(|address| address as usize)
}

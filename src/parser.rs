use crate::assemble::OpcodeType;
use crate::lang::Opcode;
use crate::lang::{Instruction, Register};
use nom::bytes::complete::{tag, take, take_while, take_while_m_n};
use nom::character::complete::{i16, space0, space1, u16, u8};
use nom::combinator::{flat_map, map, map_opt, map_res};
use nom::error::ParseError;
use nom::sequence::{delimited, preceded, separated_pair, terminated, tuple};
use nom::IResult;
use nom::Parser;
use rustc_hash::FxHashMap;
use strum::IntoEnumIterator;

#[derive(Debug)]
struct OpcodeError {}

struct Opcodes {
    map: FxHashMap<String, Opcode>,
}

impl Opcodes {
    fn new() -> Opcodes {
        let mut result = FxHashMap::default();
        for opcode in Opcode::iter() {
            result.insert(opcode.to_string().to_lowercase(), opcode);
        }
        Opcodes { map: result }
    }

    fn get(&self, name: &str) -> Option<&Opcode> {
        self.map.get(name)
    }
}

fn register(input: &str) -> IResult<&str, u8> {
    preceded(tag("r"), u8)(input)
}

fn opcode<'a>(
    opcodes: &'a Opcodes,
    opcode_type: OpcodeType,
) -> impl Fn(&'a str) -> IResult<&'a str, &Opcode> {
    move |input: &'a str| {
        map_opt(take_while(|c: char| c.is_alphanumeric()), |s| {
            let opcode = opcodes.get(s);
            if let Some(opcode) = opcode {
                if opcode.opcode_type() == opcode_type {
                    Some(opcode)
                } else {
                    None
                }
            } else {
                None
            }
        })(input)
    }
}
fn instruction_immediate<'a>(
    input: &'a str,
    opcodes: &'a Opcodes,
) -> IResult<&'a str, (u8, (&'a Opcode, u8, i16))> {
    separated_pair(
        register,
        delimited(space0, tag("="), space0),
        tuple((
            opcode(opcodes, OpcodeType::Immediate),
            preceded(space1, register),
            preceded(space1, i16),
        )),
    )(input)
}

fn instruction_register<'a>(
    input: &'a str,
    opcodes: &'a Opcodes,
) -> IResult<&'a str, (u8, (&'a Opcode, u8, u8))> {
    separated_pair(
        register,
        delimited(space0, tag("="), space0),
        tuple((
            opcode(opcodes, OpcodeType::Register),
            preceded(space1, register),
            preceded(space1, register),
        )),
    )(input)
}

fn instruction_load<'a>(
    input: &'a str,
    opcodes: &'a Opcodes,
) -> IResult<&'a str, (u8, (&'a Opcode, u8, u16))> {
    separated_pair(
        register,
        delimited(space0, tag("="), space0),
        tuple((
            opcode(opcodes, OpcodeType::Load),
            preceded(space1, register),
            preceded(space1, u16),
        )),
    )(input)
}

fn instruction_store<'a>(
    input: &'a str,
    opcodes: &'a Opcodes,
) -> IResult<&'a str, ((&'a Opcode, u8, u16), u8)> {
    separated_pair(
        tuple((
            opcode(opcodes, OpcodeType::Store),
            preceded(space1, register),
            preceded(space1, u16),
        )),
        delimited(space0, tag("="), space0),
        register,
    )(input)
}
// r1 = addi r0 15
// r2 = add r3 r4
// r1 = lb r0 10
// sb r0 10 = r1
// beq r1 r2 2
// target 2
// call 14

// register one is relatively easy to parse

// R = opcode R R

#[cfg(test)]
mod tests {
    use super::*;
    use nom_test_helpers::assert_error;

    #[test]
    fn test_register() {
        assert_eq!(register("r1"), Ok(("", 1)));
        assert_eq!(register("r10"), Ok(("", 10)));
        assert_eq!(register("r10 "), Ok((" ", 10)));
    }

    #[test]
    fn test_opcodes() {
        let opcodes = Opcodes::new();
        assert_eq!(opcodes.get("addi"), Some(&Opcode::Addi));
    }

    #[test]
    fn test_opcode() {
        let opcodes = Opcodes::new();
        assert_eq!(
            opcode(&opcodes, OpcodeType::Immediate)("addi"),
            Ok(("", &Opcode::Addi))
        );

        assert_error!(opcode(&opcodes, OpcodeType::Register)("addi"));
    }

    #[test]
    fn test_instruction_register() {
        let opcodes = Opcodes::new();
        assert_eq!(
            instruction_register("r1 = add r2 r3", &opcodes),
            Ok(("", (1, (&Opcode::Add, 2, 3))))
        );
    }

    #[test]
    fn test_instruction_immediate() {
        let opcodes = Opcodes::new();
        assert_eq!(
            instruction_immediate("r1 = addi r2 5", &opcodes),
            Ok(("", (1, (&Opcode::Addi, 2, 5))))
        );
        assert_eq!(
            instruction_immediate("r1 = addi r2 -5", &opcodes),
            Ok(("", (1, (&Opcode::Addi, 2, -5))))
        );
    }

    #[test]
    fn test_instruction_load() {
        let opcodes = Opcodes::new();
        assert_eq!(
            instruction_load("r1 = lb r2 5", &opcodes),
            Ok(("", (1, (&Opcode::Lb, 2, 5))))
        );
    }

    #[test]
    fn test_instruction_store() {
        let opcodes = Opcodes::new();
        assert_eq!(
            instruction_store("sb r2 5 = r1", &opcodes),
            Ok(("", ((&Opcode::Sb, 2, 5), 1)))
        );
    }
}

use crate::assemble::OpcodeType;
use crate::lang::Opcode;
use crate::lang::{Branch, BranchTarget, CallId, Immediate, Instruction, Load, Register, Store};
use nom::bytes::complete::{tag, take, take_while, take_while_m_n};
use nom::character::complete::{i16, space0, space1, u16, u8};
use nom::combinator::{flat_map, map, map_opt, map_res};
use nom::error::ParseError;
use nom::sequence::{delimited, preceded, separated_pair, terminated, tuple};
use nom::IResult;
use nom::Parser;
use rustc_hash::FxHashMap;
use std::convert::{From, Into};
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
) -> impl Fn(&'a str) -> IResult<&'a str, Opcode> {
    move |input: &'a str| {
        map_opt(take_while(|c: char| c.is_alphanumeric()), |s| {
            let opcode = opcodes.get(s);
            if let Some(opcode) = opcode {
                let t: OpcodeType = (*opcode).into();
                if t == opcode_type {
                    Some(*opcode)
                } else {
                    None
                }
            } else {
                None
            }
        })(input)
    }
}
fn opcode_immediate<'a>(
    input: &'a str,
    opcodes: &'a Opcodes,
) -> IResult<&'a str, (Opcode, Immediate)> {
    let (input, (rd, (opcode, rs, value))) = separated_pair(
        register,
        delimited(space0, tag("="), space0),
        tuple((
            opcode(opcodes, OpcodeType::Immediate),
            preceded(space1, register),
            preceded(space1, i16),
        )),
    )(input)?;
    Ok((input, (opcode, Immediate { rd, rs, value })))
}

fn opcode_register<'a>(
    input: &'a str,
    opcodes: &'a Opcodes,
) -> IResult<&'a str, (Opcode, Register)> {
    let (input, (rd, (opcode, rs1, rs2))) = separated_pair(
        register,
        delimited(space0, tag("="), space0),
        tuple((
            opcode(opcodes, OpcodeType::Register),
            preceded(space1, register),
            preceded(space1, register),
        )),
    )(input)?;
    Ok((input, (opcode, Register { rd, rs1, rs2 })))
}

fn opcode_load<'a>(input: &'a str, opcodes: &'a Opcodes) -> IResult<&'a str, (Opcode, Load)> {
    let (input, (rd, (opcode, rs, offset))) = separated_pair(
        register,
        delimited(space0, tag("="), space0),
        tuple((
            opcode(opcodes, OpcodeType::Load),
            preceded(space1, register),
            preceded(space1, u16),
        )),
    )(input)?;
    Ok((input, (opcode, Load { rd, rs, offset })))
}

fn opcode_store<'a>(input: &'a str, opcodes: &'a Opcodes) -> IResult<&'a str, (Opcode, Store)> {
    let (input, ((opcode, rd, offset), rs)) = separated_pair(
        tuple((
            opcode(opcodes, OpcodeType::Store),
            preceded(space1, register),
            preceded(space1, u16),
        )),
        delimited(space0, tag("="), space0),
        register,
    )(input)?;
    Ok((input, (opcode, Store { rd, rs, offset })))
}

fn opcode_branch<'a>(input: &'a str, opcodes: &'a Opcodes) -> IResult<&'a str, (Opcode, Branch)> {
    let (input, (opcode, rs1, rs2, target)) = tuple((
        opcode(opcodes, OpcodeType::Branch),
        preceded(space1, register),
        preceded(space1, register),
        preceded(space1, u8),
    ))(input)?;
    Ok((input, (opcode, Branch { rs1, rs2, target })))
}

fn opcode_target<'a>(
    input: &'a str,
    opcodes: &'a Opcodes,
) -> IResult<&'a str, (Opcode, BranchTarget)> {
    let (input, (opcode, identifier)) = tuple((
        opcode(opcodes, OpcodeType::BranchTarget),
        preceded(space1, u8),
    ))(input)?;
    Ok((input, (opcode, BranchTarget { identifier })))
}

fn opcode_call<'a>(input: &'a str, opcodes: &'a Opcodes) -> IResult<&'a str, (Opcode, CallId)> {
    let (input, (opcode, identifier)) =
        tuple((opcode(opcodes, OpcodeType::Call), preceded(space1, u16)))(input)?;
    Ok((input, (opcode, CallId { identifier })))
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
            Ok(("", Opcode::Addi))
        );

        assert_error!(opcode(&opcodes, OpcodeType::Register)("addi"));
    }

    #[test]
    fn test_opcode_register() {
        let opcodes = Opcodes::new();
        assert_eq!(
            opcode_register("r1 = add r2 r3", &opcodes),
            Ok((
                "",
                (
                    Opcode::Add,
                    Register {
                        rd: 1,
                        rs1: 2,
                        rs2: 3
                    }
                )
            ))
        );
    }

    #[test]
    fn test_opcode_immediate() {
        let opcodes = Opcodes::new();
        assert_eq!(
            opcode_immediate("r1 = addi r2 5", &opcodes),
            Ok((
                "",
                (
                    Opcode::Addi,
                    Immediate {
                        rd: 1,
                        rs: 2,
                        value: 5
                    }
                )
            ))
        );
        assert_eq!(
            opcode_immediate("r1 = addi r2 -5", &opcodes),
            Ok((
                "",
                (
                    Opcode::Addi,
                    Immediate {
                        rd: 1,
                        rs: 2,
                        value: -5
                    }
                )
            ))
        );
    }

    #[test]
    fn test_opcode_load() {
        let opcodes = Opcodes::new();
        assert_eq!(
            opcode_load("r1 = lb r2 5", &opcodes),
            Ok((
                "",
                (
                    Opcode::Lb,
                    Load {
                        rd: 1,
                        rs: 2,
                        offset: 5
                    }
                )
            ))
        );
    }

    #[test]
    fn test_opcode_store() {
        let opcodes = Opcodes::new();
        assert_eq!(
            opcode_store("sb r2 5 = r1", &opcodes),
            Ok((
                "",
                (
                    Opcode::Sb,
                    Store {
                        rd: 2,
                        rs: 1,
                        offset: 5
                    }
                )
            ))
        );
    }

    #[test]
    fn test_opcode_branch() {
        let opcodes = Opcodes::new();
        assert_eq!(
            opcode_branch("beq r1 r2 10", &opcodes),
            Ok((
                "",
                (
                    Opcode::Beq,
                    Branch {
                        rs1: 1,
                        rs2: 2,
                        target: 10
                    }
                )
            ))
        )
    }

    #[test]
    fn test_opcode_target() {
        let opcodes = Opcodes::new();
        assert_eq!(
            opcode_target("target 10", &opcodes),
            Ok(("", (Opcode::Target, BranchTarget { identifier: 10 })))
        )
    }

    #[test]
    fn test_call() {
        let opcodes = Opcodes::new();
        assert_eq!(
            opcode_call("call 10", &opcodes),
            Ok(("", (Opcode::Call, CallId { identifier: 10 })))
        )
    }
}

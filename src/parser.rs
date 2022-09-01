use crate::lang::Opcode;
use crate::lang::{Branch, BranchTarget, CallId, Immediate, Instruction, Load, Register, Store};
use crate::opcodetype::OpcodeType;
use nom::bytes::complete::{is_not, tag, take_until, take_while};
use nom::character::complete::char;
use nom::character::complete::{i16, line_ending, newline, space0, space1, u16, u8};
use nom::combinator::{eof, map_opt, opt, value};
use nom::multi::many_till;
use nom::sequence::{delimited, pair, preceded, separated_pair, terminated, tuple};
use nom::IResult;
use rustc_hash::FxHashMap;
use std::convert::Into;
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
fn instruction_immediate<'a>(
    opcodes: &'a Opcodes,
) -> impl Fn(&'a str) -> IResult<&'a str, Instruction> {
    move |input: &'a str| {
        let (input, (rd, (opcode, rs, value))) = separated_pair(
            register,
            delimited(space0, tag("="), space0),
            tuple((
                opcode(opcodes, OpcodeType::Immediate),
                preceded(space1, register),
                preceded(space1, i16),
            )),
        )(input)?;
        Ok((input, (opcode, Immediate { rd, rs, value }).into()))
    }
}

fn instruction_register<'a>(
    opcodes: &'a Opcodes,
) -> impl Fn(&'a str) -> IResult<&'a str, Instruction> {
    move |input: &'a str| {
        let (input, (rd, (opcode, rs1, rs2))) = separated_pair(
            register,
            delimited(space0, tag("="), space0),
            tuple((
                opcode(opcodes, OpcodeType::Register),
                preceded(space1, register),
                preceded(space1, register),
            )),
        )(input)?;
        Ok((input, (opcode, Register { rd, rs1, rs2 }).into()))
    }
}

fn instruction_load<'a>(opcodes: &'a Opcodes) -> impl Fn(&'a str) -> IResult<&'a str, Instruction> {
    move |input: &'a str| {
        let (input, (rd, (opcode, rs, offset))) = separated_pair(
            register,
            delimited(space0, tag("="), space0),
            tuple((
                opcode(opcodes, OpcodeType::Load),
                preceded(space1, register),
                preceded(space1, u16),
            )),
        )(input)?;
        Ok((input, (opcode, Load { rd, rs, offset }).into()))
    }
}

fn instruction_store<'a>(
    opcodes: &'a Opcodes,
) -> impl Fn(&'a str) -> IResult<&'a str, Instruction> {
    move |input: &'a str| {
        let (input, ((opcode, rd, offset), rs)) = separated_pair(
            tuple((
                opcode(opcodes, OpcodeType::Store),
                preceded(space1, register),
                preceded(space1, u16),
            )),
            delimited(space0, tag("="), space0),
            register,
        )(input)?;
        Ok((input, (opcode, Store { rd, rs, offset }).into()))
    }
}

fn instruction_branch<'a>(
    opcodes: &'a Opcodes,
) -> impl Fn(&'a str) -> IResult<&'a str, Instruction> {
    move |input: &'a str| {
        let (input, (opcode, rs1, rs2, target)) = tuple((
            opcode(opcodes, OpcodeType::Branch),
            preceded(space1, register),
            preceded(space1, register),
            preceded(space1, u8),
        ))(input)?;
        Ok((input, (opcode, Branch { rs1, rs2, target }).into()))
    }
}

fn instruction_target<'a>(
    opcodes: &'a Opcodes,
) -> impl Fn(&'a str) -> IResult<&'a str, Instruction> {
    move |input: &'a str| {
        let (input, (opcode, identifier)) = tuple((
            opcode(opcodes, OpcodeType::BranchTarget),
            preceded(space1, u8),
        ))(input)?;
        Ok((input, (opcode, BranchTarget { identifier }).into()))
    }
}

fn instruction_call<'a>(opcodes: &'a Opcodes) -> impl Fn(&'a str) -> IResult<&'a str, Instruction> {
    move |input: &'a str| {
        let (input, (opcode, identifier)) =
            tuple((opcode(opcodes, OpcodeType::Call), preceded(space1, u16)))(input)?;
        Ok((input, (opcode, CallId { identifier }).into()))
    }
}

fn end_of_line(input: &str) -> IResult<&str, ()> {
    if input.is_empty() {
        Ok((input, ()))
    } else {
        let (input, _) = line_ending(input)?;
        Ok((input, ()))
    }
}

fn peol_comment(i: &str) -> IResult<&str, ()> {
    value(
        (), // Output is thrown away.
        pair(char('#'), is_not("\n\r")),
    )(i)
}

fn instruction_with_optional_comment<'a>(
    opcodes: &'a Opcodes,
) -> impl Fn(&'a str) -> IResult<&'a str, Instruction> {
    move |input: &'a str| {
        let (input, instruction) = nom::branch::alt((
            instruction_immediate(opcodes),
            instruction_register(opcodes),
            instruction_load(opcodes),
            instruction_store(opcodes),
            instruction_branch(opcodes),
            instruction_target(opcodes),
            instruction_call(opcodes),
        ))(input)?;
        let (input, _) = opt(preceded(space0, peol_comment))(input)?;
        Ok((input, instruction))
    }
}

fn instructions<'a>(input: &'a str, opcodes: &'a Opcodes) -> IResult<&'a str, Vec<Instruction>> {
    let (input, instructions) = nom::multi::many0(terminated(
        instruction_with_optional_comment(opcodes),
        end_of_line,
    ))(input)?;
    Ok((input, instructions))
}

pub fn parse(input: &str) -> Result<Vec<Instruction>, String> {
    let opcodes = Opcodes::new();
    let (_, instructions) = instructions(input, &opcodes).map_err(|e| e.to_string())?;
    Ok(instructions)
}

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
    fn test_instruction_register() {
        let opcodes = Opcodes::new();
        assert_eq!(
            instruction_register(&opcodes)("r1 = add r2 r3"),
            Ok((
                "",
                Instruction::Add(Register {
                    rd: 1,
                    rs1: 2,
                    rs2: 3
                })
            ))
        );
    }

    #[test]
    fn test_instruction_immediate() {
        let opcodes = Opcodes::new();
        assert_eq!(
            instruction_immediate(&opcodes)("r1 = addi r2 5"),
            Ok((
                "",
                Instruction::Addi(Immediate {
                    rd: 1,
                    rs: 2,
                    value: 5
                })
            ))
        );
        assert_eq!(
            instruction_immediate(&opcodes)("r1 = addi r2 -5"),
            Ok((
                "",
                Instruction::Addi(Immediate {
                    rd: 1,
                    rs: 2,
                    value: -5
                })
            ))
        );
    }

    #[test]
    fn test_instruction_load() {
        let opcodes = Opcodes::new();
        assert_eq!(
            instruction_load(&opcodes)("r1 = lb r2 5"),
            Ok((
                "",
                Instruction::Lb(Load {
                    rd: 1,
                    rs: 2,
                    offset: 5
                })
            ))
        );
    }

    #[test]
    fn test_instruction_store() {
        let opcodes = Opcodes::new();
        assert_eq!(
            instruction_store(&opcodes)("sb r2 5 = r1"),
            Ok((
                "",
                Instruction::Sb(Store {
                    rd: 2,
                    rs: 1,
                    offset: 5
                })
            ))
        );
    }

    #[test]
    fn test_instruction_branch() {
        let opcodes = Opcodes::new();
        assert_eq!(
            instruction_branch(&opcodes)("beq r1 r2 10"),
            Ok((
                "",
                Instruction::Beq(Branch {
                    rs1: 1,
                    rs2: 2,
                    target: 10
                })
            ))
        );
    }

    #[test]
    fn test_instruction_target() {
        let opcodes = Opcodes::new();
        assert_eq!(
            instruction_target(&opcodes)("target 10"),
            Ok(("", Instruction::Target(BranchTarget { identifier: 10 })))
        );
    }

    #[test]
    fn test_instruction_call() {
        let opcodes = Opcodes::new();
        assert_eq!(
            instruction_call(&opcodes)("call 10"),
            Ok(("", Instruction::Call(CallId { identifier: 10 })))
        );
    }

    #[test]
    fn test_instructions() {
        let opcodes = Opcodes::new();
        assert_eq!(
            instructions("call 10\nr1 = add r2 r3", &opcodes),
            Ok((
                "",
                vec![
                    Instruction::Call(CallId { identifier: 10 }),
                    Instruction::Add(Register {
                        rd: 1,
                        rs1: 2,
                        rs2: 3
                    })
                ]
            ))
        )
    }

    #[test]
    fn test_instructions_with_comment() {
        let opcodes = Opcodes::new();
        assert_eq!(
            instructions("call 10 # foo\nr1 = add r2 r3 # bar", &opcodes),
            Ok((
                "",
                vec![
                    Instruction::Call(CallId { identifier: 10 }),
                    Instruction::Add(Register {
                        rd: 1,
                        rs1: 2,
                        rs2: 3
                    })
                ]
            ))
        )
    }
}

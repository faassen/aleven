use crate::lang::{
    Branch, BranchOpcode, BranchTarget, BranchTargetOpcode, CallId, CallIdOpcode, Immediate,
    ImmediateOpcode, Instruction, Load, LoadOpcode, Register, RegisterOpcode, Store, StoreOpcode,
};
use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, take_while};
use nom::character::complete::{char, multispace0, multispace1};
use nom::character::complete::{i16, line_ending, space0, space1, u16, u8};
use nom::combinator::{eof, map_opt, opt, value};
use nom::error::{convert_error, Error, VerboseError};
use nom::multi::many0;
use nom::sequence::{delimited, pair, preceded, separated_pair, terminated, tuple};
use nom::IResult;
use rustc_hash::FxHashMap;
use std::fmt::Display;
use strum::IntoEnumIterator;

type ParseResult<'a, T> = IResult<&'a str, T>; // , VerboseError<&'a str>>;

#[derive(Debug)]
struct OpcodeError {}

struct AllOpcodes {
    immediate_opcodes: Opcodes<ImmediateOpcode>,
    register_opcodes: Opcodes<RegisterOpcode>,
    load_opcodes: Opcodes<LoadOpcode>,
    store_opcodes: Opcodes<StoreOpcode>,
    branch_opcodes: Opcodes<BranchOpcode>,
    branch_target_opcodes: Opcodes<BranchTargetOpcode>,
    call_id_opcodes: Opcodes<CallIdOpcode>,
}

impl AllOpcodes {
    fn new() -> AllOpcodes {
        AllOpcodes {
            immediate_opcodes: Opcodes::new(),
            register_opcodes: Opcodes::new(),
            load_opcodes: Opcodes::new(),
            store_opcodes: Opcodes::new(),
            branch_opcodes: Opcodes::new(),
            branch_target_opcodes: Opcodes::new(),
            call_id_opcodes: Opcodes::new(),
        }
    }
}

struct Opcodes<T: Display> {
    map: FxHashMap<String, T>,
}

impl<T: Display + IntoEnumIterator> Opcodes<T> {
    fn new() -> Opcodes<T> {
        let mut result = FxHashMap::default();
        for opcode in T::iter() {
            result.insert(opcode.to_string().to_lowercase(), opcode);
        }
        Opcodes { map: result }
    }

    fn get(&self, name: &str) -> Option<&T> {
        self.map.get(name)
    }
}

fn register(input: &str) -> ParseResult<u8> {
    preceded(tag("r"), u8)(input)
}

fn opcode<'a, T: Display + IntoEnumIterator + Copy>(
    opcodes: &'a Opcodes<T>,
) -> impl Fn(&'a str) -> ParseResult<'a, T> {
    move |input: &'a str| {
        map_opt(take_while(|c: char| c.is_alphanumeric()), |s| {
            opcodes.get(s).copied()
        })(input)
    }
}

fn instruction_immediate<'a>(
    opcodes: &'a Opcodes<ImmediateOpcode>,
) -> impl Fn(&'a str) -> ParseResult<'a, Instruction> {
    move |input: &'a str| {
        let (input, (rd, (opcode, rs, value))) = separated_pair(
            register,
            delimited(space0, tag("="), space0),
            tuple((
                opcode(opcodes),
                preceded(space1, register),
                preceded(space1, i16),
            )),
        )(input)?;
        Ok((
            input,
            (Instruction::Immediate(Immediate {
                opcode,
                rd,
                rs,
                value,
            })),
        ))
    }
}

fn instruction_register<'a>(
    opcodes: &'a Opcodes<RegisterOpcode>,
) -> impl Fn(&'a str) -> ParseResult<'a, Instruction> {
    move |input: &'a str| {
        let (input, (rd, (opcode, rs1, rs2))) = separated_pair(
            register,
            delimited(space0, tag("="), space0),
            tuple((
                opcode(opcodes),
                preceded(space1, register),
                preceded(space1, register),
            )),
        )(input)?;
        Ok((
            input,
            (Instruction::Register(Register {
                opcode,
                rd,
                rs1,
                rs2,
            })),
        ))
    }
}

fn instruction_load<'a>(
    opcodes: &'a Opcodes<LoadOpcode>,
) -> impl Fn(&'a str) -> ParseResult<'a, Instruction> {
    move |input: &'a str| {
        let (input, (rd, (opcode, rs, offset))) = separated_pair(
            register,
            delimited(space0, tag("="), space0),
            tuple((
                opcode(opcodes),
                preceded(space1, register),
                preceded(space1, u16),
            )),
        )(input)?;
        Ok((
            input,
            Instruction::Load(Load {
                opcode,
                rd,
                rs,
                offset,
            }),
        ))
    }
}

fn instruction_store<'a>(
    opcodes: &'a Opcodes<StoreOpcode>,
) -> impl Fn(&'a str) -> ParseResult<'a, Instruction> {
    move |input: &'a str| {
        let (input, ((opcode, rd, offset), rs)) = separated_pair(
            tuple((
                opcode(opcodes),
                preceded(space1, register),
                preceded(space1, u16),
            )),
            delimited(space0, tag("="), space0),
            register,
        )(input)?;
        Ok((
            input,
            Instruction::Store(Store {
                opcode,
                rd,
                rs,
                offset,
            }),
        ))
    }
}

fn instruction_branch<'a>(
    opcodes: &'a Opcodes<BranchOpcode>,
) -> impl Fn(&'a str) -> ParseResult<'a, Instruction> {
    move |input: &'a str| {
        let (input, (opcode, rs1, rs2, target)) = tuple((
            opcode(opcodes),
            preceded(space1, register),
            preceded(space1, register),
            preceded(space1, u8),
        ))(input)?;
        Ok((
            input,
            Instruction::Branch(Branch {
                opcode,
                rs1,
                rs2,
                target,
            }),
        ))
    }
}

fn instruction_target<'a>(
    opcodes: &'a Opcodes<BranchTargetOpcode>,
) -> impl Fn(&'a str) -> ParseResult<'a, Instruction> {
    move |input: &'a str| {
        let (input, (opcode, identifier)) = tuple((opcode(opcodes), preceded(space1, u8)))(input)?;
        Ok((
            input,
            (Instruction::BranchTarget(BranchTarget { opcode, identifier })),
        ))
    }
}

fn instruction_call<'a>(
    opcodes: &'a Opcodes<CallIdOpcode>,
) -> impl Fn(&'a str) -> ParseResult<'a, Instruction> {
    move |input: &'a str| {
        let (input, (opcode, identifier)) = tuple((opcode(opcodes), preceded(space1, u16)))(input)?;
        Ok((input, (Instruction::CallId(CallId { opcode, identifier }))))
    }
}

fn end_of_line(input: &str) -> ParseResult<()> {
    if input.is_empty() {
        Ok((input, ()))
    } else {
        let (input, _) = line_ending(input)?;
        Ok((input, ()))
    }
}

fn whitespace(input: &str) -> ParseResult<()> {
    value((), multispace1)(input)
}

fn peol_comment(input: &str) -> ParseResult<()> {
    value(
        (), // Output is thrown away.
        pair(char('#'), is_not("\n\r")),
    )(input)
}

fn whitespace_and_comments(input: &str) -> ParseResult<()> {
    value((), many0(alt((whitespace, peol_comment))))(input)
}

fn instruction<'a>(opcodes: &'a AllOpcodes) -> impl Fn(&'a str) -> ParseResult<'a, Instruction> {
    move |input: &'a str| {
        alt((
            instruction_immediate(&opcodes.immediate_opcodes),
            instruction_register(&opcodes.register_opcodes),
            instruction_load(&opcodes.load_opcodes),
            instruction_store(&opcodes.store_opcodes),
            instruction_branch(&opcodes.branch_opcodes),
            instruction_target(&opcodes.branch_target_opcodes),
            instruction_call(&opcodes.call_id_opcodes),
        ))(input)
    }
}

fn instruction_with_optional_comment<'a>(
    opcodes: &'a AllOpcodes,
) -> impl Fn(&'a str) -> ParseResult<'a, Instruction> {
    move |input: &'a str| {
        let (input, instruction) = instruction(opcodes)(input)?;
        let (input, _) = space0(input)?;
        let (input, _) = opt(peol_comment)(input)?;
        Ok((input, instruction))
    }
}

fn instructions<'a>(
    opcodes: &'a AllOpcodes,
) -> impl Fn(&'a str) -> ParseResult<'a, Vec<Instruction>> {
    move |input: &'a str| {
        terminated(
            nom::multi::many0(delimited(
                whitespace_and_comments,
                terminated(instruction_with_optional_comment(opcodes), end_of_line),
                whitespace_and_comments,
            )),
            eof,
        )(input)
    }
}

/// Parse a vector of instructions from a string
pub fn parse(input: &str) -> Result<Vec<Instruction>, String> {
    let opcodes = AllOpcodes::new();
    let (_, instructions) = instructions(&opcodes)(input).map_err(|e| e.to_string())?;
    Ok(instructions)
}

// pub fn parse_program(input: &str) -> Result<Program, String> {}

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
        assert_eq!(opcodes.get("addi"), Some(&ImmediateOpcode::Addi));
    }

    #[test]
    fn test_opcode() {
        let opcodes = Opcodes::<ImmediateOpcode>::new();
        assert_eq!(opcode(&opcodes)("addi"), Ok(("", ImmediateOpcode::Addi)));

        let opcodes = Opcodes::<RegisterOpcode>::new();
        assert_error!(opcode(&opcodes)("addi"));
    }

    #[test]
    fn test_instruction_register() {
        let opcodes = Opcodes::new();
        assert_eq!(
            instruction_register(&opcodes)("r1 = add r2 r3"),
            Ok((
                "",
                Instruction::Register(Register {
                    opcode: RegisterOpcode::Add,
                    rd: 1,
                    rs1: 2,
                    rs2: 3
                })
            ))
        );
    }

    #[test]
    fn test_instruction_register_broken() {
        let opcodes = Opcodes::new();
        assert_error!(instruction_register(&opcodes)("r1 = foobar r2 r3"),);
    }

    #[test]
    fn test_instruction_immediate() {
        let opcodes = Opcodes::new();
        assert_eq!(
            instruction_immediate(&opcodes)("r1 = addi r2 5"),
            Ok((
                "",
                Instruction::Immediate(Immediate {
                    opcode: ImmediateOpcode::Addi,
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
                Instruction::Immediate(Immediate {
                    opcode: ImmediateOpcode::Addi,
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
                Instruction::Load(Load {
                    opcode: LoadOpcode::Lb,
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
                Instruction::Store(Store {
                    opcode: StoreOpcode::Sb,
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
                Instruction::Branch(Branch {
                    opcode: BranchOpcode::Beq,
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
            Ok((
                "",
                Instruction::BranchTarget(BranchTarget {
                    opcode: BranchTargetOpcode::Target,
                    identifier: 10
                })
            ))
        );
    }

    #[test]
    fn test_instruction_call() {
        let opcodes = Opcodes::new();
        assert_eq!(
            instruction_call(&opcodes)("call 10"),
            Ok((
                "",
                Instruction::CallId(CallId {
                    opcode: CallIdOpcode::Call,
                    identifier: 10
                })
            ))
        );
    }

    #[test]
    fn test_instruction_broken() {
        let opcodes = AllOpcodes::new();
        assert_error!(instruction(&opcodes)("r1 = broken r2 5"),);
    }

    #[test]
    fn test_instruction_with_optional_comment_broken() {
        let opcodes = AllOpcodes::new();
        assert_error!(instruction_with_optional_comment(&opcodes)(
            "r1 = broken r2 5"
        ),);
    }

    #[test]
    fn test_instructions() {
        let opcodes = AllOpcodes::new();
        let r = instructions(&opcodes)("call 10\nr1 = add r2 r3");

        assert_eq!(
            r,
            Ok((
                "",
                vec![
                    Instruction::CallId(CallId {
                        opcode: CallIdOpcode::Call,
                        identifier: 10
                    }),
                    Instruction::Register(Register {
                        opcode: RegisterOpcode::Add,
                        rd: 1,
                        rs1: 2,
                        rs2: 3
                    })
                ]
            ))
        )
    }

    #[test]
    fn test_instructions_first_broken_opcode() {
        let opcodes = AllOpcodes::new();
        let r = instructions(&opcodes)("r1 = broken r2 r3");
        assert_error!(r)
    }

    #[test]
    fn test_instructions_second_broken_opcode() {
        let opcodes = AllOpcodes::new();
        let r = instructions(&opcodes)("call 10\nr1 = broken r2 r3");
        assert_error!(r)
    }

    #[test]
    fn test_instructions_with_comment() {
        let opcodes = AllOpcodes::new();
        let r = instructions(&opcodes)("call 10 # foo\nr1 = add r2 r3 # bar");
        assert_eq!(
            r,
            Ok((
                "",
                vec![
                    Instruction::CallId(CallId {
                        opcode: CallIdOpcode::Call,
                        identifier: 10
                    }),
                    Instruction::Register(Register {
                        opcode: RegisterOpcode::Add,
                        rd: 1,
                        rs1: 2,
                        rs2: 3
                    })
                ]
            ))
        )
    }

    #[test]
    fn test_instructions_with_blank_lines() {
        let opcodes = AllOpcodes::new();
        let r = instructions(&opcodes)("call 10\n\nr1 = add r2 r3");
        assert_eq!(
            r,
            Ok((
                "",
                vec![
                    Instruction::CallId(CallId {
                        opcode: CallIdOpcode::Call,
                        identifier: 10
                    }),
                    Instruction::Register(Register {
                        opcode: RegisterOpcode::Add,
                        rd: 1,
                        rs1: 2,
                        rs2: 3
                    })
                ]
            ))
        );
    }

    #[test]
    fn test_whitespace_and_comments() {
        assert_eq!(whitespace_and_comments(""), Ok(("", ())));
        assert_eq!(whitespace_and_comments(" "), Ok(("", ())));
        assert_eq!(whitespace_and_comments(" \n "), Ok(("", ())));
        assert_eq!(whitespace_and_comments(" # foo"), Ok(("", ())));
        assert_eq!(whitespace_and_comments(" # foo\n # bar"), Ok(("", ())));
        assert_eq!(whitespace_and_comments(" yes yes"), Ok(("yes yes", ())));
    }

    #[test]
    fn test_instructions_with_comment_lines() {
        let opcodes = AllOpcodes::new();
        let r = instructions(&opcodes)("call 10\n# this is a comment \nr1 = add r2 r3");
        assert_eq!(
            r,
            Ok((
                "",
                vec![
                    Instruction::CallId(CallId {
                        opcode: CallIdOpcode::Call,
                        identifier: 10
                    }),
                    Instruction::Register(Register {
                        opcode: RegisterOpcode::Add,
                        rd: 1,
                        rs1: 2,
                        rs2: 3
                    })
                ]
            ))
        )
    }

    #[test]
    fn test_parse() {
        let instructions = parse(
            "
        r2 = lb r1 0
        r2 = srli r2 2
        sh r3 10 = r2
        ",
        )
        .unwrap();
        let expected_instructions = [
            Instruction::Load(Load {
                opcode: LoadOpcode::Lb,
                offset: 0,
                rs: 1,
                rd: 2,
            }),
            Instruction::Immediate(Immediate {
                opcode: ImmediateOpcode::Srli,
                value: 2,
                rs: 2,
                rd: 2,
            }),
            Instruction::Store(Store {
                opcode: StoreOpcode::Sh,
                offset: 10,
                rs: 2,
                rd: 3,
            }),
        ];
        assert_eq!(instructions, expected_instructions);
    }

    #[test]
    fn test_parse_trailing_space() {
        let instructions = parse(
            "
        r2 = lb r1 0
        r2 = srli r2 2 
        sh r3 10 = r2
        ",
        )
        .unwrap();
        let expected_instructions = [
            Instruction::Load(Load {
                opcode: LoadOpcode::Lb,
                offset: 0,
                rs: 1,
                rd: 2,
            }),
            Instruction::Immediate(Immediate {
                opcode: ImmediateOpcode::Srli,
                value: 2,
                rs: 2,
                rd: 2,
            }),
            Instruction::Store(Store {
                opcode: StoreOpcode::Sh,
                offset: 10,
                rs: 2,
                rd: 3,
            }),
        ];
        assert_eq!(instructions, expected_instructions);
    }

    #[test]
    fn test_parse_unknown_opcode() {
        assert!(parse(
            "
        r2 = lb r1 0
        r2 = broken r2 2
        sh r3 10 = r2
        ",
        )
        .is_err());
    }
}

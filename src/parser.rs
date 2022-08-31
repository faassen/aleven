use crate::lang::Opcode;
use crate::lang::{Instruction, Register};
use nom::bytes::complete::{tag, take, take_while, take_while_m_n};
use nom::character::complete::{space0, space1, u8};
use nom::combinator::{flat_map, map_opt, map_res};
use nom::number::complete::be_u16;
use nom::IResult;
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

fn register_nr(input: &str) -> IResult<&str, u8> {
    u8(input)
}

fn register(input: &str) -> IResult<&str, u8> {
    let (input, _) = tag("r")(input)?;
    register_nr(input)
}

fn opcode<'a>(input: &'a str, opcodes: &'a Opcodes) -> IResult<&'a str, &'a Opcode> {
    map_opt(take_while(|c: char| c.is_alphanumeric()), |s| {
        opcodes.get(s)
    })(input)
}

fn assign(input: &str) -> IResult<&str, ()> {
    let (input, _) = space0(input)?;
    let (input, _) = tag("=")(input)?;
    let (input, _) = space0(input)?;
    Ok((input, ()))
}

fn register_assign(input: &str) -> IResult<&str, u8> {
    let (input, register) = register(input)?;
    let (input, _) = assign(input)?;
    Ok((input, register))
}

fn opcode_register<'a>(
    input: &'a str,
    opcodes: &'a Opcodes,
) -> IResult<&'a str, (Opcode, u8, u8, u8)> {
    let (input, rd) = register(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = tag("=")(input)?;
    let (input, _) = space0(input)?;
    let (input, opcode) = opcode(input, opcodes)?;
    let (input, _) = space1(input)?;
    let (input, rs1) = register(input)?;
    let (input, _) = space1(input)?;
    let (input, rs2) = register(input)?;
    Ok((input, (*opcode, rd, rs1, rs2)))
}
// r1 = addi 15 r0
// r2 = add r3 r4
// r1 = lb 10 r0
// sb 10 r0 = r1
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

        assert_eq!(opcode("addi", &opcodes), Ok(("", &Opcode::Addi)));
    }

    #[test]
    fn test_assign() {
        assert_eq!(assign("=whatever"), Ok(("whatever", ())));
        assert_eq!(assign(" =whatever"), Ok(("whatever", ())));
        assert_eq!(assign(" = whatever"), Ok(("whatever", ())));
    }

    #[test]
    fn test_register_assign() {
        assert_eq!(register_assign("r1 = whatever"), Ok(("whatever", 1)));
        assert_eq!(register_assign("r10 = whatever"), Ok(("whatever", 10)));
    }

    #[test]
    fn test_opcode_register() {
        let opcodes = Opcodes::new();
        assert_eq!(
            opcode_register("r1 = add r2 r3", &opcodes),
            Ok(("", (Opcode::Add, 1, 2, 3)))
        );
    }
}

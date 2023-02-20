use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case},
    character::complete::{i64, line_ending, space0},
    combinator::map,
    error::VerboseError,
    sequence::{delimited},
    IResult,
};
use redcode::{
    AddrMode, AddrMode::*, Modifier, Modifier::*, Opcode, Opcode::*,
};

pub fn eol(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    line_ending(input)
}

pub fn number(input: &str) -> IResult<&str, i64, VerboseError<&str>> {
    delimited(space0, only_number, space0)(input)
}

fn only_number(input: &str) -> IResult<&str, i64, VerboseError<&str>> {
    if input.starts_with("+-") {
        // If we are prefixed by "+", we shouldn't also be prefixed by "-"
        unimplemented!()
    } else if let Some(stripped_input) = input.strip_prefix('+') {
        // If we are prefixed by "+", parse w/o "+"
        i64(stripped_input)
    } else {
        // not prefixed by "+", attempt to parse normally
        i64(input)
    }
}

// Consumes exactly one opcode and returns the enum
pub fn opcode(input: &str) -> IResult<&str, Opcode, VerboseError<&str>> {
    alt((
        map(tag_no_case("DAT"), |_| Dat),
        map(tag_no_case("MOV"), |_| Mov),
        map(tag_no_case("ADD"), |_| Add),
        map(tag_no_case("SUB"), |_| Sub),
        map(tag_no_case("MUL"), |_| Mul),
        map(tag_no_case("DIV"), |_| Div),
        map(tag_no_case("MOD"), |_| Mod),
        map(tag_no_case("JMP"), |_| Jmp),
        map(tag_no_case("JMZ"), |_| Jmz),
        map(tag_no_case("JMN"), |_| Jmn),
        map(tag_no_case("DJN"), |_| Djn),
        map(tag_no_case("SPL"), |_| Spl),
        map(tag_no_case("SLT"), |_| Slt),
        map(tag_no_case("CMP"), |_| Cmp),
        map(tag_no_case("SEQ"), |_| Seq),
        map(tag_no_case("SNE"), |_| Sne),
        map(tag_no_case("NOP"), |_| Nop),
        map(tag_no_case("LDP"), |_| Ldp),
        map(tag_no_case("STP"), |_| Stp),
    ))(input)
}

pub fn addr_mode(input: &str) -> IResult<&str, AddrMode, VerboseError<&str>> {
    alt((
        map(tag("#"), |_| Immediate),
        map(tag("$"), |_| Direct),
        map(tag("*"), |_| IndirectA),
        map(tag("@"), |_| IndirectB),
        map(tag("{"), |_| PredecA),
        map(tag("<"), |_| PredecB),
        map(tag("}"), |_| PostincA),
        map(tag(">"), |_| PostincB),
    ))(input)
}

pub fn modifier(input: &str) -> IResult<&str, Modifier, VerboseError<&str>> {
    alt((
        map(tag_no_case("AB"), |_| AB),
        map(tag_no_case("BA"), |_| BA),
        map(tag_no_case("A"), |_| A),
        map(tag_no_case("B"), |_| B),
        map(tag_no_case("X"), |_| X),
        map(tag_no_case("F"), |_| F),
        map(tag_no_case("I"), |_| I),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_opcode_parsing() {
        assert_eq!(opcode("DAT"), Ok(("", Dat)));
        assert_eq!(opcode("dAtfollowingcrap"), Ok(("followingcrap", Dat)));
        assert!(!opcode(" dat").is_ok());
    }

    #[test]
    fn check_mode_parsing() {
        assert_eq!(addr_mode("**"), Ok(("*", IndirectA)));
        assert_eq!(
            addr_mode("#followingcrap"),
            Ok(("followingcrap", Immediate))
        );
        assert!(!addr_mode(" {").is_ok());
    }

    #[test]
    fn check_modifier_parsing() {
        assert_eq!(modifier("BA"), Ok(("", BA)));
        assert_eq!(modifier("B A"), Ok((" A", B)));
        assert!(!modifier(" a b").is_ok());
    }
}

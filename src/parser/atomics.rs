use crate::redcode::{AddrMode, AddrMode::*, Modifier, Modifier::*, Opcode, Opcode::*};

use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case},
    character::complete::digit1,
    combinator::{map, map_res},
    error::VerboseError,
    sequence::preceded,
    IResult,
};

// This function was adapted from the s_expr parser in the nom examples folder
/// parse an i32 decimal string
pub fn parse_num(input: &str) -> IResult<&str, i32, VerboseError<&str>> {
    // This was written when I was dumb.  It can be done with only s.parse.  TODO...
    alt((
        map_res(digit1, |s: &str| s.parse::<i32>()),
        map_res(preceded(tag("-"), digit1), |s: &str| {
            let res = s.parse::<i32>();
            // Is there a shorthand for modifying one case of the structural decomposition?
            match res {
                Ok(num_val) => Ok(-num_val),
                Err(e) => Err(e),
            }
        }),
    ))(input)
}

/// consumes exactly one opcode and returns the enum
pub fn parse_opcode(input: &str) -> IResult<&str, Opcode, VerboseError<&str>> {
    alt((
        map(tag_no_case("DAT"), { |_| Dat }),
        map(tag_no_case("MOV"), { |_| Mov }),
        map(tag_no_case("ADD"), { |_| Add }),
        map(tag_no_case("SUB"), { |_| Sub }),
        map(tag_no_case("MUL"), { |_| Mul }),
        map(tag_no_case("DIV"), { |_| Div }),
        map(tag_no_case("MOD"), { |_| Mod }),
        map(tag_no_case("JMP"), { |_| Jmp }),
        map(tag_no_case("JMZ"), { |_| Jmz }),
        map(tag_no_case("JMN"), { |_| Jmn }),
        map(tag_no_case("DJN"), { |_| Djn }),
        map(tag_no_case("SPL"), { |_| Spl }),
        map(tag_no_case("SLT"), { |_| Slt }),
        map(tag_no_case("CMP"), { |_| Cmp }),
        map(tag_no_case("SEQ"), { |_| Seq }),
        map(tag_no_case("SNE"), { |_| Sne }),
        map(tag_no_case("NOP"), { |_| Nop }),
        map(tag_no_case("LDP"), { |_| Ldp }),
        map(tag_no_case("STP"), { |_| Stp }),
    ))(input)
}

/// consumes exactly one modifier and returns the enum
pub fn parse_modifier(input: &str) -> IResult<&str, Modifier, VerboseError<&str>> {
    alt((
        // AB and BA must come before B and A to be matched greedly
        map(tag_no_case("AB"), { |_| AB }),
        map(tag_no_case("BA"), { |_| BA }),
        map(tag_no_case("A"), { |_| A }),
        map(tag_no_case("B"), { |_| B }),
        map(tag_no_case("F"), { |_| F }),
        map(tag_no_case("X"), { |_| X }),
        map(tag_no_case("I"), { |_| I }),
    ))(input)
}

/// consumes exactly one mode and returns the enum
pub fn parse_mode(input: &str) -> IResult<&str, AddrMode, VerboseError<&str>> {
    alt((
        map(tag("#"), { |_| Immediate }),
        map(tag("$"), { |_| Direct }),
        map(tag("@"), { |_| IndirectB }),
        map(tag("<"), { |_| PredecB }),
        map(tag(">"), { |_| PostincB }),
        map(tag("*"), { |_| IndirectA }),
        map(tag("{"), { |_| PredecA }),
        map(tag("}"), { |_| PostincA }),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    /// Tests that positive nubers are parsed and consumed correctly
    fn parse_pos_num() {
        assert_eq!(parse_num("12345"), Ok(("", 12345)));
        assert_eq!(
            parse_num("12345 123 extra junk on the end"),
            Ok((" 123 extra junk on the end", 12345))
        );
        assert_eq!(parse_num("").is_err(), true);
        assert_eq!(parse_num(" 123").is_ok(), false);
        assert_eq!(parse_num("").is_ok(), false);
    }

    #[test]
    fn parse_neg_num() {
        assert_eq!(parse_num("-12345"), Ok(("", -12345)));
        assert_eq!(
            parse_num("-12345 123 extra junk on the end"),
            Ok((" 123 extra junk on the end", -12345))
        );
        assert_eq!(parse_num("-").is_err(), true);
        assert_eq!(parse_num(" -123").is_ok(), false);
        assert_eq!(parse_num("- 123").is_ok(), false);
    }

    #[test]
    fn check_opcode_parsing() {
        assert_eq!(parse_opcode("DAT"), Ok(("", Dat)));
        assert_eq!(parse_opcode("dAtfollowingcrap"), Ok(("followingcrap", Dat)));
        assert_eq!(parse_opcode(" dat").is_ok(), false);
    }

    #[test]
    fn check_mode_parsing() {
        assert_eq!(parse_mode("**"), Ok(("*", IndirectA)));
        assert_eq!(
            parse_mode("#followingcrap"),
            Ok(("followingcrap", Immediate))
        );
        assert_eq!(parse_mode(" {").is_ok(), false);
    }

    #[test]
    fn check_modifier_parsing() {
        assert_eq!(parse_modifier("BA"), Ok(("", BA)));
        assert_eq!(parse_modifier("B A"), Ok((" A", B)));
        assert_eq!(parse_modifier(" a b").is_ok(), false);
    }
}

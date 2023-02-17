use std::error::Error;

use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, tag_no_case},
    character::complete::{i64, line_ending, space0, u64},
    combinator::{eof, map, opt, recognize},
    error::VerboseError,
    sequence::{delimited, pair, preceded, tuple},
    Err, Finish, IResult,
};
use redcode::{
    default_modifiers, AddrMode, AddrMode::*, Instruction, Modifier,
    Modifier::*, Opcode, Opcode::*, RelaxedCompleteInstruction, RelaxedWarrior,
};

/// Formal grammer of a redcode loadfile
///     Loadfiles are rigid, and do not permit omitted fields, extra lines, etc.
///     Adapted from https://corewar.co.uk/standards/icws94.htm with additions from http://www.koth.org/info/pmars-redcode-94.txt
///
/// The syntax that marzipan accepts is more permissive in a number of ways than
/// required by ICWS.
///  * All whitespace is optional excluding newlines
///  * PSPACE and pmars extensions are accepted including opcodes, modes, and
///    PIN
///  * ORG and END are accepted.  These are underspecified in ICWS for the
///    loadfile but used in PMARS
///
/// Grammer follows relatively standard rules:
///  * ^A means any symbol except for A.
///  * A* means 0 or more occurrences of A.
///  * A+ means 1 or more occurrences of A.
///  * A? means 0 or one occurrences of A.
///  * A | B means either A or B.
///  * A B means the symbol A followed by the symbol B.
///  * (A B ...) means the symbols A B ... grouped.
/// ```
/// // loadfile:
/// //     list
/// //
/// // list:
/// //     line | line list
/// //
/// // line:
/// //     whitespace (comment | instruction | pseudoop)? whitespace eol
/// //
/// // comment:
/// //     ; text
/// //
/// // instruction:
/// //     opcode . modifier whitespace mode whitespace number whitespace , whitespace mode whitespace number
/// //
/// // opcode: (ignoring case)
/// //     DAT | MOV | ADD | SUB | MUL | DIV | MOD | JMP | JMZ | JMN | DJN | SPL | SLT | CMP | SEQ |
/// //     SNE | NOP | LDP | STP
/// //
/// // pseudoop:
/// //     ORG | ORG whitespace number | END | END whitespace number
/// //
/// // modifier:
/// //     A | B | AB | BA | F | X | I
/// //
/// // whitespace:
/// //     (SPACE | HORIZONTAL_TAB)*
/// //
/// // mode:
/// //     # | $ | @ | < | > | * | { | }
/// //
/// // number:
/// //     ( + | - )? ( 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 )+
/// //
/// // eol:
/// //     \n | \r\n
/// //
/// // text
/// //     (^eol)*
/// ```
pub fn parse(
    warrior: &str,
    ommit_modifier: bool,
) -> Result<RelaxedWarrior, Err<VerboseError<&str>>> {
    let mut input = warrior;
    let mut instructions = vec![];
    let mut start = None;
    let mut pin = None;

    while !input.is_empty() {
        match parse_line(input, ommit_modifier) {
            Ok((leftover, e)) => {
                input = leftover;
                match e {
                    LineContent::Empty() => {}
                    LineContent::Comment(_) => {}
                    LineContent::Eof() => {
                        return Ok(RelaxedWarrior {
                            code: instructions,
                            start: start.unwrap_or(0),
                            pin,
                        })
                    }
                    LineContent::Pin(val) => pin = Some(val),
                    LineContent::Instruction(instr) => instructions.push(instr),
                    LineContent::Org(Some(e)) => start = Some(e),
                    LineContent::Org(None) => start = Some(0),
                    LineContent::End(Some(e)) => {
                        return Ok(RelaxedWarrior {
                            code: instructions,
                            start: e,
                            pin,
                        })
                    }
                    LineContent::End(None) => {
                        return Ok(RelaxedWarrior {
                            code: instructions,
                            start: start.unwrap_or(0),
                            pin,
                        })
                    }
                }
            }
            Err(e) => return Err(e),
        }
    }
    Ok(RelaxedWarrior::default())
}

pub fn parse_instr<'a>(
    line: &'a str,
    ommit_modifier: bool,
) -> Result<RelaxedCompleteInstruction, Box<dyn Error + 'a>> {
    match parse_line(line, ommit_modifier).finish() {
        Ok((_, LineContent::Instruction(instr))) => Ok(instr),
        Ok((_, _content)) => Err("Parsed the line not as an instruction but \
                                  as something else"
            .into()),
        Err(e) => Err(Box::new(e)),
    }
}

// Conventions:
//  - parsers for objects inside lines don't consume the newline characters.
//  - parsers consume internal whitespace but not surrounding whitespace
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum LineContent<'a> {
    Comment(&'a str),
    Instruction(RelaxedCompleteInstruction),
    Empty(),
    Org(Option<i64>),
    End(Option<i64>),
    Pin(u64),
    Eof(),
}

fn parse_line(
    input: &str,
    ommit_modifier: bool,
) -> IResult<&str, LineContent, VerboseError<&str>> {
    let parse_instr = match ommit_modifier {
        true => parse_loadfile_88_instr,
        false => parse_loadfile_94_instr,
    };
    alt((
        // Parse a line with only 0 or more whitespace characters
        map(pair(space0, line_ending), |_| LineContent::Empty()),
        // Parse a line containing an instruction possibly with surrounding
        // whitespace
        map(
            delimited(space0, parse_instr, pair(space0, line_ending)),
            LineContent::Instruction,
        ),
        // Parse a line with a comment possibly preceded by whitespace
        map(
            delimited(space0, parse_comment, pair(space0, line_ending)),
            LineContent::Comment,
        ),
        // Parse an ORG pseudoop without an address (default to 0)
        map(
            delimited(space0, tag_no_case("ORG"), pair(space0, line_ending)),
            |_| LineContent::Org(None),
        ),
        // Parse an END pseudoop with the starting address, ending with EOF or
        // linebreak
        map(
            delimited(
                pair(space0, tag_no_case("END")),
                preceded(space0, i64),
                pair(space0, alt((eof, line_ending))),
            ),
            |num| LineContent::End(Some(num)),
        ),
        // Parse an END pseudoop without an address.  Any junk can can come
        // after end. For that reason, this must be checked at lower
        // priority than END followed by a start address
        map(preceded(space0, tag_no_case("END")), |_| {
            LineContent::End(None)
        }),
        // Parse an ORG pseudoop with the starting address
        map(
            delimited(
                pair(space0, tag_no_case("ORG")),
                preceded(space0, i64),
                pair(space0, line_ending),
            ),
            |num| LineContent::Org(Some(num)),
        ),
        // Parse a PIN pseudo op
        map(
            delimited(
                pair(space0, tag_no_case("PIN")),
                preceded(space0, u64),
                pair(space0, line_ending),
            ),
            LineContent::Pin,
        ),
        // Parse a line containing possible whitespace and an EOF or end of
        // input
        map(pair(space0, eof), |_| LineContent::Eof()),
    ))(input)
}

fn parse_loadfile_88_instr(
    input: &str,
) -> IResult<&str, RelaxedCompleteInstruction, VerboseError<&str>> {
    let tuple_instruction = tuple((
        parse_opcode,
        space0,
        parse_addr_mode,
        space0,
        i64,
        space0,
        tag(","),
        space0,
        parse_addr_mode,
        space0,
        i64,
    ))(input);
    match tuple_instruction {
        Ok((
            leftover,
            (
                opcode,
                _,
                a_addr_mode,
                _,
                a_field,
                _,
                _,
                _,
                b_addr_mode,
                _,
                b_field,
            ),
        )) => Ok((
            leftover,
            RelaxedCompleteInstruction {
                instr: Instruction {
                    opcode,
                    modifier: default_modifiers(
                        opcode,
                        a_addr_mode,
                        b_addr_mode,
                    ),
                    a_addr_mode,
                    b_addr_mode,
                },
                a_field,
                b_field,
            },
        )),
        Err(e) => Err(e),
    }
}

/// Produces an Instruction, consuming line content but not the newline
fn parse_loadfile_94_instr(
    input: &str,
) -> IResult<&str, RelaxedCompleteInstruction, VerboseError<&str>> {
    let tuple_instruction = tuple((
        parse_opcode,
        tag("."),
        parse_modifier,
        space0,
        parse_addr_mode,
        space0,
        i64,
        space0,
        tag(","),
        space0,
        parse_addr_mode,
        space0,
        i64,
    ))(input);
    match tuple_instruction {
        Ok((
            leftover,
            (
                opcode,
                _,
                modifier,
                _,
                a_addr_mode,
                _,
                a_field,
                _,
                _,
                _,
                b_addr_mode,
                _,
                b_field,
            ),
        )) => Ok((
            leftover,
            RelaxedCompleteInstruction {
                instr: Instruction {
                    opcode,
                    modifier,
                    a_addr_mode,
                    b_addr_mode,
                },
                a_field,
                b_field,
            },
        )),
        Err(e) => Err(e),
    }
}

fn parse_comment(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    preceded(tag(";"), recognize(opt(is_not("\r\n"))))(input)
}

// Consumes exactly one opcode and returns the enum
fn parse_opcode(input: &str) -> IResult<&str, Opcode, VerboseError<&str>> {
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

fn parse_addr_mode(input: &str) -> IResult<&str, AddrMode, VerboseError<&str>> {
    alt((
        map(tag_no_case("#"), |_| Immediate),
        map(tag_no_case("$"), |_| Direct),
        map(tag_no_case("*"), |_| IndirectA),
        map(tag_no_case("@"), |_| IndirectB),
        map(tag_no_case("{"), |_| PredecA),
        map(tag_no_case("<"), |_| PredecB),
        map(tag_no_case("}"), |_| PostincA),
        map(tag_no_case(">"), |_| PostincB),
    ))(input)
}

fn parse_modifier(input: &str) -> IResult<&str, Modifier, VerboseError<&str>> {
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
        assert_eq!(parse_opcode("DAT"), Ok(("", Dat)));
        assert_eq!(
            parse_opcode("dAtfollowingcrap"),
            Ok(("followingcrap", Dat))
        );
        assert_eq!(parse_opcode(" dat").is_ok(), false);
    }

    #[test]
    fn check_mode_parsing() {
        assert_eq!(parse_addr_mode("**"), Ok(("*", IndirectA)));
        assert_eq!(
            parse_addr_mode("#followingcrap"),
            Ok(("followingcrap", Immediate))
        );
        assert_eq!(parse_addr_mode(" {").is_ok(), false);
    }

    #[test]
    fn check_modifier_parsing() {
        assert_eq!(parse_modifier("BA"), Ok(("", BA)));
        assert_eq!(parse_modifier("B A"), Ok((" A", B)));
        assert_eq!(parse_modifier(" a b").is_ok(), false);
    }

    #[test]
    fn test_comment() {
        assert_eq!(
            parse_comment(";1234\nabc"),
            Ok(("\nabc", "1234")),
            "The comment parser should return the content of the comment \
             excluding the \";\" without consuming the line ending"
        );
        assert_eq!(
            parse_comment(";\n"),
            Ok(("\n", "")),
            "The comment parser should accept empty comments"
        );
        assert_eq!(
            parse_comment(";asdf\r\nabc"),
            Ok(("\r\nabc", "asdf")),
            "The comment parser should accept carrage return + newline style \
             line endings"
        );
        assert_eq!(
            parse_comment("; asdf\n"),
            Ok(("\n", " asdf")),
            "The comment parser shouldn't consume the line ending after a \
             commend"
        );
    }
}

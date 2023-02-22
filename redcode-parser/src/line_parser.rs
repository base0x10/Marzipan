use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, tag_no_case},
    character::complete::space0,
    combinator::map,
    error::VerboseError,
    sequence::{preceded, tuple},
    IResult,
};
use redcode::{default_modifiers, Instruction, RelaxedCompleteInstruction};

use crate::loadfile_primitives::{addr_mode, modifier, number, opcode};

/// Parses the content of a line containing a '94 style instruction without
/// consuming eol
pub fn instr_94_line(
    input: &str,
) -> IResult<&str, RelaxedCompleteInstruction, VerboseError<&str>> {
    let tuple_instruction = tuple((
        space0,
        opcode,
        tag("."),
        modifier,
        space0,
        addr_mode,
        space0,
        number,
        space0,
        tag(","),
        space0,
        addr_mode,
        space0,
        number,
    ))(input);
    match tuple_instruction {
        Ok((
            leftover,
            (
                _,
                opcode,
                _, // "."
                modifier,
                _, // space0
                a_addr_mode,
                _, // space0
                a_field,
                _, // space0
                _, // ","
                _, // space0
                b_addr_mode,
                _, // space0
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

/// Parses the content of a line containing a '88 style instruction without
/// consuming eol
pub fn instr_88_line(
    input: &str,
) -> IResult<&str, RelaxedCompleteInstruction, VerboseError<&str>> {
    let tuple_instruction = tuple((
        space0,
        opcode,
        space0,
        addr_mode,
        space0,
        number,
        space0,
        tag(","),
        space0,
        addr_mode,
        space0,
        number,
    ))(input);
    match tuple_instruction {
        Ok((
            leftover,
            (
                _, // space0
                opcode,
                _, // space0
                a_addr_mode,
                _, // space0
                a_field,
                _, // space0
                _, // ","
                _, // space0
                b_addr_mode,
                _, // space0,
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

/// parses the content of a comment line without consuming eol
pub fn comment_line(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    preceded(preceded(space0, tag(";")), alt((is_not("\r\n"), space0)))(input)
}

/// parses the content of an ORG line without consuming eol
pub fn org_line(input: &str) -> IResult<&str, i64, VerboseError<&str>> {
    let prefix = preceded(space0, tag_no_case("ORG"));
    preceded(prefix, number)(input)
}

/// parses the content of a PIN line without consuming eol
pub fn pin_line(input: &str) -> IResult<&str, i64, VerboseError<&str>> {
    let prefix = preceded(space0, tag_no_case("PIN"));
    preceded(prefix, number)(input)
}

/// parses the content of an END line without consuming eol (or eof)
pub fn end_line(input: &str) -> IResult<&str, Option<i64>, VerboseError<&str>> {
    let prefix = preceded(space0, tag_no_case("END"));
    let maybe_num = alt((map(number, Some), map(space0, |_| None)));
    preceded(prefix, maybe_num)(input)
}

/// matches the content of an empty line without consuming an eol or EOF
/// This is explicitly allowed to not consume any input and return success
pub fn empty_line(input: &str) -> IResult<&str, (), VerboseError<&str>> {
    map(space0, |_| ())(input)
}

#[cfg(test)]
mod tests {
    use coverage_helper::test;
    use redcode::{test_utils, CompleteInstruction};

    use super::*;

    #[test]
    fn test_all_94_instrs() {
        for instruction in test_utils::all_instructions() {
            let expected = CompleteInstruction {
                instr: instruction.clone(),
                a_field: 1234,
                b_field: 1234,
            };
            let input = expected.to_string();

            assert_eq!(
                instr_94_line(&input).map(|(leftover, instr)| (
                    leftover,
                    instr.normalize(8000u32)
                )),
                Ok(("", expected)),
                "The instruction parser should parse the instruction \"{}\"",
                input
            );
        }
    }

    #[test]
    fn test_all_88_instrs() {
        for instruction in test_utils::all_instructions() {
            // Convert from existing 94 style instruction with its modifier to
            // the 88 style instruction with the implied default modifier
            let expected = CompleteInstruction {
                instr: Instruction {
                    opcode: instruction.opcode,
                    modifier: default_modifiers(
                        instruction.opcode,
                        instruction.a_addr_mode,
                        instruction.b_addr_mode,
                    ),
                    a_addr_mode: instruction.a_addr_mode,
                    b_addr_mode: instruction.b_addr_mode,
                },
                a_field: 1234,
                b_field: 1234,
            };
            let input = format!(
                "{} {}{}, {}{}",
                instruction.opcode,
                instruction.a_addr_mode,
                1234,
                instruction.b_addr_mode,
                1234
            );

            assert_eq!(
                instr_88_line(&input).map(|(leftover, instr)| (
                    leftover,
                    instr.normalize(8000u32)
                )),
                Ok(("", expected)),
                "The instruction parser should parse the instruction \"{}\"",
                input
            );
        }
    }

    #[test]
    fn test_comment() {
        assert_eq!(
            comment_line(";1234\nabc"),
            Ok(("\nabc", "1234")),
            "The comment parser should return the content of the comment \
             excluding the \";\" without consuming the line ending"
        );
        assert_eq!(
            comment_line(";\n"),
            Ok(("\n", "")),
            "The comment parser should accept empty comments"
        );
        assert_eq!(
            comment_line(";asdf\r\nabc"),
            Ok(("\r\nabc", "asdf")),
            "The comment parser should accept carriage return + newline style \
             line endings"
        );
        assert_eq!(
            comment_line("; asdf\n"),
            Ok(("\n", " asdf")),
            "The comment parser shouldn't consume the line ending after a \
             commend"
        );
        assert_eq!(
            comment_line(";\r\n"),
            Ok(("\r\n", "")),
            "The comment parser should parse an empty comment with a carriage \
             return + newline style line ending"
        );
    }

    #[test]
    fn test_comment_fail() {
        assert!(
            comment_line("1234\n").is_err(),
            "The comment parser should fail if no semicolon is provided"
        );
        assert!(
            comment_line("").is_err(),
            "The comment parser should fail if no input is provided"
        );
    }

    #[test]
    fn test_org() {
        assert_eq!(
            org_line("ORG 1234\nabc"),
            Ok(("\nabc", 1234)),
            "The org parser should return the content of the org excluding \
             the \"ORG\" without consuming the line ending"
        );
        assert!(
            org_line("ORG\n").is_err(),
            "The org parser should fail if no number is provided"
        );
        assert_eq!(org_line("ORG 1234\r\nabc"), Ok(("\r\nabc", 1234)));

        assert_eq!(
            org_line("ORG1234"),
            Ok(("", 1234)),
            "The org parser should accept no space between ORG and the number"
        );
        assert_eq!(
            org_line("oRg 1234\r\n"),
            Ok(("\r\n", 1234)),
            "The org parser should be case insensitive"
        );
        assert_eq!(
            org_line("org +1234\r\n"),
            Ok(("\r\n", 1234)),
            "The org parser should accept positive numbers"
        );
        assert_eq!(
            org_line("org -1234\r\n"),
            Ok(("\r\n", -1234)),
            "The org parser should accept negative numbers"
        );
    }

    #[test]
    fn test_org_fail() {
        assert!(
            org_line("1234\n").is_err(),
            "The org parser should fail if no \"ORG\" is provided"
        );
        assert!(
            org_line("").is_err(),
            "The org parser should fail if no input is provided"
        );
        assert!(
            org_line("ORG\n").is_err(),
            "The org parser should fail if no number is provided"
        );
        assert!(
            org_line("ORG ++1234\n").is_err(),
            "The org parser should fail if the number has two leading signs"
        );
    }

    #[test]
    fn test_pin() {
        assert_eq!(
            pin_line("PIN 1234\nabc"),
            Ok(("\nabc", 1234)),
            "The pin parser should return the content of the pin excluding \
             the \"PIN\" without consuming the line ending"
        );
        assert!(
            pin_line("PIN\n").is_err(),
            "The pin parser should fail if no number is provided"
        );
        assert_eq!(
            pin_line("PIN 1234\r\nabc"),
            Ok(("\r\nabc", 1234)),
            "The pin parser should accept carriage return + newline style \
             line endings"
        );
    }

    #[test]
    fn test_pin_fail() {
        assert!(
            pin_line("1234\n").is_err(),
            "The pin parser should fail if no \"PIN\" is provided"
        );
        assert!(
            pin_line("").is_err(),
            "The pin parser should fail if no input is provided"
        );
        assert!(
            pin_line("PIN\n").is_err(),
            "The pin parser should fail if no number is provided"
        );
    }

    #[test]
    fn test_end() {
        assert_eq!(
            end_line("END 1234\nabc"),
            Ok(("\nabc", Some(1234))),
            "The end parser should return the content of the end excluding \
             the \"END\" without consuming the line ending"
        );
        assert_eq!(
            end_line("END\n"),
            Ok(("\n", None)),
            "The end parser should accept empty ends"
        );
        assert_eq!(end_line("END 1234\r\nabc"), Ok(("\r\nabc", Some(1234))));
    }

    #[test]
    fn test_end_fail() {
        assert!(
            end_line("1234\n").is_err(),
            "The end parser should fail if no \"END\" is provided"
        );
        assert!(
            end_line("").is_err(),
            "The end parser should fail if no input is provided"
        );
    }
}

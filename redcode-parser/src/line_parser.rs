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
        number,
        tag(","),
        space0,
        addr_mode,
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
                a_field,
                _, // ","
                _, // space0
                b_addr_mode,
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
        number,
        tag(","),
        space0,
        addr_mode,
        number,
    ))(input);
    match tuple_instruction {
        Ok((
            leftover,
            (
                _,
                opcode,
                _, // space0
                a_addr_mode,
                a_field,
                _, // ","
                _, // space0
                b_addr_mode,
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
    use super::*;
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
            "The comment parser should accept carrage return + newline style \
             line endings"
        );
        assert_eq!(
            comment_line("; asdf\n"),
            Ok(("\n", " asdf")),
            "The comment parser shouldn't consume the line ending after a \
             commend"
        );
    }
}

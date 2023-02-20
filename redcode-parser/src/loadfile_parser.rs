use std::error::Error;

use nom::{
    branch::alt,
    combinator::{eof, map},
    error::VerboseError,
    sequence::terminated,
    sequence::pair,
    Err, Finish, IResult,
};
use redcode::{RelaxedCompleteInstruction, RelaxedWarrior};

use crate::{line_parser::*, loadfile_primitives::eol};

pub fn parse(
    warrior: &str,
    omit_modifier: bool,
) -> Result<RelaxedWarrior, Err<VerboseError<&str>>> {
    let mut input = warrior;
    let mut instructions = vec![];
    let mut start = None;
    let mut pin = None;

    while !input.is_empty() {
        match parse_line(input, omit_modifier) {
            Ok((leftover, e)) => {
                input = leftover;
                match e {
                    LineContent::Empty() => {}
                    LineContent::Comment(_) => {}
                    LineContent::Pin(val) => pin = Some(val),
                    LineContent::Instruction(instr) => instructions.push(instr),
                    LineContent::Org(e) => start = Some(e),
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
    omit_modifier: bool,
) -> Result<RelaxedCompleteInstruction, Box<dyn Error + 'a>> {
    match parse_line(line, omit_modifier).finish() {
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
    Org(i64),
    End(Option<i64>),
    Pin(i64),
}

fn parse_line(
    input: &str,
    omit_modifier: bool,
) -> IResult<&str, LineContent, VerboseError<&str>> {
    let parse_instr = match omit_modifier {
        true => instr_88_line,
        false => instr_94_line,
    };

    // Parse the content from an eol or eof terminated segment of input
    // If terminated by EOF, we return the content, and the next invocation
    // will not match any body_content parsers, but will match an end_content_parser
    let body_content_parser = alt((
        map(parse_instr, LineContent::Instruction),
        map(comment_line, LineContent::Comment),
        map(org_line, LineContent::Org),
        map(pin_line, LineContent::Pin),
        map(end_line, |_| LineContent::Empty())
    ));
    let body_content_parser = terminated(body_content_parser, alt((eof, eol)));

    // Parse the various situations that terminate loadfile parsing
    let end_content_parser =  alt((
        // an "END" tag, regardless of how it is terminated
        map(end_line, LineContent::End),
        // an eof, optionally preceded by some whitespace
        map(pair(empty_line, eof), |_| LineContent::End(None))
    ));

    // end_content_parser *MUST* be checked before body content parser or else an empty line terminated by eof
    // will be interpreted as a body content line
    alt((end_content_parser, body_content_parser))(input)
}

use std::error::Error;

use nom::{
    branch::alt,
    combinator::{eof, map},
    error::VerboseError,
    sequence::{pair, terminated},
    Err, Finish, IResult,
};
use redcode::{RelaxedCompleteInstruction, RelaxedWarrior};

use crate::{
    line_parser::{
        comment_line, empty_line, end_line, instr_88_line, instr_94_line,
        org_line, pin_line,
    },
    loadfile_primitives::eol,
};

/// Parse a loadfile formatted warrior.  If `omit_modifier` is set, modifiers
/// must not be present, in the style of the '88 loadfile format.
///
/// Parses until an END statement is encountered, or the end of the input is
/// reached.
/// 
/// # Errors
/// 
/// Returns an error containing the source of the parsing issue and the
/// unprocessed input in the event that parsing does not terminate due to end of input
/// or an `END` statement, and the remaining content doesn't match any part of the redcode grammar.  
pub fn parse(
    warrior: &str,
    omit_modifier: bool,
) -> Result<RelaxedWarrior, Err<VerboseError<&str>>> {
    let mut input = warrior;
    let mut instructions = vec![];
    let mut start = None;
    let mut pin = None;

    loop {
        match parse_line(input, omit_modifier) {
            Ok((leftover, e)) => {
                input = leftover;
                match e {
                    LineContent::Empty() | LineContent::Comment(_) => {}
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
}

// Parse an instruction from the input string provided.  If `omit_modifier` is
// set, modifiers
/// must not be present, in the style of the '88 loadfile format.
///
/// The instruction text must be on the first line of the input, though it may
/// or may not be terminated by a newline.  Additional data past the first line
/// of the input is not parsed, and does not need to be valid.
/// 
/// # Errors
/// 
/// Returns an error with diagnostics and information about the consumed input in
/// the event that the first line of the input does not contain a valid Redcode instruction.  
pub fn parse_instr<'a>(
    line: &'a str,
    omit_modifier: bool,
) -> Result<RelaxedCompleteInstruction, Box<dyn Error + 'a>> {
    // TODO(jespy) try to get rid of the extra error type and only return a VerboseError
    // directly call the right instruction line parser.  This will be a semver breaking
    // change.  
    match parse_line(line, omit_modifier).finish() {
        Ok((_, LineContent::Instruction(instr))) => Ok(instr),
        Ok((_, _content)) => Err("Parsed the line not as an instruction but \
                                  as something else"
            .into()),
        Err(e) => Err(Box::new(e)),
    }
}

/// A container for the parsed contents a bit of the input, either terminated by
/// an EOL, an EOF, or an END line (which itself may be terminated by EOF or
/// EOL)
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum LineContent<'a> {
    /// Contains the text parsed from a comment
    Comment(&'a str),
    /// Contains a parsed instruction from the input
    Instruction(RelaxedCompleteInstruction),
    /// Represents a line that was parsed but contained only whitespace
    Empty(),
    /// Contains an ORG pseudoop.  If multiple ORG statements exist, the last
    /// statement takes effect.
    Org(i64),
    /// Indicates the end of loadfile parsing, optionally with the END argument
    /// indicating the start position.  If ORG and END statements both identify
    /// a starting position, the last one to be parsed from the file takes
    /// effect.
    End(Option<i64>),
    /// Contains a PIN specified by the author of the warrior
    Pin(i64),
}

/// Parses the content of a line.  If the result is `LineContent::End`, no
/// further calls to `parse_line` should be made.
fn parse_line(
    input: &str,
    omit_modifier: bool,
) -> IResult<&str, LineContent, VerboseError<&str>> {
    let parse_instr = if omit_modifier {instr_88_line} else {instr_94_line}; 

    // Parse the content from an eol or eof terminated segment of input
    // If terminated by EOF, we return the content, and the next invocation
    // will not match any body_content parsers, but will match an
    // end_content_parser
    let body_content_parser = alt((
        map(parse_instr, LineContent::Instruction),
        map(comment_line, LineContent::Comment),
        map(org_line, LineContent::Org),
        map(pin_line, LineContent::Pin),
        map(end_line, |_| LineContent::Empty()),
    ));
    let body_content_parser = terminated(body_content_parser, alt((eof, eol)));

    // Parse the various situations that terminate loadfile parsing
    let end_content_parser = alt((
        // an "END" tag, regardless of how it is terminated
        map(end_line, LineContent::End),
        // an eof, optionally preceded by some whitespace
        map(pair(empty_line, eof), |_| LineContent::End(None)),
    ));

    // end_content_parser *MUST* be checked before body content parser or else
    // an empty line terminated by eof will be interpreted as a body content
    // line
    alt((end_content_parser, body_content_parser))(input)
}

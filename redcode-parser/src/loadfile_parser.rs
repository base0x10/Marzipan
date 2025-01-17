use nom::{
    branch::alt,
    combinator::{eof, map},
    error::VerboseError,
    sequence::{pair, terminated},
    Err, IResult,
};
use redcode::{RelaxedCompleteInstruction, RelaxedWarrior};

use crate::{
    line_parser::{
        comment_line, empty_line, end_line, instr_88_line, instr_94_line,
        org_line, pin_line,
    },
    loadfile_primitives::eol,
};

/// Configures parser behavior.
///
/// The default options parse '94 loadfiles, and are the most permissive.  It
/// allows empty warriors, and will not parse or validate any data following
/// an `END` statement
#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
pub struct ParseOptions {
    /// Omit modifiers in the style of ICWS 88
    omit_modifiers: bool,
    /// Require parser to produce at least one instruction
    disallow_empty_warrior: bool,
    /// Require parser to consume entire input
    must_consume_all: bool,
}

impl ParseOptions {
    /// Default options
    ///
    /// Identical to [`ParseOptions::Default`] with permissive options which
    /// parse '94 loadfiles.
    pub const DEFAULT_OPTIONS: Self = Self {
        omit_modifiers: false,
        disallow_empty_warrior: false,
        must_consume_all: false,
    };
    /// Options for parsing an '88 loadfile.
    ///
    /// ICWS '88 loadfile must not have any modifiers.  Produces a warrior with
    /// modifiers set to the default values specified by the ICWS 94
    /// specification for translating an '88 loadfile into a compatible '94
    /// loadfile.
    pub const ICWS_88_OPTIONS: Self = Self {
        omit_modifiers: true,
        disallow_empty_warrior: false,
        must_consume_all: false,
    };
    /// Default permissive options that parse '94 loadfiles.
    ///
    /// This is an alias for `DEFAULT_OPTIONS`.
    pub const ICWS_94_OPTIONS: Self = Self::DEFAULT_OPTIONS;
    /// The most strict set of parsing options.
    ///
    /// May catch programming errors by rejecting inputs with data following the
    /// end of the warrior, or where the resulting warrior would contain no
    /// instructions.
    pub const STRICT_OPTIONS: Self = Self {
        omit_modifiers: false,
        disallow_empty_warrior: true,
        must_consume_all: true,
    };

    /// Require that modifiers be omitted from instructions, and use the default
    /// modifiers specified by ICWS 94 for compatibility with ICWS 88 warriors.
    #[must_use]
    pub const fn require_omitted_modifiers(mut self) -> Self {
        self.omit_modifiers = true;
        self
    }

    /// Prevents parsing warriors that contain no instructions.
    ///
    /// The default behavior accepts an empty input or an input with nothing
    /// but comments, pseudo-ops, and whitespace.  With this option enabled,
    /// the parser will return an error when the warrior output would contain
    /// no instructions.
    #[must_use]
    pub const fn disallow_empty_warrior(mut self) -> Self {
        self.disallow_empty_warrior = true;
        self
    }

    /// Require that the entire input be consumed by the parser.
    ///
    /// The default behavior is to parse until an `END` statement is
    /// encountered, or the end of the input is reached.  If this is
    /// enabled, the parser will return an error if there is any
    /// non-whitespace content remaining after the line containing the first
    /// END statement.
    #[must_use]
    pub const fn must_consume_all(mut self) -> Self {
        self.must_consume_all = true;
        self
    }
}

/// Parse a loadfile formatted warrior from the input.
///
/// [`ParseOptions`] can be used to modify the behavior of the parser. The
/// default behavior is to:
///     * Parse '94 style loadfile instructions
///     * Allow empty warriors
///     * Allow any unparsed data following the end of the warrior
///
/// # Errors
///
/// Returns an error containing the source of the parsing issue and the
/// unprocessed input if the content of the input doesn't match the redcode
/// grammar.
///
/// Also returns an error if any conditions specified by [`ParseOptions`] are
/// violated.
pub fn parse(
    warrior: &str,
    options: ParseOptions,
) -> Result<RelaxedWarrior, Err<VerboseError<&str>>> {
    let mut input = warrior;
    let mut instructions = vec![];
    let mut start = None;
    let mut pin = None;

    loop {
        match parse_line(input, options.omit_modifiers) {
            Ok((leftover, e)) => {
                input = leftover;
                match e {
                    LineContent::Empty() | LineContent::Comment(_) => {}
                    LineContent::Pin(val) => pin = Some(val),
                    LineContent::Instruction(instr) => instructions.push(instr),
                    LineContent::Org(e) => start = Some(e),
                    LineContent::End(Some(e)) => {
                        start = Some(e);
                        break;
                    }
                    LineContent::End(None) => break,
                }
            }
            Err(e) => return Err(e),
        }
    }
    if options.must_consume_all {
        // must_consume_all rejects warriors with any trailing data other that
        // whitespace including comments or additional
        // pseudo-instructions
        if !input.trim().is_empty() {
            return Err(Err::Error(VerboseError {
                errors: vec![(
                    input,
                    nom::error::VerboseErrorKind::Context(
                        "Expected end of input",
                    ),
                )],
            }));
        }
    }
    if options.disallow_empty_warrior && instructions.is_empty() {
        return Err(Err::Error(VerboseError {
            errors: vec![(
                input,
                nom::error::VerboseErrorKind::Context(
                    "Expected at least one instruction",
                ),
            )],
        }));
    }
    Ok(RelaxedWarrior {
        code: instructions,
        start: start.unwrap_or(0),
        pin,
    })
}

/// Parse up to one instruction from the input.
///
/// The instruction does not need to be on the first line of the input, or be
/// terminated by a newline character.  However, the first line containing
/// a redcode statement must be a valid instruction.  The first instruction
/// may not be preceded by a comment line, a PIN pseudoop, or an ORG pseudoop,
/// or an END pseudoop.
///
/// This function uses [`ParseOptions`] in the same manor as [`parse`].  For
/// example, it will determine if the entire input must be parsed, or if the
/// instruction should be parsed with modifiers omitted as in '88 style redcode.
/// [`ParseOptions::disallow_empty_warrior`] has no effect.  
///
/// # Errors
///
/// Returns an error containing the source of the parsing issue and the
/// unprocessed input if the content of the input does not contain a valid
/// instruction, or if the first non-blank line couldn't be parsed as an
/// instruction.
///
/// Also returns an error if any conditions specified by [`ParseOptions`] are
/// violated.
pub fn parse_instr(
    input: &str,
    options: ParseOptions,
) -> Result<RelaxedCompleteInstruction, Err<VerboseError<&str>>> {
    let mut input = input;

    loop {
        match parse_line(input, options.omit_modifiers) {
            Ok((leftovers, LineContent::Empty())) => {
                input = leftovers;
            }
            Ok((_, LineContent::End(_))) => {
                return Err(Err::Error(VerboseError {
                    errors: vec![(
                        input,
                        nom::error::VerboseErrorKind::Context(
                            "Unexpected end of input before instruction",
                        ),
                    )],
                }));
            }
            Ok((
                _,
                LineContent::Comment(_)
                | LineContent::Pin(_)
                | LineContent::Org(_),
            )) => {
                return Err(Err::Error(VerboseError {
                    errors: vec![(
                        input,
                        nom::error::VerboseErrorKind::Context(
                            "Unexpected redcode statement before instruction",
                        ),
                    )],
                }));
            }
            Ok((leftovers, LineContent::Instruction(parsed_instruction))) => {
                if options.must_consume_all && !leftovers.trim().is_empty() {
                    return Err(Err::Error(VerboseError {
                        errors: vec![(
                            leftovers,
                            nom::error::VerboseErrorKind::Context(
                                "Unexpected content following an instruction, \
                                 disallowed by ParseOptions",
                            ),
                        )],
                    }));
                }
                return Ok(parsed_instruction);
            }
            Err(e) => return Err(e),
        }
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
    let parse_instr = if omit_modifier {
        instr_88_line
    } else {
        instr_94_line
    };

    // Parse the content from an eol or eof terminated segment of input
    // If terminated by EOF, we return the content, and the next invocation
    // will not match any body_content parsers, but will match an
    // end_content_parser
    let body_content_parser = alt((
        map(parse_instr, LineContent::Instruction),
        map(comment_line, LineContent::Comment),
        map(org_line, LineContent::Org),
        map(pin_line, LineContent::Pin),
        map(empty_line, |_| LineContent::Empty()),
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

#[cfg(test)]
mod tests {
    use redcode::*;

    use super::*;

    #[test]
    fn parse_simple_instruction() {
        let input = "DAT.AB #1, $2";
        let parsed = parse_instr(input, ParseOptions::default());
        assert_eq!(
            parsed,
            Ok(RelaxedCompleteInstruction {
                instr: Instruction {
                    opcode: Opcode::Dat,
                    modifier: Modifier::AB,
                    a_addr_mode: AddrMode::Immediate,
                    b_addr_mode: AddrMode::Direct,
                },
                a_field: 1,
                b_field: 2,
            })
        );
    }

    #[test]
    fn parse_88_style_instruction() {
        let input = "DAT #1, $2";
        let parsed = parse_instr(input, ParseOptions::ICWS_88_OPTIONS);
        assert_eq!(
            parsed,
            Ok(RelaxedCompleteInstruction {
                instr: Instruction {
                    opcode: Opcode::Dat,
                    modifier: default_modifiers(
                        Opcode::Dat,
                        AddrMode::Immediate,
                        AddrMode::Direct
                    ),
                    a_addr_mode: AddrMode::Immediate,
                    b_addr_mode: AddrMode::Direct,
                },
                a_field: 1,
                b_field: 2,
            })
        );
    }

    #[test]
    fn parse_all_94_instrs() {
        for instruction in test_utils::all_instructions() {
            let expected = RelaxedCompleteInstruction {
                instr: instruction.clone(),
                a_field: 1234,
                b_field: 1234,
            };
            let input = expected.normalize(8000u32).to_string();
            let parsed = parse_instr(&input, ParseOptions::default());

            assert_eq!(
                parsed,
                Ok(expected),
                "Failed to parse instruction: {input}"
            );
        }
    }

    #[test]
    fn parse_instr_with_empty_line() {
        let input = "\n\nDAT.AB #1, $2";
        let parsed = parse_instr(input, ParseOptions::default());
        assert_eq!(
            parsed,
            Ok(RelaxedCompleteInstruction {
                instr: Instruction {
                    opcode: Opcode::Dat,
                    modifier: Modifier::AB,
                    a_addr_mode: AddrMode::Immediate,
                    b_addr_mode: AddrMode::Direct,
                },
                a_field: 1,
                b_field: 2,
            })
        );
    }

    #[test]
    fn parse_instruction_invalid_inputs() {
        let invalid_inputs = vec![
            ("", "input with no instruction should not be parsed"),
            ("\n\n", "input with no instruction should not be parsed"),
            (
                "; comment text\nDAT.AB, #2, #4\n",
                "instruction shouldn't be parsed if preceded by comment",
            ),
            (
                "ORG 0\nDAT.AB #0, #0",
                "instruction shouldn't be parsed if preceded by a pseudo-op",
            ),
            (
                "PIN 0\nDAT.AB #0, #0",
                "instruction shouldn't be parsed if preceded by a pseudo-op",
            ),
            (
                "END 0\nDAT.AB #0, #0",
                "instruction shouldn't be parsed if preceded by a pseudo-op",
            ),
        ];

        for (input, msg) in invalid_inputs {
            let parsed = parse_instr(input, ParseOptions::default());
            assert!(
                parsed.is_err(),
                "Incorrectly parsed {input} successfully .  {msg}"
            );
        }
    }

    #[test]
    fn parse_instr_with_trailing_data() {
        let input = "DAT.AB #1, $2\n ; HERE IS SOME TRAILING COMMENT TEXT";
        let parsed_allowing_trailing_data =
            parse_instr(input, ParseOptions::default());
        let parsed_disallowing_trailing_data =
            parse_instr(input, ParseOptions::default().must_consume_all());
        assert!(
            parsed_allowing_trailing_data.is_ok(),
            "Failed to parse instruction with trailing data {input}, reason: \
             {parsed_allowing_trailing_data:?}"
        );
        assert!(
            parsed_disallowing_trailing_data.is_err(),
            "Incorrectly parsed an instruction with trailing data \
             successfully {input}"
        );
    }

    #[test]
    fn parse_simple_warrior() {
        let warrior = "DAT.AB #1, $2
                          SLT.F >3, }4
                          END
                          DAT.AB #5, #6"; // This line should be ignored
        let parsed = parse(warrior, ParseOptions::default());
        assert_eq!(
            parsed,
            Ok(RelaxedWarrior {
                code: vec![
                    RelaxedCompleteInstruction {
                        instr: Instruction {
                            opcode: Opcode::Dat,
                            modifier: Modifier::AB,
                            a_addr_mode: AddrMode::Immediate,
                            b_addr_mode: AddrMode::Direct,
                        },
                        a_field: 1,
                        b_field: 2,
                    },
                    RelaxedCompleteInstruction {
                        instr: Instruction {
                            opcode: Opcode::Slt,
                            modifier: Modifier::F,
                            a_addr_mode: AddrMode::PostincB,
                            b_addr_mode: AddrMode::PostincA,
                        },
                        a_field: 3,
                        b_field: 4,
                    },
                ],
                start: 0,
                pin: None,
            })
        );
    }

    #[test]
    fn parse_warriors_with_trailing_newline() {
        let warriors = vec![
            "DAT.AB #1, <3\n",
            "DAT.AB #1, <3\n\n\r\n",
            "DAT.AB #1, <3\n\n; comment asdf\n",
        ];
        for input in warriors {
            let parsed = parse(input, ParseOptions::default());
            assert_eq!(
                parsed,
                Ok(RelaxedWarrior {
                    code: vec![RelaxedCompleteInstruction {
                        instr: Instruction {
                            opcode: Opcode::Dat,
                            modifier: Modifier::AB,
                            a_addr_mode: AddrMode::Immediate,
                            b_addr_mode: AddrMode::PredecB,
                        },
                        a_field: 1,
                        b_field: 3,
                    }],
                    start: 0,
                    pin: None,
                }),
                "Failed to parse warrior: {input}"
            );
        }
    }

    #[test]
    fn parse_warriors_without_trailing_newlines() {
        let warriors = vec![
            (
                "DAT.AB #1, <3",
                "warrior ending with an instruction not terminated by a \
                 newline",
            ),
            (
                "DAT.AB #1, <3\n; comment text",
                "warrior ending with a comment not terminated by a newline",
            ),
            (
                "DAT.AB #1, <3\nEND",
                "warrior ending with an END not terminated by a newline",
            ),
            (
                "DAT.AB #1, <3\nEND 123",
                "warrior ending with END with an argument not terminated by a \
                 newline",
            ),
            (
                "DAT.AB #1, <3\nORG 123",
                "warrior ending with an ORG not terminated by a newline",
            ),
            (
                "DAT.AB #1, <3\nPIN 123",
                "warrior ending with a PIN not terminated by a newline",
            ),
            (
                "DAT.AB #1, <3\n    ",
                "warrior ending with whitespace not terminated by a newline",
            ),
        ];
        for (input, desc) in warriors {
            let parsed = parse(input, ParseOptions::default());
            assert!(
                parsed.is_ok(),
                "failed to successfully parse a {desc}\ninput: {input}"
            );
        }
    }

    #[test]
    fn parse_empty_warrior() {
        let empty_warriors = vec![
            "",
            "end\n",
            "END\nDAT.AB #1, $2",
            "ORG 123\n; a comment\nEND\n",
        ];
        for warrior in empty_warriors.iter() {
            let parsed_allowing_empty = parse(warrior, ParseOptions::default());
            assert!(
                parsed_allowing_empty.is_ok(),
                "Failed to parse \"{}\" with settings that allow empty \
                 warriors",
                warrior
            );
        }
        for warrior in empty_warriors.iter() {
            let parsed_disallowing_empty = parse(
                warrior,
                ParseOptions::default().disallow_empty_warrior(),
            );
            assert!(
                parsed_disallowing_empty.is_err(),
                "Incorrectly parsed \"{}\" successfully with settings that \
                 disallow empty warriors",
                warrior
            );
        }
    }

    #[test]
    fn parse_warriors_with_trailing_data() {
        let warriors_with_trailing_data = vec![
            "DAT.AB #1, $2
                          SLT.F >3, }4
                          END
                          DAT.AB #5, #6",
            "END\nDAT.AB #1, $2",
            "END\n; A comment after the end",
        ];

        for warrior in warriors_with_trailing_data.iter() {
            let parsed_allowing_trailing_data =
                parse(warrior, ParseOptions::default());
            assert!(
                parsed_allowing_trailing_data.is_ok(),
                "Failed to parse \"{}\" with settings that should allow empty \
                 warriors",
                warrior
            );
        }

        for warrior in warriors_with_trailing_data.iter() {
            let parsed_disallowing_trailing_data =
                parse(warrior, ParseOptions::default().must_consume_all());
            assert!(
                parsed_disallowing_trailing_data.is_err(),
                "Incorrectly parsed \"{}\" successfully with settings that \
                 should disallow trailing data",
                warrior
            );
        }
    }

    #[test]
    fn parse_88_style_warriors() {
        let warriors = vec![
            (
                "ADD #2, $3",
                RelaxedWarrior {
                    code: vec![RelaxedCompleteInstruction {
                        instr: Instruction {
                            opcode: Opcode::Add,
                            modifier: default_modifiers(
                                Opcode::Add,
                                AddrMode::Immediate,
                                AddrMode::Direct,
                            ),
                            a_addr_mode: AddrMode::Immediate,
                            b_addr_mode: AddrMode::Direct,
                        },
                        a_field: 2,
                        b_field: 3,
                    }],
                    start: 0,
                    pin: None,
                },
            ),
            (
                "DAT #1, $2
                SLT }6, }-18
                ; Here's a comment
                PIN 13
                ORG 177
                JMZ $-1, $-1
                END",
                RelaxedWarrior {
                    code: vec![
                        RelaxedCompleteInstruction {
                            instr: Instruction {
                                opcode: Opcode::Dat,
                                modifier: default_modifiers(
                                    Opcode::Dat,
                                    AddrMode::Immediate,
                                    AddrMode::Direct,
                                ),
                                a_addr_mode: AddrMode::Immediate,
                                b_addr_mode: AddrMode::Direct,
                            },
                            a_field: 1,
                            b_field: 2,
                        },
                        RelaxedCompleteInstruction {
                            instr: Instruction {
                                opcode: Opcode::Slt,
                                modifier: default_modifiers(
                                    Opcode::Slt,
                                    AddrMode::PostincA,
                                    AddrMode::PostincA,
                                ),
                                a_addr_mode: AddrMode::PostincA,
                                b_addr_mode: AddrMode::PostincA,
                            },
                            a_field: 6,
                            b_field: -18,
                        },
                        RelaxedCompleteInstruction {
                            instr: Instruction {
                                opcode: Opcode::Jmz,
                                modifier: default_modifiers(
                                    Opcode::Jmz,
                                    AddrMode::Direct,
                                    AddrMode::Direct,
                                ),
                                a_addr_mode: AddrMode::Direct,
                                b_addr_mode: AddrMode::Direct,
                            },
                            a_field: -1,
                            b_field: -1,
                        },
                    ],
                    start: 177,
                    pin: Some(13),
                },
            ),
            (
                "",
                RelaxedWarrior {
                    code: vec![],
                    start: 0,
                    pin: None,
                },
            ),
        ];

        for (warrior, expected) in warriors.iter() {
            let parsed = parse(warrior, ParseOptions::ICWS_88_OPTIONS);
            assert!(parsed.is_ok());
            assert_eq!(parsed.unwrap(), *expected);
        }
    }

    #[test]
    fn parse_warrior_with_missing_newline() {
        let warrior = "DAT.AB #1, #2DAT.F #3, #4";
        let parsed = parse(warrior, ParseOptions::default());
        assert!(
            parsed.is_err(),
            "warrior parsing should require instructions to be separated by \
             newlines"
        );
    }

    #[test]
    fn parse_warrior_with_multiple_pins() {
        let warrior = "DAT.AB #1, #2
                          DAT.F #3, #4
                          PIN 1
                          PIN 2
                          END";
        let parsed = parse(warrior, ParseOptions::default());
        assert!(parsed.is_ok(), "warrior parsing should allow multiple pins");
        assert_eq!(
            parsed.unwrap().pin.unwrap(),
            2,
            "warrior parsing should use the last pin specified"
        );
    }

    #[test]
    fn parse_warriors_with_multiple_orgs() {
        let correct_start = 2;
        let warriors = vec![
            (
                "DAT.AB #1, #2
                          DAT.F #3, #4
                          ORG 1
                          ORG 2
                          END",
                "A warrior with two ORG statements should use the lase one",
            ),
            (
                "DAT.AB #1, #2
                          DAT.F #3, #4
                          ORG 2
                          END
                          ORG 1",
                "only ORG statements prior to END should be considered",
            ),
            (
                "DAT.AB #1, #2
                          DAT.F #3, #4
                          ORG 1
                          ORG -5
                          END 2
                          ORG 1",
                "The first END statement, if contains a start position, \
                 should be used",
            ),
        ];

        for (warrior_test, desc) in warriors {
            let parsed = parse(warrior_test, ParseOptions::default());
            assert!(parsed.is_ok(), "failed to parse warrior for case: {desc}");
            assert_eq!(parsed.unwrap().start, correct_start, "{desc}");
        }
    }
}

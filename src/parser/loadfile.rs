use crate::redcode::{AddrMode, Instruction, Modifier, Opcode, RedAddr};
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, tag_no_case},
    error::VerboseError,
    sequence::{delimited, preceded, pair, separated_pair, terminated},
    IResult,
    character::complete::{
        space1,
        line_ending
    }
};

use super::atomics;

/// Formal grammer of a redcode loadfile
///     Loadfiles are rigid, and do not permit omitted fields, extra lines, etc.  
///     Adapted from https://corewar.co.uk/standards/icws94.htm with additions from http://www.koth.org/info/pmars-redcode-94.txt
///         
/// Grammer follows relatively standard rules: by default replacement is single substitution.
///  * ^SYMBOL means any symbol except for SYMBOL.  SYMBOL* means 0 or more occurrences of symbol
///  * A | B means either A or B.  
///  * A B means the symbol A followed by the symbol B.
///  * (A B ...) means the symbols A B ... grouped so that ^ or * operate on the result
/// ```
/// // loadfile:
/// //     list
/// //
/// // list:
/// //     line | line list
/// //
/// // line:
/// //     comment | instruction
/// //
/// // comment:
/// //     ; text EOL
/// //
/// // instruction:
/// //     opcode . modifier whitespace mode number , whitespace mode number (whitespace text)* EOL
/// //
/// // opcode:
/// //     DAT | MOV | ADD | SUB | MUL | DIV | MOD | JMP | JMZ | JMN | DJN | SPL | SLT | CMP | SEQ |
/// //     SNE | NOP | LDP | STP
/// //
/// // modifier:
/// //     A | B | AB | BA | F | X | I
/// //
/// // whitespace:
/// //     SPACE whitespace | HORIZONTAL_TAB whitespace | SPACE | HORIZONTAL_TAB
/// //
/// // mode:
/// //     # | $ | @ | < | > | * | { | }
/// //
/// // number:
/// //     [0-9] [0-9]*
/// //
/// // text
/// //     ^EOL
/// ```

fn parse_opmodifier(input: &str) -> IResult<&str, (Opcode, Modifier), VerboseError<&str>> {
    separated_pair(atomics::parse_opcode, tag("."), atomics::parse_modifier)(input)
}

fn parse_field(input: &str) -> IResult<&str, (AddrMode, i32), VerboseError<&str>> {
    pair(atomics::parse_mode, atomics::parse_num)(input)
}

fn parse_fields(
    input: &str,
) -> IResult<&str, ((AddrMode, i32), (AddrMode, i32)), VerboseError<&str>> {
    separated_pair(parse_field, pair(tag(","), space1), parse_field)(input)
}

pub fn parse_instr(input: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let tup_instr = separated_pair(parse_opmodifier, space1, parse_fields)(input);
    match tup_instr {
        Ok((leftover, ((opcode, modifier), ((a_addr_mode, a_value), (b_addr_mode, b_value))))) => {
            Ok((
                leftover,
                Instruction {
                    opcode,
                    modifier,
                    a_addr_mode,
                    b_addr_mode,
                    a_value: RedAddr::from_i32(a_value),
                    b_value: RedAddr::from_i32(b_value),
                },
            ))
        }
        Err(e) => Err(e),
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum LineContent<'a> {
    Comment(&'a str),
    Instruction(Instruction),
    Empty(),
    Org(i32)
}

// empty lines are parsed as comments with no text
fn parse_comment_line(input: &str) -> IResult<&str, LineContent, VerboseError<&str>> {
    match preceded(tag(";"), is_not("\n"))(input) {
        Ok((leftover, captured)) => Ok((leftover, LineContent::Comment(captured))),
        Err(err) => Err(err),
    }
}

fn parse_instr_line(input: &str) -> IResult<&str, LineContent, VerboseError<&str>> {
    match terminated(parse_instr, is_not(";\n"))(input) {
        Ok((leftover, captured)) => Ok((leftover, LineContent::Instruction(captured))),
        Err(err) => Err(err),
    }
}

fn parse_org_line(input: &str) -> IResult<&str, LineContent, VerboseError<&str>> {
    match terminated(separated_pair(tag_no_case("ORG"), space1, atomics::parse_num), is_not("\n;"))(input) {
        Ok((leftover, (_, captured))) => Ok((leftover, LineContent::Org(captured))),
        Err(err) => Err(err),
    }
}

fn parse_empty_line(input: &str) -> IResult<&str, LineContent, VerboseError<&str>> {
    match line_ending(input) {
        Ok((leftover, _)) => Ok((leftover, LineContent::Empty())),
        Err(err) => Err(err),
    }
}



pub fn parse_loadfile(input: &str) -> Result<(Vec<Instruction>, i32), (&str, Vec<Instruction>, Vec<&str>)> {
    let mut input = input;
    let mut comments = Vec::new();
    let mut instructions = Vec::new();
    let mut org = 0;
    while !input.is_empty() {
        let res = alt((parse_comment_line, parse_instr_line, parse_empty_line, parse_org_line))(input);
        println!("{:?}", res);
        match res {
            Ok((leftover, LineContent::Instruction(instr))) => {
                input = leftover;
                instructions.push(instr);
            }
            Ok((leftover, LineContent::Comment(comment))) => {
                input = leftover;
                comments.push(comment);
            }
            Ok((leftover, LineContent::Empty())) => {
                input = leftover;
            }
            Ok((leftover, LineContent::Org(val))) => {
                input = leftover;
                org = val;
            }
            // TODO: convert this to a useful trace that can be passed to end user,
            // for now the comments and instructions which have so far been parsed are good enough
            Err(_e) => return Err(("Issue Parsing loadfile", instructions, comments)),
        }
    }
    Ok((instructions, org))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_loadfile() {
        let loadfile_data = "add.AB #-16, #-3\nadd.AB #-16, #-3\nadd.AB #-16, #-3\n";
        let res = parse_loadfile(loadfile_data);
        println!("Result of parser is {:?}", res);
        assert_eq!(true, res.is_ok());
        assert_eq!(res.unwrap().0.len(), 3);
    }

    #[test]
    fn test_comment() {
        assert_eq!(
            parse_comment_line(";1234\nabc"),
            Ok(("abc", LineContent::Comment("1234")))
        );
    }
    #[test]
    fn test_instr_parse() {
        let to_parse = "DAT.A #100, #100";
        assert_eq!(
            parse_instr(to_parse),
            Ok((
                "",
                Instruction {
                    opcode: Opcode::Dat,
                    modifier: Modifier::A,
                    a_addr_mode: AddrMode::Immediate,
                    b_addr_mode: AddrMode::Immediate,
                    a_value: RedAddr::new(100),
                    b_value: RedAddr::new(100)
                }
            ))
        );
    }
}

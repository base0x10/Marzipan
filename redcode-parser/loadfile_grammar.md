# Redcode Loadfile Grammar

The following formal grammar describes the language parsed by
[loadfile_parser.rs](src/loadfile_parser.rs).  The grammar is adapted from
pMARS's [REDCODE REFERENCE](http://www.koth.org/info/pmars-redcode-94.txt) and
the ICWS '94 [specification](https://corewar.co.uk/standards/icws94.htm).

The grammar provided is unambiguous, wildly verbose, and generally less helpful
than the two documents linked above.  It's a pure substitution grammar in BNF
formatting without any features or operators from BNF / EBNF / ABNF / TBNF 
/ ISO-EBNF.

The [loadfile_parser.rs](src/loadfile_parser.rs) also supports parsing a single
instruction, and parsing warriors or instructions that are required to omit
the modifier portion of the instruction.  The latter matches the ICWS '88
loadfile format, though it does not reject warriors using opcodes or modes
that are not present in ICWS '88.  I haven't provided separate grammars for
these.

## Notes

Marzipan is _slightly_ more permissive than required by the ICWS '94
specification.  Differences and ambiguities with the ICWS standard
and pMARS REDCODE REFERENCE are resolved below.

 * Opcodes and modes that are not present in the [ICWS '94
   drafts](https://corewar.co.uk/standards/icws94.htm) are implemented.
   * Opcodes:
     * `SEQ` - skip next instruction if A is equal to B (same as `CMP`)
     * `SNE` - skip next instruction if A is not equal to B
     * `NOP` - no operation
     * `LDP` - load P-space cell A into B
     * `STP` - store A into P-space cell B
   * Addressing Modes
     * `*` - Indirect using A-field
     * `{` - Predecrement indirect using A-field
     * `}` - Postincrement indirect using A-field
 * The encoding is UTF-8.  Loadfiles with invalid encodings are not
   rejected, rather they are not representable in rust's `&str`. 
 * Any text after a line with an `END` statement is not parsed.
 * All whitespace is optional with the exception of newline characters.
 * Blank lines are allowed.
 * The `END` Pseudo-opcode, which the ICWS specification does not explicitly
   allow in the loadfile format, is accepted with or without the optional start
   position field.
 * Any number of `ORG` statements along with at most one `END` statement at the
   end of a file are accepted.  The last `ORG` or `END` statement with an
   argument used, or 0 if none are present.
 * Multiple PIN statements are allowed, with only the last being used
 * Numbers have the same form for field values, PIN arguments,
   ORG optional arguments, and END optional arguments.
    * Must be representable as `i64` (twos complement 64 bit integers)
    * May (but are not required to) being with a unary '+' or '-'
    * May not contain any other characters besides an optional leading sign and
      the characters `0-9`

Comments are the only part of the grammar with the flexibility to contain
non-ascii characters.  They may contain any UTF-8 sequence terminated by a
newline.  I have found one warrior containing invalid UTF-8 though more likely
exist (CoreWar predates UTF-8 adoption by over a decade, and authors rightly
prefer to use character sets that can accurately encode their name).  Rust
will enforce this, but callers should take care to re-encode or replace invalid
UTF-8 sequences when reading possibly old warriors.

## Grammar

As noted above, numbers are limited `i64` values, and comments accept any UTF-8
sequence terminated by a newline.  All literals are case insensitive (e.g.
'dAt' or '.Ab' are valid).

```
loadfile    ::= list
                | list end

list        ::= line
                | line list

line        ::= eol
                | whitespace eol
                | statement eol
                | whitespace statement eol
                | statement whitespace eol
                | whitespace statement whitespace eol

statement   ::= comment
                | instruction
                | org

comment     ::= ';' text

instr       ::= opcode '.' modifier whitespace fields

fields      ::= field whitespace ',' whitespace field

field       ::= mode whitespace number

opcode      ::= 'DAT'
                | 'MOV'
                | 'ADD'
                | 'SUB'
                | 'MUL'
                | 'DIV'
                | 'MOD'
                | 'JMP'
                | 'JMZ'
                | 'JMN'
                | 'DJN'
                | 'SPL'
                | 'SLT'
                | 'CMP'
                | 'SEQ'
                | 'SNE'
                | 'NOP'
                | 'LDP'
                | 'STP'

org         ::= 'ORG'
                | 'ORG' whitespace number

end         ::= whitespace 'END'
                | 'END' whitespace number

modifier    ::= 'A'
                | 'B'
                | 'AB'
                | 'BA'
                | 'F'
                | 'X'
                | 'I'

whitespace  ::= ws_char
                | whitespace ws_char

ws_char     ::= ' '
                | '/t'

mode        ::= '#'
                | '$'
                | '@'
                | '<'
                | '>'
                | '*'
                | '{'
                | '}'

number      ::= num_chars
                | '+' num_chars
                | '-' num_chars

num_chars   ::= num_char
                | num_char num_chars

num_char    ::= '1'
                | '2'
                | '3'
                | '4'
                | '5'
                | '6'
                | '7'
                | '8'
                | '9'
                | '0'

eol         ::= '/n'
                | '/r/n'

text ::= /* Any UTF-8 sequence not containing eol */
```
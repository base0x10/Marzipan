use core::fmt;

use serde::{Deserialize, Serialize};

/// Fields hold values that are positive offsets from their own core address.
/// They are stored and used modulo `core_size`.
pub type FieldValue = u32;

/// The operand portion of an instruction.  
///
/// Supports operands from '88 and '94 ICWS standards plus pMARS extensions
#[derive(
    Copy,
    Clone,
    Debug,
    Eq,
    PartialEq,
    Hash,
    FromPrimitive,
    ToPrimitive,
    Serialize,
    Deserialize,
)]
pub enum Opcode {
    /// Remove the current task from a warrior's task queue
    Dat,

    /// Replace the B-target with the A-value and queue the next instruction
    Mov,

    /// Replace the B-target with the sum of the A/B values, and queue the next
    /// instruction
    Add,

    /// Replace the B-target with the B-value minus the A-value, and queue the
    /// next instruction
    Sub,

    /// Replace the B-target with the A-value times the B-value, and queue the
    /// next instruction
    Mul,

    /// Replace the B-target with the B-value divided by the A-value.  If part
    /// of the A-value is zero, the corresponding part of the B-target is
    /// unmodified.  The next instruction is queued only if no division by zero
    /// was attempted.
    Div,

    /// Replace the B-target with the remainder from the B-value divided by the
    /// A-value.  If part of the A-value is zero, the corresponding part of the
    /// B-target is unmodified.  The next instruction is queued only if no
    /// division by zero was attempted.
    Mod,

    /// Queues the the sum of the program counter and the A-pointer
    Jmp,

    /// Queues the sum of the program counter and the A-pointer if B-value is
    /// zero, and otherwise queues the next instruction.  
    Jmz,

    /// Queues the sum of the program counter and the A-pointer if any part of
    /// the B-value is not zero, and otherwise queues the next instruction.
    Jmn,

    /// Decrements the B-value and B-target, and then queues the sum of the
    /// program counter and the A-pointer if the decremented B-value is not
    /// zero, and otherwise queues the next instruction.
    Djn,

    /// Queues the next instruction, and then queues the sum of the program
    /// counter and the A-pointer.  If the queue is full, only the next
    /// instruction is queued.
    Spl,

    /// Compares the A-value to B-value.  If every part of the A-value is less
    /// than the corresponding part of the B-value, the instruction after next
    /// is queued, and otherwise the next instruction is queued.  
    Slt,

    /// Compares the A-value to B-value.  If every part is equal, the
    /// instruction after the next is queued, and otherwise the next
    /// instruction is queued.  
    Cmp,

    /// Compares the A-value to B-value.  If every part is equal, the
    /// instruction after next is queued, and otherwise the next instruction is
    /// queued.
    Seq,

    /// Compares the A-value to B-Value.  If any part of the A-value is not
    /// equal to the corresponding part of the B-value, the instruction after
    /// next is queued, and otherwise the next instruction is queued.  
    Sne,

    /// Queues the next instruction and does nothing else.
    Nop,

    /// Replace the B-target with a value loaded from PSPACE at an index
    /// specified by the A-value.  Queue the next instruction.
    Ldp,

    /// Replace a value in PSPACE at an index specified by the B-value with the
    /// A-value.  Queue the next instruction.  
    Stp,
}

#[allow(
    clippy::use_debug,
    reason = "Debug formatter used to get the opcode mnemonic from enum value"
)]
impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use Debug formatter to get the identifier of this variant
        write!(f, "{self:?}")
    }
}

/// The opcode modifier portion of a redcode instructions
///
/// Supports '88 and '94 ICWS standard modifiers
#[derive(
    Copy,
    Clone,
    Debug,
    Eq,
    PartialEq,
    Hash,
    FromPrimitive,
    ToPrimitive,
    Serialize,
    Deserialize,
)]
pub enum Modifier {
    /// Instruction execution proceeds with the A-value set to the A-number of
    /// the A-instruction and the B-value set to the A-number of the
    /// B-instruction.  A write to core replaces the A-number of the
    /// instruction pointed to by the B-pointer.
    A,

    /// Instruction execution proceeds with the A-value set to the B-number of
    /// the A-instruction and the B-value set to the B-number of the
    /// B-instruction.  A write to core replaces the B-number of the
    /// instruction pointed to by the B-pointer.
    B,

    /// Instruction execution proceeds with the A-value set to the A-number of
    /// the A-instruction and the B-value set to the B-number of the
    /// B-instruction.  A write to core replaces the B-number of the
    /// instruction pointed to by the B-pointer.
    AB,

    /// Instruction execution proceeds with the A-value set to the B-number of
    /// the A-instruction and the B-value set to the A-number of the
    /// B-instruction.  A write to core replaces the A-number of the
    /// instruction pointed to by the B-pointer.
    BA,

    /// Instruction execution proceeds with the A-value set to both the
    /// A-number and B-number of the A-instruction (in that order) and the
    /// B-value set to both the A-number and B-number of the B-instruction
    /// (also in that order).  A write to core replaces both the A-number and
    /// the B-number of the instruction pointed to by the B-pointer (in that
    /// order).
    F,

    /// Instruction execution proceeds with the A-value set to both the
    /// A-number and B-number of the A-instruction (in that order) and the
    /// B-value set to both the B-number and A-number of the B-instruction
    /// (in that order).  A write to to core replaces both the B-number and
    /// the A-number of the instruction pointed to by the B-pointer (in that
    /// order).
    X,

    /// Instruction execution proceeds with the A-value set to the
    /// A-instruction and the B-value set to the B-instruction.  A write to
    /// core replaces the entire instruction pointed to by the B-pointer.
    I,
}

#[allow(
    clippy::use_debug,
    reason = "Debug formatter used to get the modifier mnemonic from enum \
              value"
)]
impl fmt::Display for Modifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use Debug formatter to get the identifier of this variant
        write!(f, "{self:?}")
    }
}

/// The addressing mode applied to the field of an instruction
///
/// Supports addressing modes from the '88 and '94 ICWS standards plus pMARS
/// extensions
#[derive(
    Copy,
    Clone,
    Debug,
    Eq,
    PartialEq,
    Hash,
    FromPrimitive,
    ToPrimitive,
    Serialize,
    Deserialize,
)]
pub enum AddrMode {
    /// Represented by `#`, an immediate mode operand merely serves as storage
    /// for data.  An immediate A/B-mode in the current instruction sets the
    /// A/B-pointer to zero.
    Immediate,

    /// Represented by `$`, a direct mode operand indicates the offset from the
    /// program counter. A direct A/B-mode in the current instruction means the
    /// A/B-pointer is a copy of the offset, the A/B-number of the current
    /// instruction.
    Direct,

    /// Represented by `*`, an A-number indirect mode operand indicates the
    /// primary offset (relative to the program counter) to the secondary
    /// offset (relative to the location of the instruction in which the
    /// secondary offset is contained).  An A-number indirect A/B-mode
    /// indicates that the A/B-pointer is the sum of the A/B-number of the
    /// current instruction (the primary offset) and the A-number of the
    /// instruction pointed to by the A/B-number of the current instruction
    /// (the secondary offset).
    IndirectA, // *

    /// Represented by `@`, a B-number indirect mode operand indicates the
    /// primary offset (relative to the program counter) to the secondary
    /// offset (relative to the location of the instruction in which the
    /// secondary offset is contained).  A B-number indirect A/B-mode indicates
    /// that the A/B-pointer is the sum of the A/B-number of the current
    /// instruction (the primary offset) and the B-number of the instruction
    /// pointed to by the A/B-number of the current instruction (the secondary
    /// offset).
    IndirectB,

    /// Represented by '{', an A-number predecrement indirect mode operand
    /// indicates the primary offset (relative to the program counter) to the
    /// secondary offset (relative to the location of the instruction in which
    /// the secondary offset is contained) which is decremented prior to use.
    /// An A-number predecrement indirect A/B-mode indicates that the
    /// A/B-pointer is the sum of the A/B-number of the current instruction
    /// (the primary offset) and the decremented A-number of the instruction
    /// pointed to by the A/B-number of the current instruction (the secondary
    /// offset).
    PredecA,

    /// Represented by `<`, a B-number predecrement indirect mode operand
    /// indicates the primary offset (relative to the program counter) to the
    /// secondary offset (relative to the location of the instruction in which
    /// the secondary offset is contained) which is decremented prior to use.
    /// A B-number predecrement indirect A/B-mode indicates that the
    /// A/B-pointer is the sum of the A/B-number of the current instruction
    /// (the primary offset) and the decremented B-number of the instruction
    /// pointed to by the A/B-number of the current instruction (the secondary
    /// offset).
    PredecB, // <

    /// Represented by `}`, an A-number postincrement indirect mode operand
    /// indicates the primary offset (relative to the program counter) to
    /// the secondary offset (relative to the location of the instruction
    /// in which the secondary offset is contained) which is incremented
    /// after the results of the operand evaluation are stored.  An
    /// A-number postincrement indirect A/B-mode indicates that the
    /// A/B-pointer is the sum of the A/B-number of the current instruction
    /// (the primary offset) and the A-number of the instruction pointed to
    /// by the A/B-number of the current instruction (the secondary
    /// offset).  The A-number of the instruction pointed to by
    /// the A/B-number of the current instruction is incremented after the
    /// A/B-instruction is stored, but before the B-operand is evaluated (for
    /// A-number postincrement indirect A-mode), or the operation is executed
    /// (for A-number postincrement indirect B-mode).
    PostincA,

    /// Represented by `>`, a B-number postincrement indirect mode operand
    /// indicates the primary offset (relative to the program counter) to
    /// the secondary offset (relative to the location of the instruction
    /// in which the secondary offset is contained) which is incremented
    /// after the results of the operand evaluation are stored.  A B-number
    /// postincrement indirect A/B-mode indicates that the A/B-pointer is
    /// the sum of the A/B-number of the current instruction (the primary
    /// offset) and the B-number of the instruction pointed to by the
    /// A/B-number of the current instruction (the secondary offset).  The
    /// B-number of the instruction pointed to by the A/B-number of the
    /// current instruction is incremented after the A/B-instruction is
    /// stored, but before the B-operand is evaluated (for
    /// B-number postincrement indirect A-mode), or the operation is executed
    /// (for B-number postincrement indirect B-mode).
    PostincB, // >
}

impl fmt::Display for AddrMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Immediate => write!(f, "#"),
            Self::Direct => write!(f, "$"),
            Self::IndirectA => write!(f, "*"),
            Self::IndirectB => write!(f, "@"),
            // "{{" is escaped form of "{"
            Self::PredecA => write!(f, "{{"),
            Self::PredecB => write!(f, "<"),
            // "}}" is escaped for of "}"
            Self::PostincA => write!(f, "}}"),
            Self::PostincB => write!(f, ">"),
        }
    }
}

/// A Redcode assembly instruction including modifiers and addressing modes.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Instruction {
    /// The opcode portion of a redcode instruction e.g. `DAT` or `JMP`
    pub opcode: Opcode,
    /// The modifier portion of a redcode instruction e.g. `.BA` or `.X`
    pub modifier: Modifier,
    /// The addressing mode used by the A field e.g. '>' or `$`
    pub a_addr_mode: AddrMode,
    /// The addressing mode used by the A field e.g. '>' or `$`
    pub b_addr_mode: AddrMode,
}

/// A Redcode instruction (Opcode, modifier, modes) along with field values.
#[derive(
    Copy, Clone, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct CompleteInstruction {
    /// The opcode, modifier, and modes used by this instruction
    pub instr: Instruction,
    /// The A-field stored in this instruction
    pub a_field: FieldValue,
    /// The B-field stored in this instruction
    pub b_field: FieldValue,
}

impl fmt::Display for CompleteInstruction {
    /// Formats an instruction as a '94 loadfile syntax instruction.
    ///
    /// ```
    /// # use redcode::*;
    /// let a = CompleteInstruction {
    ///     instr: Instruction {
    ///         opcode: Opcode::Add,
    ///         modifier: Modifier::AB,
    ///         a_addr_mode: AddrMode::Immediate,
    ///         b_addr_mode: AddrMode::Direct,
    ///     },
    ///     a_field: 16,
    ///     b_field: 32,
    /// };
    ///
    /// assert_eq!(a.to_string(), "Add.AB #16, $32");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}.{} {}{}, {}{}",
            self.instr.opcode,
            self.instr.modifier,
            self.instr.a_addr_mode,
            self.a_field,
            self.instr.b_addr_mode,
            self.b_field
        )
    }
}

impl Default for Instruction {
    /// The default instruction defined by ICWS '94 is `DAT.F $0, $0`.  
    /// In '88 or '86 versions, this is not a valid instruction.  For
    /// compatibility, `DAT.F #0, #0` should be used instead.  
    fn default() -> Self {
        Self {
            opcode: Opcode::Dat,
            modifier: Modifier::F,
            a_addr_mode: AddrMode::Immediate,
            b_addr_mode: AddrMode::Immediate,
        }
    }
}

/// An assembled redcode program
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Warrior {
    /// A sequence of complete compiled redcode instructions
    pub code: Vec<CompleteInstruction>,
    /// Offset *into the warrior* where execution begins
    pub start: FieldValue,
    /// An optional identifier that warriors may optionally specify to indicate
    /// that it should share it's PSPACE with other warriors with the same pin.
    pub pin: Option<i64>,
}

impl Default for Warrior {
    fn default() -> Self {
        Self {
            code: vec![CompleteInstruction::default()],
            start: 0,
            pin: None,
        }
    }
}

#[must_use]
/// Determine the default modifier, and the translation when converting from 88
/// to 94
pub const fn default_modifiers(
    op: Opcode,
    a_mode: AddrMode,
    b_mode: AddrMode,
) -> Modifier {
    #[allow(
        clippy::match_same_arms,
        reason = "Structure match by opcode-group for legibility"
    )]
    match (op, a_mode, b_mode) {
        // Dat and Nop always default to .F
        (Opcode::Nop | Opcode::Dat, ..) => Modifier::F,
        // Mov, Seq, Sne, and Cmp
        // 1) .AB if A-mode is immediate
        // 2) .B if B-Mode is immediate and A-Mode isn't
        // 3) .I if neither A or B mode are immediate
        (
            Opcode::Mov | Opcode::Seq | Opcode::Sne | Opcode::Cmp,
            AddrMode::Immediate,
            _,
        ) => Modifier::AB,
        (
            Opcode::Mov | Opcode::Seq | Opcode::Sne | Opcode::Cmp,
            _,
            AddrMode::Immediate,
        ) => Modifier::B,
        (Opcode::Mov | Opcode::Seq | Opcode::Sne | Opcode::Cmp, ..) => {
            Modifier::I
        }
        // Add, Sub, Mul, Div, and Mod
        // 1) .AB if A-mode is immediate
        // 2) .B if B-Mode is immediate and A-Mode isn't
        // 3) .F if neither A or B mode are immediate
        (
            Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::Div | Opcode::Mod,
            AddrMode::Immediate,
            _,
        ) => Modifier::AB,
        (
            Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::Div | Opcode::Mod,
            _,
            AddrMode::Immediate,
        ) => Modifier::B,
        (
            Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::Div | Opcode::Mod,
            ..,
        ) => Modifier::F,
        // Slt, Ldb, and Stp
        // 1) .AB if A-mode is immediate
        // 2) .B in all other cases
        (Opcode::Slt | Opcode::Ldp | Opcode::Stp, AddrMode::Immediate, _) => {
            Modifier::AB
        }
        (Opcode::Slt | Opcode::Ldp | Opcode::Stp, ..) => Modifier::B,
        // Jmp, Jmz, Jmn, Djn, Spl are always .B
        (
            Opcode::Jmp | Opcode::Jmz | Opcode::Jmn | Opcode::Djn | Opcode::Spl,
            ..,
        ) => Modifier::B,
    }
}

/// Utilities for enumerating and iterating over all valid redcode instructions
pub mod test_utils {
    use super::*;

    /// All valid opcodes for '88, '94, and pMARS extensions to redcode
    pub const OPCODES: [Opcode; 19] = [
        Opcode::Dat,
        Opcode::Mov,
        Opcode::Add,
        Opcode::Sub,
        Opcode::Mul,
        Opcode::Div,
        Opcode::Mod,
        Opcode::Jmp,
        Opcode::Jmz,
        Opcode::Jmn,
        Opcode::Djn,
        Opcode::Spl,
        Opcode::Slt,
        Opcode::Cmp,
        Opcode::Seq,
        Opcode::Sne,
        Opcode::Nop,
        Opcode::Ldp,
        Opcode::Stp,
    ];

    /// All valid modifiers for '88, '94, and pMARS extensions to redcode
    pub const MODIFIERS: [Modifier; 7] = [
        Modifier::A,
        Modifier::B,
        Modifier::AB,
        Modifier::BA,
        Modifier::F,
        Modifier::X,
        Modifier::I,
    ];

    /// All valid Addressing modes for '88, '94, and pMARS extensions to redcode
    pub const ADDR_MODES: [AddrMode; 8] = [
        AddrMode::Immediate,
        AddrMode::Direct,
        AddrMode::IndirectA,
        AddrMode::IndirectB,
        AddrMode::PredecA,
        AddrMode::PredecB,
        AddrMode::PostincA,
        AddrMode::PostincB,
    ];

    /// iterate over every valid redcode instruction including '88 and '94
    /// standards, as well as pMARS extensions.
    pub fn all_instructions() -> impl Iterator<Item = Instruction> {
        itertools::iproduct!(
            OPCODES.iter(),
            MODIFIERS.iter(),
            ADDR_MODES.iter(),
            ADDR_MODES.iter()
        )
        .map(|(o, m, a, b)| Instruction {
            opcode: *o,
            modifier: *m,
            a_addr_mode: *a,
            b_addr_mode: *b,
        })
    }
}

#[cfg(test)]
mod tests {
    use coverage_helper::test;
    use itertools::Itertools;

    use super::*;
    use crate::test_utils::all_instructions;

    #[test]
    fn test_instr_default_equ() {
        let default: Instruction = Default::default();
        let manual = Instruction {
            opcode: Opcode::Dat,
            modifier: Modifier::F,
            a_addr_mode: AddrMode::Immediate,
            b_addr_mode: AddrMode::Immediate,
        };
        assert_eq!(default, manual);
    }

    #[test]
    fn test_default_warrior() {
        let default: Warrior = Default::default();
        let manual = Warrior {
            code: vec![CompleteInstruction::default()],
            start: 0,
            pin: None,
        };
        assert_eq!(default, manual);
    }

    #[test]
    fn enumerate_instructions_are_unique() {
        let instructions: Vec<Instruction> =
            test_utils::all_instructions().collect();
        let unique_instructions: Vec<Instruction> =
            test_utils::all_instructions().unique().collect();

        assert_eq!(instructions.len(), unique_instructions.len());
    }

    #[test]
    fn enumerate_instructions_right_number() {
        let expected_number = test_utils::OPCODES.len()
            * test_utils::MODIFIERS.len()
            * test_utils::ADDR_MODES.len()
            * test_utils::ADDR_MODES.len();
        assert_eq!(all_instructions().count(), expected_number);
    }

    #[test]
    fn all_instructions_have_unique_display() {
        let a_field = 123;
        let b_field = 456;
        let instructions_displayed: Vec<String> =
            test_utils::all_instructions()
                .map(|instr| CompleteInstruction {
                    instr,
                    a_field,
                    b_field,
                })
                .map(|x| x.to_string())
                .collect();

        let unique_display_reprs =
            instructions_displayed.iter().unique().count();
        assert_eq!(unique_display_reprs, instructions_displayed.len());
    }

    #[test]
    fn all_values_support_to_u8() {
        use num_traits::cast::ToPrimitive;
        // num_traits specifies that if the number of variants is within the
        // range of the specified type, then ToPrimitive should always return
        // Some.  Test that this remains true, or that more variants haven't
        // been added which break the assumptions that all types can fit within
        // a u8

        for op in test_utils::OPCODES {
            assert!(op.to_u8().is_some());
        }
        for modifier in test_utils::MODIFIERS {
            assert!(modifier.to_u8().is_some());
        }
        for mode in test_utils::ADDR_MODES {
            assert!(mode.to_u8().is_some())
        }
    }
}

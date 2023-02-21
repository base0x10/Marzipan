use core::convert::Into;

use crate::{CompleteInstruction, FieldValue, Instruction, Warrior};

/// A [`CompleteInstruction`] that allows field values less than zero or
/// greater than `core_size`.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
pub struct RelaxedCompleteInstruction {
    /// The opcode, modifier, and modes used by this instruction
    pub instr: Instruction,
    /// The A-field stored in this instruction, interpreted modulo `core_size`
    pub a_field: i64,
    /// The B-field stored in this instruction, interpreted modulo `core_size`
    pub b_field: i64,
}

impl RelaxedCompleteInstruction {
    /// Convert into a [`CompleteInstruction`], possibly by evaluating fields
    /// modulo `core_size`
    pub fn normalize<T>(&self, core_size: T) -> CompleteInstruction
    where T: Into<u64> + Copy {
        CompleteInstruction {
            instr: self.instr,
            a_field: normalize(self.a_field, core_size),
            b_field: normalize(self.b_field, core_size),
        }
    }
}

/// A [`Warrior`] with [`RelaxedCompleteInstruction`]s that allow field values
/// less than zero or greater than `core_size`
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct RelaxedWarrior {
    /// A sequence of redcode instructions
    pub code: Vec<RelaxedCompleteInstruction>,
    /// Offset from the start of a warrior where execution begins
    ///
    /// Systems may or may not accept values outside of the range
    /// 0..war_len. pMARS issues a warning but continues anyway.
    pub start: i64,
    /// An optional identifier that warriors may optionally specify to indicate
    /// that it should share it's PSPACE with other warriors with the same pin.
    pub pin: Option<i64>,
}

impl RelaxedWarrior {
    /// Convert into a [`Warrior`] consisting of [`CompleteInstruction`]s,
    /// possibly by evaluating fields modulo `core_size`
    pub fn normalize<T>(&self, core_size: T) -> Warrior
    where T: Into<u64> + Copy {
        let code = self
            .code
            .iter()
            .map(|insn| insn.normalize(core_size))
            .collect();
        Warrior {
            code,
            start: normalize(self.start, core_size),
            pin: self.pin,
        }
    }
}

impl Default for RelaxedWarrior {
    fn default() -> Self {
        Self {
            code: vec![RelaxedCompleteInstruction::default()],
            start: 0,
            pin: None,
        }
    }
}

/// Evaluate a value as if it is a core offset, wrapping around at `core_size`.
///
/// # Panics
///
/// Will panic if `core_size` is less than 0 or greater than `u32::MAX`.
pub fn normalize<T, K>(value: K, core_size: T) -> FieldValue
where
    T: Into<u64> + Copy,
    K: Into<i64>,
{
    assert!(core_size.into() < u64::from(u32::MAX));
    assert!(core_size.into() > 0);
    let core_size: i64 = core_size.into().try_into().unwrap_or(0);
    let mut v = value.into();
    while v < 0 {
        v = v.wrapping_add(core_size);
    }
    let normalized = v.checked_rem(core_size).unwrap_or(0);
    normalized.try_into().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{RelaxedCompleteInstruction, RelaxedWarrior};
    use crate::Instruction;

    #[test]
    fn verify_positive_and_negative_conversions() {
        let i = RelaxedCompleteInstruction {
            instr: Instruction::default(),
            a_field: -10,
            b_field: 20,
        };
        let normalized = i.normalize(15_u32);
        assert_eq!(normalized.a_field, 5);
        assert_eq!(normalized.b_field, 5);
    }

    #[test]
    #[should_panic]
    fn normalize_instr_with_zero_coresize() {
        let i = RelaxedCompleteInstruction {
            instr: Instruction::default(),
            a_field: 0,
            b_field: 0,
        };
        let _normalized = i.normalize(0_u32);
    }

    #[test]
    #[should_panic]
    fn normalize_instr_with_massive_coresize() {
        let i = RelaxedCompleteInstruction {
            instr: Instruction::default(),
            a_field: 0,
            b_field: 0,
        };
        let _normalized = i.normalize(u32::max as u64 + 1);
    }

    #[test]
    #[should_panic]
    fn normalize_warrior_with_zero_coresize() {
        let war = RelaxedWarrior {
            code: vec![RelaxedCompleteInstruction {
                instr: Instruction::default(),
                a_field: 0,
                b_field: 0,
            }],
            start: 0,
            pin: None,
        };
        let _normalized = war.normalize(0_u32);
    }

    #[test]
    #[should_panic]
    fn normalize_warrior_with_massive_coresize() {
        let war = RelaxedWarrior {
            code: vec![RelaxedCompleteInstruction {
                instr: Instruction::default(),
                a_field: 0,
                b_field: 0,
            }],
            start: 0,
            pin: None,
        };
        let _normalized = war.normalize(u32::max as u64 + 1);
    }
}

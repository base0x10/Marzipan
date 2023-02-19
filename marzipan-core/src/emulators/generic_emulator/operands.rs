use redcode::{AddrMode, CompleteInstruction, Instruction};

use super::offset;
use crate::{
    emulator_core::{EmulatorError, EmulatorResult},
    CoreAddr,
};

/// Loaded at the start of a cycle, not touched by later core modifications
#[derive(Copy, Clone, Debug)]
pub struct RegisterValues {
    /// The PC and the content of the instruction it points to
    pub current: RegisterValue,

    /// The A target and A values
    pub a: RegisterValue,

    /// The B target and B values
    pub b: RegisterValue,
}

/// A core index (e.g. PC or field target) and contents
#[derive(Copy, Clone, Debug)]
pub struct RegisterValue {
    /// Core index for the instruction that occupies this register
    pub idx: CoreAddr,
    /// Decoded instruction
    pub instr: Instruction,
    /// A field for the instruction in this register
    pub a_field: CoreAddr,
    /// B field for the instruction in this register
    pub b_field: CoreAddr,
}

// TODO(jespy) Break up this function into simpler components, and enable this
// lint for each
#[allow(
    clippy::indexing_slicing,
    reason = "Removing indexing adds redundant logic for error handling and \
              to satisfy the borrow checker"
)]
/// Evaluate the A and B operands according to the operand modifier
///
/// Because `PostIncrement` may modify the core core, the values in
/// [`RegisterValue`] are not guaranteed to match the in-core values.  
///
/// # Errors
///
/// Returns [`EmulatorError::InternalError`] in exceptional circumstances.
/// Typically this is the result of invalid parameters, or core corruption where
/// field values exceed `core_size - 1`.
pub fn evaluate(
    pc: CoreAddr,
    core: &mut [CompleteInstruction],
) -> EmulatorResult<RegisterValues> {
    let size = core.len();

    // Cache a copy of the current instruction before any writes to the core
    let pc_idx = usize::try_from(pc).or(Err(EmulatorError::InternalError(
        "unable to convert pc into usize",
    )))?;
    let cur = *core
        .get(pc_idx)
        .ok_or(EmulatorError::InternalError("pc larger than core size"))?;

    // The index into the core pointed to by the a_field of the current
    // instruction.  This is used lots of places.
    //  - This is used as the target if the mode is Direct
    //  - One of the fields in the instruction pointed to by this is used as as
    //    the target if the mode is Indirect
    //  - If mode is any sort of predecrement or postincrement, one of the
    //    fields in the instruction this points to is modified.
    let a_indirect_index_value = add(cur.a_field, pc, size)?;
    let a_indirect_index = usize::try_from(a_indirect_index_value).or(Err(
        EmulatorError::InternalError("unable to convert core field into usize"),
    ))?;

    // Possibly predecrement one of the fields of the instruction pointed to by
    // the a_field of the current instruction
    match cur.instr.a_addr_mode {
        AddrMode::PredecA => {
            decrement(&mut core[a_indirect_index].a_field, size)?;
        }
        AddrMode::PredecB => {
            decrement(&mut core[a_indirect_index].b_field, size)?;
        }
        _ => {}
    };

    // Evaluate the A operand
    // Cache the target index and the value of the instruction it points to
    let a_target: CoreAddr = match cur.instr.a_addr_mode {
        AddrMode::Immediate => pc,
        AddrMode::Direct => add(cur.a_field, pc, size)?,
        AddrMode::IndirectA | AddrMode::PredecA | AddrMode::PostincA => {
            add(a_indirect_index_value, core[a_indirect_index].a_field, size)?
        }
        AddrMode::IndirectB | AddrMode::PredecB | AddrMode::PostincB => {
            add(a_indirect_index_value, core[a_indirect_index].b_field, size)?
        }
    };
    let a_target_idx = usize::try_from(a_target).or(Err(
        EmulatorError::InternalError("unable to convert core field into usize"),
    ))?;
    let a_instr = core[a_target_idx];

    // Possibly postincrement one of the fields of the instruction pointed to by
    // the a_field of the current instruction
    match cur.instr.a_addr_mode {
        AddrMode::PostincA => {
            increment(&mut core[a_indirect_index].a_field, size)?;
        }
        AddrMode::PostincB => {
            increment(&mut core[a_indirect_index].b_field, size)?;
        }
        _ => {}
    };

    // The index into the core pointed to by the b_field of the current
    // instruction.  This is used lots of places.
    //  - This is used as the target if the mode is Direct
    //  - One of the fields in the instruction pointed to by this is used as as
    //    the target if the mode is Indirect
    //  - If mode is any sort of predecrement or postincrement, one of the
    //    fields in the instruction this points to is modified.
    let b_indirect_index_value = add(cur.b_field, pc, size)?;
    let b_indirect_index = usize::try_from(b_indirect_index_value).or(Err(
        EmulatorError::InternalError("unable to convert core field into usize"),
    ))?;

    // Possibly predecrement one of the fields of the instruction pointed to by
    // the b_field of the current instruction
    match cur.instr.b_addr_mode {
        AddrMode::PredecA => {
            decrement(&mut core[b_indirect_index].a_field, size)?;
        }
        AddrMode::PredecB => {
            decrement(&mut core[b_indirect_index].b_field, size)?;
        }
        _ => {}
    };

    // Evaluate the B operand
    // Cache the target index and the value of the instruction it points to
    let b_target: CoreAddr = match cur.instr.b_addr_mode {
        AddrMode::Immediate => pc,
        AddrMode::Direct => add(cur.b_field, pc, size)?,
        AddrMode::IndirectA | AddrMode::PredecA | AddrMode::PostincA => {
            add(b_indirect_index_value, core[b_indirect_index].a_field, size)?
        }
        AddrMode::IndirectB | AddrMode::PredecB | AddrMode::PostincB => {
            add(b_indirect_index_value, core[b_indirect_index].b_field, size)?
        }
    };
    let b_target_idx = usize::try_from(b_target).or(Err(
        EmulatorError::InternalError("unable to convert core field into usize"),
    ))?;
    let b_instr = core[b_target_idx];

    // Possibly postincrement one of the fields of the instruction pointed to by
    // the b_field of the current instruction
    match cur.instr.b_addr_mode {
        AddrMode::PostincA => {
            increment(&mut core[b_indirect_index].a_field, size)?;
        }
        AddrMode::PostincB => {
            increment(&mut core[b_indirect_index].b_field, size)?;
        }
        _ => {}
    };

    Ok(RegisterValues {
        current: RegisterValue {
            idx: validate(pc, size)?,
            instr: cur.instr,
            a_field: validate(cur.a_field, size)?,
            b_field: validate(cur.b_field, size)?,
        },
        a: RegisterValue {
            idx: validate(a_target, size)?,
            instr: a_instr.instr,
            a_field: validate(a_instr.a_field, size)?,
            b_field: validate(a_instr.b_field, size)?,
        },
        b: RegisterValue {
            idx: validate(b_target, size)?,
            instr: b_instr.instr,
            a_field: validate(b_instr.a_field, size)?,
            b_field: validate(b_instr.b_field, size)?,
        },
    })
}

/// Validate an address, lookup the value at that address, and add one modulo
/// core size
fn increment(val: &mut CoreAddr, size: usize) -> EmulatorResult<()> {
    let Ok(size) = CoreAddr::try_from(size) else {
        return Err(EmulatorError::InternalError(
            "core size too large to be converted into CoreAddr u32",
        ))
    };
    let new_val = super::offset(*val, 1, size)?;
    *val = new_val;
    Ok(())
}

/// Validate an address, lookup the value at that address, and subtract one
/// modulo core size
fn decrement(val: &mut CoreAddr, size: usize) -> EmulatorResult<()> {
    let Ok(size) = CoreAddr::try_from(size) else {
        return Err(EmulatorError::InternalError(
            "core size too large to be converted into CoreAddr u32",
        ))
    };
    let new_val = super::offset(*val, -1, size)?;
    *val = new_val;
    Ok(())
}

/// Add two values modulo core size
fn add(lhs: CoreAddr, rhs: CoreAddr, size: usize) -> EmulatorResult<CoreAddr> {
    let Ok(size) = CoreAddr::try_from(size) else {
        return Err(EmulatorError::InternalError(
            "core size too large to be converted into CoreAddr u32",
        ))
    };
    offset(lhs, rhs.into(), size)
}

/// Verify that a core value is valid relative to the core size
fn validate(val: CoreAddr, size: usize) -> EmulatorResult<CoreAddr> {
    let Ok(size) = CoreAddr::try_from(size) else {
        return Err(EmulatorError::InternalError(
            "core size too large to be converted into CoreAddr u32",
        ))
    };
    if val < (size as CoreAddr) {
        Ok(val)
    } else {
        Err(EmulatorError::InternalError(
            "Invalid core value greater than core size",
        ))
    }
}

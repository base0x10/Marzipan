use redcode::{CompleteInstruction, Modifier, Opcode};

use super::{
    offset, operands::RegisterValues, processes::ProcessQueueSet, pspace,
};
use crate::{
    emulator_core::{EmulatorError, EmulatorResult},
    CoreAddr,
};

/// The results of operand evaluation and the core state required to emulation
/// an instruction.
pub struct OpInputs<'a> {
    /// Currently executing warrior
    pub warrior_id: u64,
    /// Decoded and evaluated cached operands and current instruction
    pub regs: &'a RegisterValues,
    /// Currently configured core size
    pub core_size: CoreAddr,
    /// The process queue.  Emulation functions don't pop the PC, but do
    /// enqueue processes.
    pub pq: &'a mut ProcessQueueSet,
    /// Reference to in-core instructions.
    pub core: &'a mut [CompleteInstruction],
    /// PSPACE state shared by processes in the core.
    pub pspace: &'a mut pspace::PSpace,
}

impl<'a> OpInputs<'a> {
    /// Gets a mutable reference to an in-core address
    ///
    /// This helper improves error handling and allows enabling clippy's
    /// `indexing_slicing` lint. However I'd like to rip it out.
    /// Specifically, it consumes ownership of [`OpInputs`] meaning that
    /// I need to jump through hoops to keep the borrow checker happy.
    ///
    /// # Errors
    ///
    /// Returns an error if `addr` is invalid
    fn core_get_mut(
        self: OpInputs<'a>,
        addr: CoreAddr,
    ) -> EmulatorResult<&'a mut CompleteInstruction> {
        self.core
            .get_mut(addr as usize)
            .ok_or(EmulatorError::InternalError(
                "attempt to write to invalid core index",
            ))
    }
}

/// Implementation of the [`Opcode::Dat`] instruction
#[allow(clippy::unnecessary_wraps)]
#[allow(clippy::missing_const_for_fn)]
pub fn dat_op(_inputs: OpInputs) -> EmulatorResult<()> {
    // Do nothing past operand evaluation
    // Queue no further values to the process queue
    Ok(())
}

/// Implementation of the [`Opcode::Mov`] instruction
pub fn mov_op(inputs: OpInputs) -> EmulatorResult<()> {
    let next_pc = offset(inputs.regs.current.idx, 1, inputs.core_size)?;
    inputs.pq.push_back(next_pc, inputs.warrior_id)?;
    match inputs.regs.current.instr.modifier {
        Modifier::A => {
            // A MOV.A instruction would replace the A-number of the
            // instruction pointed to by the B-pointer with the A-number of the
            // A-instruction.
            let a_value = inputs.regs.a.a_field;
            let b_pointer = inputs.regs.b.idx;
            inputs.core_get_mut(b_pointer)?.a_field = a_value;
        }
        Modifier::B => {
            // A MOV.B instruction would replace the B-number of the
            // instruction pointed to by the B-pointer with the B-number of the
            // A-instruction.
            let a_value = inputs.regs.a.b_field;
            let b_pointer = inputs.regs.b.idx;
            inputs.core_get_mut(b_pointer)?.b_field = a_value;
        }
        Modifier::AB => {
            // A MOV.AB instruction would replace the B-number of the
            // instruction pointed to by the B-pointer with the A-number of the
            // A-instruction.
            let a_value = inputs.regs.a.a_field;
            let b_pointer = inputs.regs.b.idx;
            inputs.core_get_mut(b_pointer)?.b_field = a_value;
        }
        Modifier::BA => {
            // A MOV.BA instruction would replace the A-number of the
            // instruction pointed to by the B-pointer with the B-number of the
            // A-instruction.
            let a_value = inputs.regs.a.b_field;
            let b_pointer = inputs.regs.b.idx;
            inputs.core_get_mut(b_pointer)?.a_field = a_value;
        }
        Modifier::F => {
            // A MOV.F instruction would replace the A-number of the
            // instruction pointed to by the B-pointer with the A-number of the
            // A-instruction and would also replace the B-number of the
            // instruction pointed to by the B-pointer with the B-number of the
            // A-instruction.
            let a_value_a = inputs.regs.a.a_field;
            let b_value_b = inputs.regs.b.b_field;
            let b_pointer = inputs.regs.b.idx;
            let target = inputs.core_get_mut(b_pointer)?;
            target.a_field = a_value_a;
            target.b_field = b_value_b;
        }
        Modifier::X => {
            // A MOV.F instruction would replace the A-number of the
            // instruction pointed to by the B-pointer with the A-number of the
            // A-instruction and would also replace the B-number of the
            // instruction pointed to by the B-pointer with the B-number of the
            // A-instruction.
            let a_value_b = inputs.regs.a.b_field;
            let b_value_a = inputs.regs.b.a_field;
            let b_pointer = inputs.regs.b.idx;
            let target = inputs.core_get_mut(b_pointer)?;
            target.a_field = a_value_b;
            target.b_field = b_value_a;
        }
        Modifier::I => {
            // A MOV.I instruction would replace the instruction pointed to by
            // the B-pointer with the A-instruction.
            let a_value_a = inputs.regs.a.a_field;
            let a_value_b = inputs.regs.a.b_field;
            let a_value_instr = inputs.regs.a.instr;
            let b_pointer = inputs.regs.b.idx;
            let target = inputs.core_get_mut(b_pointer)?;
            target.instr = a_value_instr;
            target.a_field = a_value_a;
            target.b_field = a_value_b;
        }
    };
    Ok(())
}

/// Helper function that determines which arithmetic operation is required, and
/// performs it for two [`CoreAddr`] values.
///
/// For instances where the right hand argument is treated as a divisor, if it
/// is zero, `None` is returned.
fn perform_arithmetic(
    lhs: CoreAddr,
    rhs: CoreAddr,
    inputs: &OpInputs,
) -> Option<EmulatorResult<CoreAddr>> {
    // Performs an math modulo coresize, returning None only if division by zero
    match inputs.regs.current.instr.opcode {
        Opcode::Add => Some(offset(lhs, i64::from(rhs), inputs.core_size)),
        Opcode::Sub => {
            // offset() deals with negatives correctly
            Some(offset(
                lhs,
                0_i64.checked_sub(i64::from(rhs))?,
                inputs.core_size,
            ))
        }
        Opcode::Mul => {
            let product = u64::from(lhs).checked_mul(u64::from(rhs));
            let normalized = product
                .and_then(|p| p.checked_rem(u64::from(inputs.core_size)))
                .and_then(|e| u32::try_from(e).ok());
            Some(normalized.ok_or(EmulatorError::InternalError(
                "Impossible overflow when multiplying field values",
            )))
        }
        Opcode::Div => (rhs != 0).then(|| {
            lhs.checked_div(rhs).ok_or(EmulatorError::InternalError(
                "Impossible division by zero",
            ))
        }),
        Opcode::Mod => (rhs != 0).then(|| {
            lhs.checked_rem(rhs).ok_or(EmulatorError::InternalError(
                "Impossible division by zero",
            ))
        }),
        _ => Some(Err(EmulatorError::InternalError(
            "fn arithmetic_op should only be called with Add, Sub, Mul, Div, \
             or Mod",
        ))),
    }
}

/// Implementation of the [`Opcode::Add`], [`Opcode::Sub`], [`Opcode::Mul`],
/// [`Opcode::Div`], and [`Opcode::Mod`] instruction
pub fn arithmetic_op(inputs: OpInputs) -> EmulatorResult<()> {
    let a = inputs.regs.a;
    let b = inputs.regs.b;
    let next_pc = offset(inputs.regs.current.idx, 1, inputs.core_size)?;
    let war_id = inputs.warrior_id;
    if inputs.core_size == 0 {
        return Err(EmulatorError::InternalError("Core Size cannot be zero"));
    }

    match inputs.regs.current.instr.modifier {
        Modifier::A => {
            // Proceeds with A value set to the A number of the A instruction
            // and the B value set to the A number of the B instruction.
            // Writes to the A number of the B target
            // let b_pointer = b.idx as usize;
            let a_value = a.a_field;
            let b_value = b.a_field;
            if let Some(res) = perform_arithmetic(b_value, a_value, &inputs) {
                inputs.pq.push_back(next_pc, war_id)?;
                inputs.core_get_mut(b.idx)?.a_field = res?;
            };
        }
        Modifier::B => {
            // Proceeds with A value set to the B number of the A instruction
            // and the B value set to the B number of the B instruction.
            // Writes to the B number of the B target
            let a_value = a.b_field;
            let b_value = b.b_field;
            if let Some(res) = perform_arithmetic(b_value, a_value, &inputs) {
                inputs.pq.push_back(next_pc, war_id)?;
                inputs.core_get_mut(b.idx)?.b_field = res?;
            }
        }
        Modifier::AB => {
            // Proceeds with A value set to the A number of the A instruction
            // and the B value set to the B number of the B instruction.
            // Writes to the B number of the B target
            // let b_pointer = b.idx as usize;
            let a_value = a.a_field;
            let b_value = b.b_field;
            if let Some(res) = perform_arithmetic(b_value, a_value, &inputs) {
                inputs.pq.push_back(next_pc, war_id)?;
                inputs.core_get_mut(b.idx)?.b_field = res?;
            }
        }
        Modifier::BA => {
            // Proceeds with A value set to the B number of the A instruction
            // and the B value set to the A number of the B instruction.
            // Writes to the A number of the B target
            let a_value = a.b_field;
            let b_value = b.a_field;
            if let Some(res) = perform_arithmetic(b_value, a_value, &inputs) {
                inputs.pq.push_back(next_pc, war_id)?;
                inputs.core_get_mut(b.idx)?.a_field = res?;
            }
        }
        Modifier::F | Modifier::I => {
            // Add/Sub.I functions as Add/Sub.F would
            // F Proceeds with A value set to the A number followed by the B
            // number of the A instruction, and the B value set to the A number
            // followed by the B number of the B instruction.
            // Writes to first the A number followed by the B number of the
            // B target
            let first_result =
                perform_arithmetic(b.a_field, a.a_field, &inputs);
            let second_result =
                perform_arithmetic(b.b_field, a.b_field, &inputs);
            match (first_result, second_result) {
                (Some(first), Some(second)) => {
                    // if there was no division by zero, continue as normal
                    inputs.pq.push_back(next_pc, war_id)?;
                    let target = inputs.core_get_mut(b.idx)?;
                    target.a_field = first?;
                    target.b_field = second?;
                }
                (Some(first), None) => {
                    // If second result had a division by zero, write out first
                    // result but don't write second, and
                    // don't queue PC + 1
                    inputs.core_get_mut(b.idx)?.a_field = first?;
                }
                (None, Some(second)) => {
                    // If first result had a division by zero, write out second
                    // result but don't write first, and
                    // don't queue PC + 1
                    inputs.core_get_mut(b.idx)?.b_field = second?;
                }
                (None, None) => {
                    // If both results had division by zero don't write anything
                    // to core don't queue PC + 1
                }
            };
        }
        Modifier::X => {
            // Proceeds with A value set to the A number followed by the B
            // number of the A instruction, and the B value set to the B number
            // followed by the A number of the B instruction.
            // Writes to first the B number followed by the A number of the
            // B target
            // let b_pointer = b.idx as usize;
            let first_result =
                perform_arithmetic(b.b_field, a.a_field, &inputs);
            let second_result =
                perform_arithmetic(b.a_field, a.b_field, &inputs);
            match (first_result, second_result) {
                (Some(first), Some(second)) => {
                    // if there was no division by zero, continue as normal
                    inputs.pq.push_back(next_pc, war_id)?;
                    let target = inputs.core_get_mut(b.idx)?;
                    target.b_field = first?;
                    target.a_field = second?;
                }
                (Some(first), None) => {
                    // If second result had a division by zero, write out first
                    // result but don't write second, and
                    // don't queue PC + 1
                    inputs.core_get_mut(b.idx)?.b_field = first?;
                }
                (None, Some(second)) => {
                    // If first result had a division by zero, write out second
                    // result but don't write first, and
                    // don't queue PC + 1
                    inputs.core_get_mut(b.idx)?.a_field = second?;
                }
                (None, None) => {
                    // If both results had division by zero don't write anything
                    // to core don't queue PC + 1
                }
            }
        }
    }
    Ok(())
}

/// Implementation of the [`Opcode::Jmp`] instruction
pub fn jmp_op(inputs: OpInputs) -> EmulatorResult<()> {
    // jmp unconditionally adds the b pointer to the process queue
    inputs.pq.push_back(inputs.regs.a.idx, inputs.warrior_id)?;
    Ok(())
}

/// Implementation of the [`Opcode::Jmz`] instruction
pub fn jmz_op(inputs: OpInputs) -> EmulatorResult<()> {
    // JMZ tests the B-value to determine if it is zero.  If the B-value is
    // zero, the sum of the program counter and the A-pointer is queued.
    // Otherwise, the next instruction is queued (PC + 1).  JMZ.I functions
    // as JMZ.F would, i.e. it jumps if both the A-number and the B-number of
    // the B-instruction are zero.
    let a = inputs.regs.a;
    let b = inputs.regs.b;
    let is_zero = match inputs.regs.current.instr.modifier {
        Modifier::A | Modifier::BA => {
            // B value is the A-number of the B instruction
            b.a_field == 0
        }
        Modifier::B | Modifier::AB => {
            // B value is the B-number of the B instruction
            b.b_field == 0
        }
        Modifier::F | Modifier::X | Modifier::I => {
            // B value is the A and B numbers of the B instruction
            b.a_field == 0 && b.b_field == 0
        }
    };
    if is_zero {
        inputs.pq.push_back(a.idx, inputs.warrior_id)?;
    } else {
        let next_pc = offset(inputs.regs.current.idx, 1, inputs.core_size)?;
        inputs.pq.push_back(next_pc, inputs.warrior_id)?;
    }
    Ok(())
}

/// Implementation of the [`Opcode::Jmn`] instruction
pub fn jmn_op(inputs: OpInputs) -> EmulatorResult<()> {
    // JMN tests the B-value to determine if it is zero.  If the B-value is not
    // zero, the sum of the program counter and the A-pointer is queued.
    // Otherwise, the next instruction is queued (PC + 1).  JMN.I functions as
    // JMN.F would, i.e. it jumps if both the A-number and the B-number of the
    // B-instruction are non-zero. This is not the negation of the condition
    // for JMZ.F.
    let a = inputs.regs.a;
    let b = inputs.regs.b;
    let is_non_zero = match inputs.regs.current.instr.modifier {
        Modifier::A | Modifier::BA => {
            // B value is the A-number of the B instruction
            b.a_field != 0
        }
        Modifier::B | Modifier::AB => {
            // B value is the B-number of the B instruction
            b.b_field != 0
        }
        Modifier::F | Modifier::X | Modifier::I => {
            // B value is the A and B numbers of the B instruction
            b.a_field != 0 || b.b_field != 0
        }
    };
    if is_non_zero {
        inputs.pq.push_back(a.idx, inputs.warrior_id)?;
    } else {
        let next_pc = offset(inputs.regs.current.idx, 1, inputs.core_size);
        inputs.pq.push_back(next_pc?, inputs.warrior_id)?;
    }
    Ok(())
}

/// Implementation of the [`Opcode::Djn`] instruction
pub fn djn_op(inputs: OpInputs) -> EmulatorResult<()> {
    // DJN decrements the B-value and the B-target, then tests the B-value to
    // determine if it is zero.  If the decremented B-value is not zero, the
    // sum of the program counter and the A-pointer is queued. Otherwise, the
    // next instruction is queued (PC + 1).  DJN.I functions as DJN.F would,
    // i.e. it decrements both both A/B-numbers of the B-value and the
    // B-target, and jumps if both A/B-numbers of the B-value are non-zero.
    let a = inputs.regs.a;
    let b = inputs.regs.b;
    let size = inputs.core_size;
    let decrement = |x| offset(x, -1, size);
    let next_pc = offset(inputs.regs.current.idx, 1, inputs.core_size)?;
    let war_id = inputs.warrior_id;
    let modifier = inputs.regs.current.instr.modifier;
    let Some(b_target) = inputs.core.get_mut(b.idx as usize) else {
        return Err(EmulatorError::InternalError(
            "attempt to write to invalid core index",
        ))
    };

    match modifier {
        Modifier::A | Modifier::BA => {
            // B value is the A-number of the B instruction
            // decrement b target
            let b_target_a = b_target.a_field;
            let b_target_a = decrement(b_target_a)?;
            b_target.a_field = b_target_a;
            let non_zero = decrement(b.a_field)? != 0;
            if non_zero {
                inputs.pq.push_back(a.idx, war_id)?;
            } else {
                inputs.pq.push_back(next_pc, war_id)?;
            }
        }
        Modifier::B | Modifier::AB => {
            // B value is the B-number of the B instruction
            // decrement b target
            let b_target_b = b_target.b_field;
            let b_target_b = decrement(b_target_b)?;
            b_target.b_field = b_target_b;
            let non_zero = decrement(b.b_field)? != 0;
            if non_zero {
                inputs.pq.push_back(a.idx, war_id)?;
            } else {
                inputs.pq.push_back(next_pc, war_id)?;
            }
        }
        Modifier::F | Modifier::X | Modifier::I => {
            // B value is the A and B numbers of the B instruction
            // decrement b target
            let b_target_a = b_target.a_field;
            let b_target_a = decrement(b_target_a)?;
            let b_target_b = b_target.b_field;
            let b_target_b = decrement(b_target_b)?;
            b_target.a_field = b_target_a;
            b_target.b_field = b_target_b;
            let non_zero =
                decrement(b.a_field)? != 0 || decrement(b.b_field)? != 0;
            if non_zero {
                inputs.pq.push_back(a.idx, war_id)?;
            } else {
                inputs.pq.push_back(next_pc, war_id)?;
            }
        }
    };
    Ok(())
}

/// Implementation of the [`Opcode::Spl`] instruction
pub fn spl_op(inputs: OpInputs) -> EmulatorResult<()> {
    // SPL queues the next instruction (PC + 1) and then queues the sum of the
    // program counter and A-pointer. If the queue is full, only the next
    // instruction is queued.
    let next_pc = offset(inputs.regs.current.idx, 1, inputs.core_size);
    inputs.pq.push_back(next_pc?, inputs.warrior_id)?;
    inputs.pq.push_back(inputs.regs.a.idx, inputs.warrior_id)?;
    Ok(())
}

/// Implementation of the [`Opcode::Slt`] instruction
pub fn slt_op(inputs: OpInputs) -> EmulatorResult<()> {
    // SLT compares the A-value to the B-value.  If the A-value is less than
    // the B-value, the instruction after the next instruction (PC + 2) is
    // queued (skipping the next instruction).  Otherwise, the next
    // instruction is queued (PC + 1).  SLT.I functions as SLT.F would.
    let a = inputs.regs.a;
    let b = inputs.regs.b;
    let is_less_than = match inputs.regs.current.instr.modifier {
        Modifier::A => a.a_field < b.a_field,
        Modifier::B => a.b_field < b.b_field,
        Modifier::AB => a.a_field < b.b_field,
        Modifier::BA => a.b_field < b.a_field,
        Modifier::F | Modifier::I => {
            a.a_field < b.a_field && a.b_field < b.b_field
        }
        Modifier::X => a.a_field < b.b_field && a.b_field < b.a_field,
    };
    // Increment PC twice if the condition holds, otherwise increment once
    let amt = if is_less_than { 2 } else { 1 };
    inputs.pq.push_back(
        offset(inputs.regs.current.idx, amt, inputs.core_size)?,
        inputs.warrior_id,
    )?;
    Ok(())
}

/// Implementation of the [`Opcode::Cmp`] and [`Opcode::Seq`] instructions
pub fn cmp_op(inputs: OpInputs) -> EmulatorResult<()> {
    // CMP compares the A-value to the B-value.  If the result of the
    // comparison is equal, the instruction after the next instruction
    // (PC + 2) is queued (skipping the next instruction).  Otherwise, the
    // the next instruction is queued (PC + 1).
    let a = inputs.regs.a;
    let b = inputs.regs.b;
    let is_equal = match inputs.regs.current.instr.modifier {
        Modifier::A => a.a_field == b.a_field,
        Modifier::B => a.b_field == b.b_field,
        Modifier::AB => a.a_field == b.b_field,
        Modifier::BA => a.b_field == b.a_field,
        Modifier::F => a.a_field == b.a_field && a.b_field == b.b_field,
        Modifier::X => a.a_field == b.b_field && a.b_field == b.a_field,
        Modifier::I => {
            a.instr == b.instr
                && a.a_field == b.a_field
                && a.b_field == b.b_field
        }
    };
    // Increment PC twice if the condition holds, otherwise increment once
    let amt = if is_equal { 2 } else { 1 };
    inputs.pq.push_back(
        offset(inputs.regs.current.idx, amt, inputs.core_size)?,
        inputs.warrior_id,
    )?;
    Ok(())
}

/// Implementation of the [`Opcode::Sne`] instruction
pub fn sne_op(inputs: OpInputs) -> EmulatorResult<()> {
    // SNE compares the A-value to the B-value.  If the result of the
    // comparison is not equal, the instruction after the next instruction
    // (PC + 2) is queued (skipping the next instruction).  Otherwise, the
    // next instruction is queued (PC + 1).
    let a = inputs.regs.a;
    let b = inputs.regs.b;
    let is_not_equal = match inputs.regs.current.instr.modifier {
        Modifier::A => a.a_field != b.a_field,
        Modifier::B => a.b_field != b.b_field,
        Modifier::AB => a.a_field != b.b_field,
        Modifier::BA => a.b_field != b.a_field,
        Modifier::F => a.a_field != b.a_field || a.b_field != b.b_field,
        Modifier::X => a.a_field != b.b_field || a.b_field != b.a_field,
        Modifier::I => {
            a.instr != b.instr
                || a.a_field != b.a_field
                || a.b_field != b.b_field
        }
    };
    // Increment PC twice if the condition holds, otherwise increment once
    let amt = if is_not_equal { 2 } else { 1 };
    inputs.pq.push_back(
        offset(inputs.regs.current.idx, amt, inputs.core_size)?,
        inputs.warrior_id,
    )?;
    Ok(())
}

/// Implementation of the [`Opcode::Nop`] instruction
pub fn nop_op(inputs: OpInputs) -> EmulatorResult<()> {
    // Increments and queues the PC but otherwise has no effect past operand
    // evaluation
    inputs.pq.push_back(
        offset(inputs.regs.current.idx, 1, inputs.core_size)?,
        inputs.warrior_id,
    )?;
    Ok(())
}

/// Implementation of the [`Opcode::Ldp`] instruction
pub fn ldp_op(inputs: OpInputs) -> EmulatorResult<()> {
    // Reads a value from the PSPACE, writing it into core memory
    //
    // LDP and STP are not defined in any ICWS standard.  This implementation
    // is based on pMARS's behavior.
    //
    // The source index uses one of the fields from the A instruction, taken
    // modulo pspace size.  This is not the field that MOV would use as a
    // source index, but rather the field that MOV would use as the source
    // value.  Each location in PSPACE stores a single field, so the multi-
    // field modifiers of (X, F, I) are not meaningful.  They are defined to
    // operate identically to B.
    //
    // Similar to source index, the destination is the same destination that
    // be written to by a MOV instruction using the same modifier.  Again,
    // X, F, and I are not meaningful, and behave like B.
    //
    // Further PSPACE notes:
    //
    // It is expected that the PSPACE is not cleared between rounds in a
    // battle, so a warrior may use information from one round to pick a
    // strategy in the next round.
    //
    // Multiple warriors can the same PSPACE.  Hypothetically, this could
    // be used in multi-warrior hills where multiple warriors with the same
    // author could have an advantage with communication.
    //
    // The value at index 0 is not shared between warriors with the same pin,
    // and it does not retain it's value between rounds.  Instead it's initial
    // value indicates the outcome of the previous round in this battle.
    //
    // The pspace address space is typically smaller than the core size, and
    // almost always a factor of the core size.  By convention, it's 1/16 the
    // size of the full core.  It's not required for the size to be a factor,
    // however if this isn't the case, simple assumptions break.
    //
    // For example the pspace locations x and x+1 will usually be adjacent
    // (modulo pspace size) except when pspace is not a factor of coresize
    // and x+1 = coresize = 0.
    // In general: (x % coresize) % pspace size != (x % pspace size) % coresize
    //
    // Queue PC + 1
    inputs.pq.push_back(
        offset(inputs.regs.current.idx, 1, inputs.core_size)?,
        inputs.warrior_id,
    )?;
    let a = inputs.regs.a;
    let b = inputs.regs.b;
    let source_index = match inputs.regs.current.instr.modifier {
        Modifier::A | Modifier::AB => a.a_field,
        Modifier::B
        | Modifier::BA
        | Modifier::F
        | Modifier::X
        | Modifier::I => a.b_field,
    };
    let value = inputs.pspace.read(source_index, inputs.warrior_id)?;
    match inputs.regs.current.instr.modifier {
        Modifier::A | Modifier::BA => {
            let target = inputs.core_get_mut(b.idx)?;
            target.a_field = value;
        }
        Modifier::B
        | Modifier::AB
        | Modifier::F
        | Modifier::X
        | Modifier::I => {
            let target = inputs.core_get_mut(b.idx)?;
            target.b_field = value;
        }
    };
    Ok(())
}

/// Implementation of the [`Opcode::Stp`] instruction
pub fn stp_op(inputs: OpInputs) -> EmulatorResult<()> {
    // Reads a value from the PSPACE, writing it into core memory
    //
    // LDP and STP are not defined in any ICWS standard.  This implementation
    // is based on pMARS's behavior.
    let a = inputs.regs.a;
    let b = inputs.regs.b;
    let source_value = match inputs.regs.current.instr.modifier {
        Modifier::A | Modifier::AB => {
            // A field of a operand
            a.a_field
        }
        Modifier::B
        | Modifier::BA
        | Modifier::F
        | Modifier::X
        | Modifier::I => {
            // B field of a operand
            a.b_field
        }
    };
    let pspace_dest_index = match inputs.regs.current.instr.modifier {
        Modifier::A | Modifier::BA => {
            // a field of b operand
            b.a_field
        }
        Modifier::B
        | Modifier::AB
        | Modifier::F
        | Modifier::X
        | Modifier::I => {
            // b field of b operand
            b.b_field
        }
    };

    inputs
        .pspace
        .write(pspace_dest_index, source_value, inputs.warrior_id)?;

    // Queue PC + 1
    inputs.pq.push_back(
        offset(inputs.regs.current.idx, 1, inputs.core_size)?,
        inputs.warrior_id,
    )?;
    Ok(())
}

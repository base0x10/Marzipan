/// Encoding and decoding methods for in-core representation
mod bytecode;
/// Core emulator instruction dispatch loop and [`crate::EmulatorCore`] trait
/// implementation
mod dispatch;
/// Logic for executing decoded instructions in the emulator core
mod emulation_operations;
/// Logic for evaluating instruction operands including predecrement and
/// postincrement core mutations
mod operands;
/// A FIFO queue with configurable maximum size
mod processes;
/// Structures to track warrior pin assignments and pspace memory values
mod pspace;

pub use dispatch::Emulator;

use crate::{
    emulator_core::{EmulatorError, EmulatorResult},
    CoreAddr,
};

/// evaluate a + offset with the right modulo-coresize arithmetic
fn offset(
    initial: CoreAddr,
    offset: i64,
    size: CoreAddr,
) -> EmulatorResult<CoreAddr> {
    let mut res = offset;
    while res < 0 {
        res = res.checked_add(i64::from(size)).ok_or(
            EmulatorError::InternalError(
                "impossible integer overflow while adding core size to a \
                 negative offset",
            ),
        )?;
    }
    let sum = res.checked_add(i64::from(initial)).ok_or(
        EmulatorError::InternalError(
            "impossible integer overflow adding normalized offset to initial \
             value",
        ),
    )?;
    let normalized = sum.checked_rem(i64::from(size)).ok_or(
        EmulatorError::InternalError(
            "impossible division by zero when dividing by core size",
        ),
    )?;
    CoreAddr::try_from(normalized).map_or(
        Err(EmulatorError::InternalError(
            "Error converting normalized offset in i64 to CoreAddr u32",
        )),
        Ok,
    )
}

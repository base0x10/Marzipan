use alloc::fmt;

use crate::{BytecodeInstructionIdentifier, CoreAddr};

/// Result type shared by emulator implementations
pub type EmulatorResult<T> = core::result::Result<T, EmulatorError>;

/// Interface for low-level interactions with an emulator implementation.
///
/// This API exposes the primitives required to setup, modify, and run an
/// emulator.  A higher level API is required for things like rounds, warrior
/// loading, battles, or PSPACE maintenance.
///
/// [`CoreAddr`] values are transparent, meaning that values like 0 and 15 have
/// their standard meaning, but [`BytecodeInstructionIdentifier`]s are opaque.
/// Except with a matching `bytecode_format`, the conversions with redcode types
/// are only valid for the duration of the emulator object.
///
/// The [`BytecodeInstructionIdentifier`] type allows transparent copies and
/// blind mutations on core and warrior data without roundtrip conversions to
/// redcode types.  This is useful for optimizers, evolvers, and save/restore
/// states. For other uses, the converted values are more meaningful.
pub trait EmulatorCore {
    /// Execute the next instruction for a warrior, returning its address.
    /// Returns None for a warrior id with no active processes.
    ///
    /// # Errors
    ///
    /// Returns [`EmulatorError`]s for invalid inputs or in the event of an
    /// internal error.
    fn step(&mut self, warrior_id: u64) -> EmulatorResult<Option<CoreAddr>>;

    /// Execute up to a number of cycles, or until the count of active warriors
    /// reaches a value.  Returns the number of cycles executed.  
    ///
    /// A cycle may execute more than one core instruction.  Each active
    /// warrior executes one instruction per cycle.
    ///
    /// # Errors
    ///
    /// Returns [`EmulatorError`]s for invalid inputs or in the event of an
    /// internal error.
    fn run(
        &mut self,
        cycles: u64,
        warriors_remaining: u64,
    ) -> EmulatorResult<u64>;

    /// Query per-core settings such as `bytecode_format` and `core_size`.
    ///
    /// [`EmulatorCore`]s are classified by [`CoreSettings`] which
    /// aren't expected to be configurable through the [`EmulatorCore`] trait.
    /// Users of [`EmulatorCore`] might need to configure a new emulator or
    /// dispatch to different emulators depending on the required settings.
    fn core_settings(self) -> CoreSettings;

    /// Query the value stored at an address in the core.
    ///
    /// # Errors
    ///
    /// Returns [`EmulatorError`]s in the event of an internal error.
    /// Implementations may return an error if addr is not a valid core
    /// address from 0 to `core_size - 1`.
    fn read_core(
        &self,
        addr: CoreAddr,
    ) -> EmulatorResult<(BytecodeInstructionIdentifier, CoreAddr, CoreAddr)>;

    /// Modify the value stored at an address in the core.
    ///
    /// Field values and `insn` should be valid based on the current emulator
    /// configuration. E.g. `addr`, `a_field`, and `b_field` should be
    /// between `0` and `core_size-1`.
    ///
    /// # Errors
    ///
    /// Returns [`EmulatorError`]s for invalid inputs or in the event of an
    /// internal error.  Implementations may document if parameters are checked
    /// or the the core is silently corrupted.  
    fn write_core(
        &mut self,
        addr: CoreAddr,
        insn: BytecodeInstructionIdentifier,
        a_field: CoreAddr,
        b_field: CoreAddr,
    ) -> EmulatorResult<()>;

    /// Read a value from the PSPACE owned by some warrior.
    ///
    /// PSPACE support, allocations, and PIN assignments are defined by the
    /// implementation.
    ///
    /// # Errors
    ///
    /// May return [`EmulatorError`]s for invalid inputs or in the event of an
    /// internal error.  Implementations may document if parameters are checked,
    /// or if invalid values may result in unpredictable results.
    fn read_pspace(
        &self,
        warrior_id: u64,
        addr: CoreAddr,
    ) -> EmulatorResult<CoreAddr>;

    /// Write a value to the PSPACE owned by some warrior.
    ///
    /// Aside from the special address of zero, warriors that share a pin
    /// read from and write to the same PSPACE.
    ///
    /// PSPACE support, allocations, and PIN assignments are defined by the
    /// implementation.
    ///
    /// # Errors
    ///
    /// Returns [`EmulatorError`]s for invalid inputs or in the event of an
    /// internal error.  Implementations may document if parameters are checked
    /// or if invalid values are silently written.  This may result in
    /// unpredictable behavior or later errors.
    fn write_pspace(
        &mut self,
        warrior_id: u64,
        addr: CoreAddr,
        value: CoreAddr,
    ) -> EmulatorResult<()>;

    /// Removes any state associated with the core.  Writes the new values to
    /// the entire core.  All observable state is removed including process
    /// queues, partial-cycle state, and all PSPACE mapping and values.  
    ///
    /// # Errors
    ///
    /// Returns an [`EmulatorError::InvalidParam`] if any part of the new
    /// initial instruction is invalid for the core settings active for this
    /// emulator.
    fn reset_core(
        &mut self,
        initial_instr: BytecodeInstructionIdentifier,
        initial_a: CoreAddr,
        initial_b: CoreAddr,
    ) -> EmulatorResult<()>;

    /// Returns the set of warriors with non-empty process queues.
    fn active_warrior_set(&self) -> Vec<u64>;

    /// Returns a copy of the process queue for a warrior.  This will be empty
    /// for inactive warriors.  Otherwise the next process to execute is first.
    ///
    /// # Errors
    ///
    /// Returns [`EmulatorError`]s for invalid inputs or in the event of an
    /// internal error.
    fn read_process_queue(
        &self,
        warrior_id: u64,
    ) -> EmulatorResult<Vec<CoreAddr>>;

    /// Replaces the warriors current processes with the values in the input.
    ///
    /// Emulators my implement process queues in way that make in-place
    /// modification difficult.  As a result, this is the only generic method
    /// to give a warrior a process queue, kill a warrior, or modify its
    /// process queue.
    ///
    /// Values in the process queue should be valid core addresses in the range
    /// between 0 and `core_size - 1`.
    ///
    /// # Errors
    ///
    /// May return [`EmulatorError`]s for invalid inputs or in the event of an
    /// internal error.  Implementations may document if parameters are checked
    /// or if invalid values are silently written.  This may result in
    /// unpredictable behavior or later errors.
    fn replace_process_queue(
        &mut self,
        warrior_id: u64,
        process_queue: &[CoreAddr],
    ) -> EmulatorResult<()>;

    /// Convert this emulator's bytecode representation to a redcode Instruction
    ///
    /// # Errors
    ///
    /// Returns an [`EmulatorError::InvalidParam`] if `bytecode_instr` is not a
    /// valid encoded bytecode instruction from this emulator core. This may
    /// be the case if using bytecode from incompatible emulators.
    fn bytecode_to_rc(
        &self,
        bytecode_instr: BytecodeInstructionIdentifier,
    ) -> EmulatorResult<redcode::Instruction>;

    /// Convert a redcode instruction to this emulator's bytecode representation
    fn rc_to_bytecode(
        &self,
        redcode_instr: redcode::Instruction,
    ) -> BytecodeInstructionIdentifier;
}

/// Configurations applied to an emulator.  
///
/// These are typically configured when an emulator is constructed and
/// static through the lifetime of an emulator object.  
pub struct CoreSettings {
    /// Number of addresses in the core.  All fields are modulo `core_size`
    pub core_size: u64,

    /// Number of addresses allocated to each PSPACE.
    ///
    /// Warriors might not be allocated a PSPACE unless a PIN is assigned.
    /// Warriors sharing a PIN always share a PSPACE except for the special
    /// address at 0.  Emulators with a 0 `pspace_size` may not support any
    /// PSPACE instructions.  
    pub pspace_size: u64,

    /// The number of warriors supported by the core.  The valid range for
    /// warrior IDs is also defined from 0 to `warriors - 1`.  
    ///
    /// There is no notion of loaded warriors, so one emulator implementation
    /// can be substituted with another that supports more warriors.  Warriors
    /// without active processes have no observable effect on the core.
    pub warriors: u64,

    /// Defines the maximum size of the process queue for each warrior.
    ///
    /// After the process queue reaches this size, additional SPL instructions
    /// will only queue the first value (PC + 1).
    /// <https://corewar.co.uk/standards/icws94.htm#5.5.14>
    pub processes: u64,

    /// A string identifying the format and version used by
    /// [`BytecodeInstructionIdentifier`]s.
    ///
    /// When two emulators or any two systems agree on the same non-empty
    /// `bytecode_format` string, values from one can be used with another.
    /// This is true between processes, languages, machines, decades, etc.
    ///
    /// The `None` case and empty string identify when a bytecode value can
    /// only be used with the emulator that produced it.  The emulator may
    /// come up with encodings on the fly, and it might not be possible to
    /// convert a bytecode identifier value to redcode outside of the lifetime
    /// of the specific instance of the emulator.
    ///
    /// The values for bytecode instruction identifiers are unsigned 32 bit
    /// numbers.  To avoid encoding and endianness issues, they should be
    /// represented however positive numbers are typically represented, not
    /// converted into bytes/bits/signed integers.  The value is
    /// large enough for space inefficient bytecode formats, or redcode
    /// extensions that require 20-fold more instructions.
    ///
    /// By convention, the string is formatted as "url::package::name::version"
    /// e.g. "base0x10.com::marzipan-core::NurseryOptimized::0.0.1".
    /// Url and package prevent accidental future incompatibilities.  Package
    /// and version make it safe to rollout changes incrementally without
    /// coordinated package upgrades, database migrations, or server downtime.
    ///
    /// Url may be any url owned by the author, including github repo. Even
    /// when not following this convention, implementations should be careful
    /// to change this string whenever the encoding changes.
    pub bytecode_format: Option<&'static str>,
}

/// Possible error kinds for operations on emulator implementations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmulatorError {
    /// Out of range or otherwise illegal inputs
    InvalidParam(&'static str),

    /// Requested operation is not valid for this implementation
    UnsupportedFeature(&'static str),

    /// Implementation bug.
    ///
    /// If this ever shows up, please create an issue:
    /// <https://https://github.com/base0x10/Marzipan/issues/new>
    InternalError(&'static str),

    /// Not yet implemented functionality.
    ///
    /// If this ever shows up in a ease, please create an issue:
    /// <https://https://github.com/base0x10/Marzipan/issues/new>
    UnimplementedError(&'static str),
}

impl fmt::Display for EmulatorError {
    #[allow(clippy::pattern_type_mismatch)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidParam(msg) => {
                write!(f, "invalid parameter value for EmulatorCore function")?;
                write!(f, "{msg}")
            }
            Self::InternalError(msg) => {
                write!(f, "internal emulator implementation error")?;
                write!(f, "{msg}")?;
                write!(f, "this is a bug")?;
                write!(f, "we would appreciate a bug report: https://https://github.com/base0x10/Marzipan/issues/new")
            }
            Self::UnsupportedFeature(msg) => {
                write!(
                    f,
                    "requested operation is not valid for this implementation"
                )?;
                write!(f, "{msg}")
            }
            Self::UnimplementedError(msg) => {
                write!(f, "not yet implemented functionality")?;
                write!(f, "{msg}")?;
                write!(
                    f,
                    "if this error shows up in released code, it's a bug"
                )?;
                write!(f, "we would appreciate a bug report: https://https://github.com/base0x10/Marzipan/issues/new")
            }
        }
    }
}

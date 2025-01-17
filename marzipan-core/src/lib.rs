//! # Marzipan-Core
//!
//! Marzipan-Core emulates the Redcode assembly programs used the CoreWar
//! programming game.  It is part of Marzipan, a collection of related tools
//! for CoreWar.
//!
//! ## Usage
//!
//! Marzipan-Core provides the trait [`EmulatorCore`] to interact with
//! different emulator implementations.  [`emulators`] contains implementations
//! with various supported features.
//!
//! [`EmulatorCore`] is a low-level interface with primitives for launching an
//! emulator or accessing the emulator state.  It isn't a fully featured system
//! like pMARS.
//!
//! ```rust
//! # use redcode::*;
//! # use marzipan_core::emulators::generic_emulator;
//! # use crate::marzipan_core::EmulatorCore;
//! # let core_size = 8000;
//! # let pspace_size = 500;
//! # let warriors = 2;
//! # let process = 80_000;
//! # let pspace_map = vec![(0, 0), (1, 1)];
//! // This example uses generic_emulator. Each emulator is constructed differently
//! let mut emulator = generic_emulator::Emulator::new(
//!     core_size,
//!     pspace_size,
//!     warriors,
//!     process,
//! )
//! .unwrap();
//! // Other emulator implementations might have a static pspace or even none at all
//! emulator.initialize_pspace(&pspace_map);
//!
//! let dwarf = vec![
//!     // Omitted for brevity
//!     // Add.AB #4, $3
//! #    CompleteInstruction{
//! #        instr: Instruction {
//! #            opcode: Opcode::Add,
//! #            modifier: Modifier::AB,
//! #            a_addr_mode: AddrMode::Immediate,
//! #            b_addr_mode: AddrMode::Direct
//! #        },
//! #        a_field: 4,
//! #        b_field: 3
//! #    },
//!     // MOV.I $2, @2
//! #    CompleteInstruction{
//! #        instr: Instruction {
//! #            opcode: Opcode::Mov,
//! #            modifier: Modifier::A,
//! #            a_addr_mode: AddrMode::Direct,
//! #            b_addr_mode: AddrMode::IndirectB
//! #        },
//! #        a_field: 2,
//! #        b_field: 2
//! #    },
//!     // JMP.B $-2, $0
//! #    CompleteInstruction{
//! #        instr: Instruction {
//! #            opcode: Opcode::Jmp,
//! #            modifier: Modifier::A,
//! #            a_addr_mode: AddrMode::Direct,
//! #            b_addr_mode: AddrMode::Direct
//! #        },
//! #        a_field: 8000 - 2,
//! #        b_field: 3
//! #    },
//! ];
//!
//! // Load a dwarf at address 0 for warrior 0
//! emulator.replace_process_queue(0, &[0]);
//! for (offset, instruction) in dwarf.iter().enumerate() {
//!     let bytecode = emulator.rc_to_bytecode(instruction.instr);
//!     emulator.write_core((0 + offset) as u32, bytecode, instruction.a_field, instruction.b_field);
//! }
//!
//!
//! // Load a dwarf at address 100 for warrior 1
//! emulator.replace_process_queue(1, &[100]);
//! for (offset, instruction) in dwarf.iter().enumerate() {
//!     let bytecode = emulator.rc_to_bytecode(instruction.instr);
//!     emulator.write_core((100 + offset) as u32, bytecode, instruction.a_field, instruction.b_field);
//! }
//!
//! // run for 80,000 cycles, or until only one warrior remains
//! let cycles_run = emulator.run(80_000, 1);
//! assert_eq!(cycles_run, Ok(80_000));
//! ```
//!
//! ## Emulators
//!
//! Each emulator defines its own methods for constructions or configuration.
//! Once constructed, [`EmulatorCore`] provides a common interface to execute
//! Redcode programs.
//!
//! Most emulators other than [`emulators::generic_emulator`] define their core
//! size and other settings with associated constants.  These emulators are
//! faster, but emulators with different settings will have different types.
//! Programs generally need to write code for each configuration it plans to use
//! with a `const` [`CoreSettings`] and constructor.
//!
//! #### `GenericEmulator`
//!
//! `GenericEmulator` is the reference implementation.  The emulation code
//! attempts to be a direct translation of the (partial) CoreWar specification.
//! It should be obviously correct, easy to read and debug, and portable.
// TODO(jespy) Add a section about pmars verification tests
// TODO(jespy) Add benchmarked performance relative to pmars
//
// #### `NurseryEmulator`
//
// TODO(jespy) this
//
// `NurseryEmulator` is an optimized implementation that runs a single warrior
// alone in the core.  Its intended purpose is to identify _stillborn_
// warriors that self-terminate quickly.  This may be a helpful pruning step
// for evolvers or optimizers that generate many random, low quality warriors.
//
// Core size and supported instructions are specified at compile time.
//! ## Bytecode Formats
//!
//! Virtual machines and emulators often execute a custom bytecode.  This
//! approach sits somewhere between native execution and direct interpretation.
//! Marzipan-Core allows emulators to define and share custom bytecode formats.
//! This allows greater design flexibility and more optimizations.
//!
//! [`EmulatorCore`] provides the same interface to emulators using different
//! bytecode formats.  The following types are used to interact with Redcode
//! values without decoding them back into Redcode.
//!
//!  * [`BytecodeInstructionIdentifier`] is an opaque type backed by a `u32`.
//!    Each emulator provides encoding/decoding functions which guarantee a
//!    1-to-1 mapping with the non-field parts of a Redcode instruction.
//!  * [`CoreAddr`] is an alias for `u32`, and emulators should support values
//!    from 0 (first address) to the core size minus one (last address).
//!
//! Emulators don't need to use these types internally.  However most emulators
//! use `u32`s or smaller integer types.  This allows moves and copies to be
//! performed without an intermediate Redcode encoding/decoding.  This is
//! particularly useful for implementing evolvers, optimizers, and debuggers
//! with save-states.
//!
//! `bytecode_format` in [`CoreSettings`] allows emulators to identify
//! compatible internal representations.  By default, values of
//! [`BytecodeInstructionIdentifier`] are only valid for use with the object
//! that produced them.  If two emulators share a `bytecode_format`, bytecode
//! from one may always be used with another.  `bytecode_format` should use
//! a version so that bytecode can be safely persisted or shared between
//! different machines running different software.
//!
//! ## MARS
//!
//! The term MARS (Memory Array Redcode Simulator) describes a fully featured
//! emulator like pMARS (the defacto standard emulator) or exhaust.
//! The emulators in Marzipan-Core are not fully featured MARSs, but they could
//! be used as part of a MARS.
//!
//! MARSs do not just emulate instructions in the virtual core.  They
//! also parse warriors and configurations, setup the virtual core for each
//! round, and compose a number of rounds into a battle.  Because of this,
//! emulators in this crate are not MARSs.
//!
//! #### marzipan-core limitations:
//!
//!  * [`EmulatorCore`]s allow for querying but not configuring
//!    [`CoreSettings`].  This includes settings such as the size of the core,
//!    the number of warriors, or the supported instructions.  A MARS will need
//!    to know how to dispatch to or construct an [`EmulatorCore`] with the
//!    desired settings.
//!  * The [`EmulatorCore`] interface has no notion of warrior loading.  A MARS
//!    implementation will need to choose where to load warriors, place the
//!    instructions of the warriors into the core, and add the address of the
//!    first instruction to that warrior's process queue.
//!  * The [`EmulatorCore`] interface has no notion of battles.  A MARS
//!    implementation will need to setup the core before each round, interpret
//!    the state of the core after termination, and keep track of results.
//!  * Pspace persistance and setup is performed through the [`EmulatorCore`]
//!    interface, not managed by the emulator.  The user controls allocation and
//!    assignment of warriors to pspace PINs.  A MARS implementation is in
//!    control of pspace persistance as well as the special warrior-specific `0`
//!    location that typically represents the result of the previous round.
// Make clippy as annoying as possible
#![deny(
    // All typically enabled warnings are converted into errors
    // includes correctness, suspicious, style, complexity, and perf
    clippy::all,
    // Error on cargo lints
    clippy::cargo,
)]
#![warn(
    // Warn on pedantic and in-development nursery lints
    clippy::pedantic,
    clippy::nursery,
    // Lints from "restriction" group - enforce a consistent if arbitrary style
    clippy::alloc_instead_of_core,
    clippy::allow_attributes_without_reason,
    clippy::arithmetic_side_effects,
    clippy::unnecessary_cast,
    clippy::as_underscore,
    clippy::assertions_on_result_states,
    clippy::clone_on_ref_ptr,
    clippy::create_dir,
    clippy::dbg_macro,
    clippy::decimal_literal_representation,
    clippy::default_numeric_fallback,
    clippy::deref_by_slicing,
    clippy::empty_drop,
    clippy::empty_structs_with_brackets,
    clippy::exit,
    clippy::expect_used,
    clippy::filetype_is_file,
    clippy::float_arithmetic,
    clippy::fn_to_numeric_cast_any,
    clippy::format_push_string,
    clippy::get_unwrap,
    clippy::if_then_some_else_none,
    clippy::indexing_slicing,
    clippy::large_include_file,
    clippy::let_underscore_must_use,
    clippy::lossy_float_literal,
    clippy::map_err_ignore,
    clippy::mem_forget,
    clippy::missing_docs_in_private_items,
    clippy::missing_enforced_import_renames,
    clippy::mixed_read_write_in_expression,
    clippy::mod_module_files,
    clippy::multiple_inherent_impl,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::partial_pub_fields,
    clippy::pattern_type_mismatch,
    clippy::print_stderr,
    clippy::print_stdout,
    clippy::rc_buffer,
    clippy::rc_mutex,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::same_name_method,
    clippy::unseparated_literal_suffix,
    clippy::shadow_unrelated,
    clippy::std_instead_of_alloc,
    clippy::std_instead_of_core,
    clippy::str_to_string,
    clippy::string_slice,
    clippy::string_to_string,
    clippy::suspicious_xor_used_as_pow,
    clippy::todo,
    clippy::try_err,
    clippy::undocumented_unsafe_blocks,
    clippy::unimplemented,
    clippy::unnecessary_safety_comment,
    clippy::unnecessary_safety_doc,
    clippy::unnecessary_self_imports,
    clippy::unneeded_field_pattern,
    clippy::unreachable,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    clippy::use_debug,
    clippy::verbose_file_reads,
)]
#![allow(
    clippy::needless_pass_by_value,
    reason = "Internal Compiler Error bug workaround: https://github.com/rust-lang/rust-clippy/issues/10344"
)]

// Use no-std collections
extern crate alloc;

/// Contains the [`EmulatorCore`] trait for low-level emulator interactions
mod emulator_core;
pub use emulator_core::{
    CoreSettings, EmulatorCore, EmulatorError, EmulatorResult,
};

/// An offset into an emulator core, valid from 0 to `core_size - 1` inclusive.
///
/// [`EmulatorCore`]s use this type when referencing address, field, and values,
/// though they may use a different type internally.
pub type CoreAddr = redcode::FieldValue;

/// An opaque identifier for a redcode instruction.
///
/// There is a 1-1 mapping provided by each emulator core with redcode types,
/// but only it is only valid for emulators with the same `bytecode_format`.
/// See also [`CoreSettings`].
pub type BytecodeInstructionIdentifier = u32;

/// Emulator implementations.
pub mod emulators;

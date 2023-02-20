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
    // FPs for this lint
    // https://github.com/rust-lang/rust-clippy/issues/10377
    // clippy::allow_attributes_without_reason,
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
// require reason="..." #[allow(...)]
#![feature(lint_reasons)]

//! Parsing functions for CoreWar's redcode syntax.
//!
//! The exact grammar supported for loadfile parsing is specified in
//! [`loadfile_grammar.md`](../loadfile_grammars.md).

/// Parsing functions for warriors and individual instructions either in '88 or
/// '94 loadfile format
mod loadfile_parser;
pub use loadfile_parser::{parse, parse_instr};

/// Internal functions which evaluate a the content of a single line, without
/// consuming any newline characters
mod line_parser;

/// Internal functions which parse well defined primitives from the redcode
/// grammar.
mod loadfile_primitives;

/// Module that exposes nom parsing functions for recoginizing
/// and mapping raw redcode language elements.
///
/// A more or less strict redcode lexer/parser can be written from these elements
pub mod atomics;

/// parser for loadfiles, a stricter specification of redcode
/// that does not require preprocessing, only parsing and loading
/// into the core.  
pub mod loadfile;

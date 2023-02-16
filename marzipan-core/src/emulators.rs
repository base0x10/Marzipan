/// [`generic_emulator::Emulator`] is the reference implementation.  The
/// emulation code attempts to be a direct translation of the (partial) CoreWar
/// specification. It should be obviously correct, easy to read and debug, and
/// portable.
///
/// This emulator doesn't rely on `const` for core size or other
/// settings.  It can be configured at runtime to run with arbitrary settings.
pub mod generic_emulator;

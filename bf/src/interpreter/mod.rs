mod cell;

#[allow(clippy::module_inception)]
mod interpreter;
pub use interpreter::Interpreter;

mod tape;
pub use tape::Tape;

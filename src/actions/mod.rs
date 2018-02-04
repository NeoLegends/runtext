use std::io;

pub mod command;

/// Represents an action to be executed upon a context transition.
pub trait Action {
    /// Asynchronously start executing the action.
    fn enter(&mut self) -> io::Result<()>;

    /// Synchronously stop executing the action.
    fn leave(&mut self) -> io::Result<()>;
}
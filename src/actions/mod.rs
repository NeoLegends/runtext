use std::io;

use futures::prelude::*;

pub mod command;

/// Represents an action to be executed upon a context transition.
pub trait Action {
    /// Asynchronously start executing the action.
    fn enter(&mut self) -> Box<Future<Item = (), Error = io::Error>>;

    /// Synchronously stop executing the action.
    fn leave(&mut self) -> Box<Future<Item = (), Error = io::Error>>;
}
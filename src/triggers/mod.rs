use std::io;

use futures::prelude::*;
use tokio_core::reactor::Handle;

pub mod wifi;

/// A context activity change
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Activity {
    /// The context is currently active.
    Active,

    /// The context is inactive.
    Inactive
}

/// A context-based evidence source.
pub trait Trigger {
    /// Start listening for the context and dispatch signals
    /// whenever the context is entered or left.
    fn listen(&mut self, handle: Handle) -> Box<Stream<Item = Activity, Error = io::Error>>;
}
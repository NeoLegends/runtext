#[cfg_attr(target_os = "macos", path = "macos.rs")]
mod imp;

pub const TRIGGER_NAME: &'static str = "wifi";

pub use self::imp::*;

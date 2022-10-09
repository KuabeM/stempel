//! Errors for stempel.

pub use color_eyre::eyre::{bail, eyre, Error, Result, WrapErr};
use thiserror::Error;

#[derive(Debug, Error)]
pub struct UsageError(pub String);

impl std::fmt::Display for UsageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[macro_export]
macro_rules! usage_err {
    ($($arg:tt)*) => {
        UsageError(format!($($arg)*))
    };
}

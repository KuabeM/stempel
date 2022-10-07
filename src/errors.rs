//! Errors for stempel.

use thiserror::Error;

pub type Result<T, E = TimeErr> = std::result::Result<T, E>;

#[derive(Error, Debug)]
pub enum TimeErr {
    #[error("Command failed: {0}")]
    CmdFail(String),
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Read failed: {0}")]
    Read(String),
    #[error("Write failed: {0}")]
    Write(String),
    #[error("Parsing failed: {0}")]
    Parse(String),
    #[error("Io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serde: {0}")]
    Serde(#[from] serde_json::Error),
}

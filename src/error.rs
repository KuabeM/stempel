use thiserror::Error;

#[derive(Error, Debug, Eq, PartialEq)]
pub enum TimeError {
    #[error("failed to de/serialize {0}")]
    SerializationError(String),
    #[error("failed to handle file {0}")]
    IoError(String),
}

use thiserror::Error;

#[derive(Error, Debug, Eq, PartialEq)]
pub enum TimeError {
    #[error("failed to serialize {0}")]
    SerializationError(String),
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum TimeError {
    #[error("failed to serialize {0}")]
    SerializationError(String),
    #[error("File not found {0}")]
    FileNotFound(String),
}


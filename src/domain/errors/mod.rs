use thiserror::Error;

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Message queue error: {0}")]
    MessageQueueError(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Processing error: {0}")]
    ProcessingError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

impl From<lapin::Error> for DomainError {
    fn from(err: lapin::Error) -> Self {
        DomainError::MessageQueueError(err.to_string())
    }
}

impl From<serde_json::Error> for DomainError {
    fn from(err: serde_json::Error) -> Self {
        DomainError::SerializationError(err.to_string())
    }
}

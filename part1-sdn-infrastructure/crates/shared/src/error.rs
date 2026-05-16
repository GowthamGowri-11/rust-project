use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("OpenFlow protocol error: {0}")]
    Protocol(String),

    #[error("Switch error: {0}")]
    Switch(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Timeout error: {0}")]
    Timeout(String),

    #[error("Invalid message: {0}")]
    InvalidMessage(String),

    #[error("Not found: {0}")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, Error>;

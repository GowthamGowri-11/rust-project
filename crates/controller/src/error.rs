use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum ControllerError {
    #[error("Switch not found: {0}")]
    SwitchNotFound(String),

    #[error("Flow not found: {0}")]
    FlowNotFound(Uuid),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Flow installation failed: {0}")]
    FlowInstallationFailed(String),

    #[error("OpenFlow protocol error: {0}")]
    ProtocolError(String),

    #[error("Invalid flow rule: {0}")]
    InvalidFlowRule(String),

    #[error("Maximum connections reached")]
    TooManyConnections,

    #[error("Message too large: {0} bytes")]
    MessageTooLarge(usize),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, ControllerError>;

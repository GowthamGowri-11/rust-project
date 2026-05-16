use thiserror::Error;

#[derive(Error, Debug)]
pub enum ResilienceError {
    #[error("Recovery failed: {0}")]
    RecoveryFailed(String),

    #[error("No backup path available for {0}")]
    NoBackupPath(String),

    #[error("Failure detection error: {0}")]
    DetectionError(String),
}

pub type Result<T> = std::result::Result<T, ResilienceError>;

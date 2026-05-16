use thiserror::Error;

#[derive(Error, Debug)]
pub enum MlError {
    #[error("Model not loaded: {0}")]
    ModelNotLoaded(String),

    #[error("Model loading failed: {0}")]
    ModelLoadFailed(String),

    #[error("Inference failed: {0}")]
    InferenceFailed(String),

    #[error("Invalid input shape: expected {expected}, got {actual}")]
    InvalidInputShape { expected: String, actual: String },

    #[error("Invalid model format: {0}")]
    InvalidModelFormat(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, MlError>;

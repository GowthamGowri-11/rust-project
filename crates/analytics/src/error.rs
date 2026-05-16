use thiserror::Error;

#[derive(Error, Debug)]
pub enum AnalyticsError {
    #[error("Analysis failed: {0}")]
    AnalysisFailed(String),

    #[error("Insufficient data: {0}")]
    InsufficientData(String),

    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),
}

pub type Result<T> = std::result::Result<T, AnalyticsError>;

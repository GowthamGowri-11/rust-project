use thiserror::Error;

#[derive(Error, Debug)]
pub enum BenchmarkError {
    #[error("Benchmark failed: {0}")]
    BenchmarkFailed(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Measurement error: {0}")]
    MeasurementError(String),
}

pub type Result<T> = std::result::Result<T, BenchmarkError>;

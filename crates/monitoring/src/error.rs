use thiserror::Error;

#[derive(Error, Debug)]
pub enum MonitoringError {
    #[error("eBPF probe failed: {0}")]
    EbpfProbeFailed(String),

    #[error("Metric collection failed: {0}")]
    MetricCollectionFailed(String),

    #[error("Invalid interface: {0}")]
    InvalidInterface(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, MonitoringError>;

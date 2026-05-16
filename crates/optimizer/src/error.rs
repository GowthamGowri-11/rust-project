use thiserror::Error;

#[derive(Error, Debug)]
pub enum OptimizerError {
    #[error("No path found between {src} and {dst}")]
    NoPathFound { src: String, dst: String },

    #[error("Optimization failed: {0}")]
    OptimizationFailed(String),

    #[error("Invalid topology: {0}")]
    InvalidTopology(String),
}

pub type Result<T> = std::result::Result<T, OptimizerError>;

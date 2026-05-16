use thiserror::Error;

#[derive(Error, Debug)]
pub enum PolicyError {
    #[error("Policy validation failed: {0}")]
    ValidationFailed(String),

    #[error("Policy not found: {0}")]
    PolicyNotFound(String),

    #[error("Rule conflict: {0}")]
    RuleConflict(String),

    #[error("Invalid policy: {0}")]
    InvalidPolicy(String),

    #[error("SLA violation: {0}")]
    SlaViolation(String),
}

pub type Result<T> = std::result::Result<T, PolicyError>;

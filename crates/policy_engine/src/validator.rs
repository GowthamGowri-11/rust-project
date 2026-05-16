use crate::error::Result;
use crate::policies::Policy;
use tracing::debug;

/// Policy validator
pub struct PolicyValidator;

impl PolicyValidator {
    pub fn new() -> Self {
        Self
    }

    /// Validate policy
    pub fn validate(&self, policy: &Policy) -> Result<()> {
        debug!("Validating policy: {}", policy.name);

        // Check if policy has at least one rule
        if policy.rules.is_empty() {
            return Err(crate::error::PolicyError::ValidationFailed(
                "Policy must have at least one rule".to_string(),
            ));
        }

        // Validate constraints
        self.validate_constraints(&policy.constraints)?;

        Ok(())
    }

    fn validate_constraints(&self, constraints: &crate::policies::PolicyConstraints) -> Result<()> {
        // Validate latency constraint
        if let Some(latency) = constraints.max_latency_ms {
            if latency <= 0.0 {
                return Err(crate::error::PolicyError::ValidationFailed(
                    "Max latency must be positive".to_string(),
                ));
            }
        }

        // Validate bandwidth constraint
        if let Some(bandwidth) = constraints.min_bandwidth_bps {
            if bandwidth == 0 {
                return Err(crate::error::PolicyError::ValidationFailed(
                    "Min bandwidth must be positive".to_string(),
                ));
            }
        }

        Ok(())
    }
}

impl Default for PolicyValidator {
    fn default() -> Self {
        Self::new()
    }
}

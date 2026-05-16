use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureEvent {
    pub event_id: String,
    pub failure_type: FailureType,
    pub affected_component: String,
    pub timestamp: DateTime<Utc>,
    pub severity: FailureSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailureType {
    LinkDown,
    SwitchDown,
    HighLatency,
    PacketLoss,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailureSeverity {
    Minor,
    Major,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryAction {
    pub action_id: String,
    pub action_type: RecoveryType,
    pub target: String,
    pub executed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryType {
    Reroute { backup_path: Vec<String> },
    Failover { backup_component: String },
    Throttle { rate_limit: u64 },
}

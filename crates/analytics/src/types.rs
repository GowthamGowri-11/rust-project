use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficPattern {
    pub pattern_id: String,
    pub flow_count: u32,
    pub avg_packet_size: f64,
    pub peak_bandwidth: u64,
    pub duration_sec: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CongestionReport {
    pub link_id: String,
    pub severity: CongestionSeverity,
    pub utilization: f64,
    pub packet_loss: f64,
    pub latency_ms: f64,
    pub affected_flows: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CongestionSeverity {
    Low,
    Medium,
    High,
    Critical,
}

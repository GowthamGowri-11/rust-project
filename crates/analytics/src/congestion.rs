use serde::{Deserialize, Serialize};

/// Congestion analysis engine
pub struct CongestionAnalyzer {
    threshold_utilization: f64,
    threshold_latency: f64,
    threshold_loss: f64,
}

impl CongestionAnalyzer {
    pub fn new() -> Self {
        Self {
            threshold_utilization: 0.8,  // 80% utilization
            threshold_latency: 100.0,     // 100ms latency
            threshold_loss: 1.0,          // 1% packet loss
        }
    }

    pub fn with_thresholds(utilization: f64, latency: f64, loss: f64) -> Self {
        Self {
            threshold_utilization: utilization,
            threshold_latency: latency,
            threshold_loss: loss,
        }
    }

    /// Calculate congestion score (0.0 - 1.0)
    pub fn calculate_score(&self, metrics: &LinkMetrics) -> f64 {
        let util_score = (metrics.utilization / self.threshold_utilization).min(1.0);
        let latency_score = (metrics.latency_ms / self.threshold_latency).min(1.0);
        let loss_score = (metrics.packet_loss / self.threshold_loss).min(1.0);

        // Weighted average
        (util_score * 0.5) + (latency_score * 0.3) + (loss_score * 0.2)
    }

    /// Determine congestion severity
    pub fn determine_severity(&self, score: f64) -> CongestionSeverity {
        match score {
            s if s < 0.3 => CongestionSeverity::None,
            s if s < 0.5 => CongestionSeverity::Low,
            s if s < 0.7 => CongestionSeverity::Medium,
            s if s < 0.9 => CongestionSeverity::High,
            _ => CongestionSeverity::Critical,
        }
    }

    /// Analyze link for congestion
    pub fn analyze_link(&self, metrics: &LinkMetrics) -> CongestionReport {
        let score = self.calculate_score(metrics);
        let severity = self.determine_severity(score);

        CongestionReport {
            link_id: metrics.link_id.clone(),
            score,
            severity,
            utilization: metrics.utilization,
            latency_ms: metrics.latency_ms,
            packet_loss: metrics.packet_loss,
            bandwidth_bps: metrics.bandwidth_bps,
            timestamp: chrono::Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkMetrics {
    pub link_id: String,
    pub utilization: f64,
    pub latency_ms: f64,
    pub packet_loss: f64,
    pub bandwidth_bps: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CongestionReport {
    pub link_id: String,
    pub score: f64,
    pub severity: CongestionSeverity,
    pub utilization: f64,
    pub latency_ms: f64,
    pub packet_loss: f64,
    pub bandwidth_bps: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum CongestionSeverity {
    None,
    Low,
    Medium,
    High,
    Critical,
}

impl Default for CongestionAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

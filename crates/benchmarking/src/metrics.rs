use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Benchmark metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkMetrics {
    pub latency: LatencyMetrics,
    pub throughput: ThroughputMetrics,
    pub packet_loss: PacketLossMetrics,
    pub duration: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyMetrics {
    pub min_ms: f64,
    pub max_ms: f64,
    pub avg_ms: f64,
    pub median_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub stddev_ms: f64,
}

impl LatencyMetrics {
    pub fn from_samples(samples: &[f64]) -> Self {
        if samples.is_empty() {
            return Self::default();
        }

        let mut sorted = samples.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let min = sorted[0];
        let max = sorted[sorted.len() - 1];
        let avg = samples.iter().sum::<f64>() / samples.len() as f64;
        let median = sorted[sorted.len() / 2];
        let p95 = sorted[(sorted.len() as f64 * 0.95) as usize];
        let p99 = sorted[(sorted.len() as f64 * 0.99) as usize];

        let variance = samples
            .iter()
            .map(|x| (x - avg).powi(2))
            .sum::<f64>() / samples.len() as f64;
        let stddev = variance.sqrt();

        Self {
            min_ms: min,
            max_ms: max,
            avg_ms: avg,
            median_ms: median,
            p95_ms: p95,
            p99_ms: p99,
            stddev_ms: stddev,
        }
    }
}

impl Default for LatencyMetrics {
    fn default() -> Self {
        Self {
            min_ms: 0.0,
            max_ms: 0.0,
            avg_ms: 0.0,
            median_ms: 0.0,
            p95_ms: 0.0,
            p99_ms: 0.0,
            stddev_ms: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputMetrics {
    pub total_bytes: u64,
    pub total_packets: u64,
    pub bytes_per_sec: f64,
    pub packets_per_sec: f64,
    pub bits_per_sec: f64,
}

impl ThroughputMetrics {
    pub fn calculate(total_bytes: u64, total_packets: u64, duration_secs: f64) -> Self {
        let bytes_per_sec = total_bytes as f64 / duration_secs;
        let packets_per_sec = total_packets as f64 / duration_secs;
        let bits_per_sec = bytes_per_sec * 8.0;

        Self {
            total_bytes,
            total_packets,
            bytes_per_sec,
            packets_per_sec,
            bits_per_sec,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketLossMetrics {
    pub sent: u64,
    pub received: u64,
    pub lost: u64,
    pub loss_rate: f64,
}

impl PacketLossMetrics {
    pub fn calculate(sent: u64, received: u64) -> Self {
        let lost = sent.saturating_sub(received);
        let loss_rate = if sent > 0 {
            (lost as f64 / sent as f64) * 100.0
        } else {
            0.0
        };

        Self {
            sent,
            received,
            lost,
            loss_rate,
        }
    }
}

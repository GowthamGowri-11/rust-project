use serde::{Deserialize, Serialize};

/// Feature extraction engine for ML
pub struct FeatureExtractor {
    window_size: usize,
}

impl FeatureExtractor {
    pub fn new(window_size: usize) -> Self {
        Self { window_size }
    }

    /// Extract features from traffic samples
    pub fn extract(&self, samples: &[TrafficSample]) -> TrafficFeatures {
        if samples.is_empty() {
            return TrafficFeatures::default();
        }

        let bandwidth_values: Vec<f64> = samples.iter().map(|s| s.bandwidth_bps as f64).collect();
        let latency_values: Vec<f64> = samples.iter().map(|s| s.latency_ms).collect();
        let loss_values: Vec<f64> = samples.iter().map(|s| s.packet_loss).collect();

        TrafficFeatures {
            // Statistical features
            avg_bandwidth: Self::mean(&bandwidth_values),
            std_bandwidth: Self::std_dev(&bandwidth_values),
            max_bandwidth: Self::max(&bandwidth_values),
            min_bandwidth: Self::min(&bandwidth_values),

            avg_latency: Self::mean(&latency_values),
            std_latency: Self::std_dev(&latency_values),
            max_latency: Self::max(&latency_values),
            min_latency: Self::min(&latency_values),

            avg_loss: Self::mean(&loss_values),
            max_loss: Self::max(&loss_values),

            // Temporal features
            bandwidth_trend: Self::calculate_trend(&bandwidth_values),
            latency_trend: Self::calculate_trend(&latency_values),

            // Derived features
            utilization: samples.last().map(|s| s.utilization).unwrap_or(0.0),
            flow_count: samples.last().map(|s| s.flow_count).unwrap_or(0),
            
            sample_count: samples.len(),
        }
    }

    /// Convert features to ML input vector
    pub fn to_vector(&self, features: &TrafficFeatures) -> Vec<f32> {
        vec![
            features.avg_bandwidth as f32,
            features.std_bandwidth as f32,
            features.max_bandwidth as f32,
            features.min_bandwidth as f32,
            features.avg_latency as f32,
            features.std_latency as f32,
            features.max_latency as f32,
            features.min_latency as f32,
            features.avg_loss as f32,
            features.max_loss as f32,
            features.bandwidth_trend as f32,
            features.latency_trend as f32,
            features.utilization as f32,
            features.flow_count as f32,
        ]
    }

    fn mean(values: &[f64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }
        values.iter().sum::<f64>() / values.len() as f64
    }

    fn std_dev(values: &[f64]) -> f64 {
        if values.len() < 2 {
            return 0.0;
        }
        let mean = Self::mean(values);
        let variance = values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / (values.len() - 1) as f64;
        variance.sqrt()
    }

    fn max(values: &[f64]) -> f64 {
        values.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
    }

    fn min(values: &[f64]) -> f64 {
        values.iter().cloned().fold(f64::INFINITY, f64::min)
    }

    fn calculate_trend(values: &[f64]) -> f64 {
        if values.len() < 2 {
            return 0.0;
        }
        let first_half = &values[..values.len() / 2];
        let second_half = &values[values.len() / 2..];
        Self::mean(second_half) - Self::mean(first_half)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficSample {
    pub bandwidth_bps: u64,
    pub latency_ms: f64,
    pub packet_loss: f64,
    pub utilization: f64,
    pub flow_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficFeatures {
    pub avg_bandwidth: f64,
    pub std_bandwidth: f64,
    pub max_bandwidth: f64,
    pub min_bandwidth: f64,
    pub avg_latency: f64,
    pub std_latency: f64,
    pub max_latency: f64,
    pub min_latency: f64,
    pub avg_loss: f64,
    pub max_loss: f64,
    pub bandwidth_trend: f64,
    pub latency_trend: f64,
    pub utilization: f64,
    pub flow_count: u32,
    pub sample_count: usize,
}

impl Default for TrafficFeatures {
    fn default() -> Self {
        Self {
            avg_bandwidth: 0.0,
            std_bandwidth: 0.0,
            max_bandwidth: 0.0,
            min_bandwidth: 0.0,
            avg_latency: 0.0,
            std_latency: 0.0,
            max_latency: 0.0,
            min_latency: 0.0,
            avg_loss: 0.0,
            max_loss: 0.0,
            bandwidth_trend: 0.0,
            latency_trend: 0.0,
            utilization: 0.0,
            flow_count: 0,
            sample_count: 0,
        }
    }
}

impl Default for FeatureExtractor {
    fn default() -> Self {
        Self::new(100)
    }
}

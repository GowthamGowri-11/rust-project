use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Traffic pattern detector
pub struct PatternDetector {
    pattern_cache: HashMap<String, DetectedPattern>,
}

impl PatternDetector {
    pub fn new() -> Self {
        Self {
            pattern_cache: HashMap::new(),
        }
    }

    /// Detect traffic patterns
    pub fn detect(&mut self, link_id: &str, samples: &[f64]) -> Option<DetectedPattern> {
        if samples.len() < 10 {
            return None;
        }

        let pattern_type = self.classify_pattern(samples);
        let periodicity = self.detect_periodicity(samples);
        let volatility = self.calculate_volatility(samples);

        let pattern = DetectedPattern {
            link_id: link_id.to_string(),
            pattern_type,
            periodicity,
            volatility,
            confidence: self.calculate_confidence(samples),
            sample_count: samples.len(),
        };

        self.pattern_cache.insert(link_id.to_string(), pattern.clone());
        Some(pattern)
    }

    fn classify_pattern(&self, samples: &[f64]) -> PatternType {
        let trend = self.calculate_linear_trend(samples);
        let variance = self.calculate_variance(samples);

        if trend.abs() < 0.1 && variance < 0.2 {
            PatternType::Stable
        } else if trend > 0.5 {
            PatternType::Increasing
        } else if trend < -0.5 {
            PatternType::Decreasing
        } else if variance > 0.5 {
            PatternType::Bursty
        } else {
            PatternType::Periodic
        }
    }

    fn detect_periodicity(&self, samples: &[f64]) -> Option<usize> {
        // Simple autocorrelation-based periodicity detection
        if samples.len() < 20 {
            return None;
        }

        let max_lag = samples.len() / 2;
        let mut best_lag = 0;
        let mut best_correlation = 0.0;

        for lag in 2..max_lag {
            let correlation = self.autocorrelation(samples, lag);
            if correlation > best_correlation {
                best_correlation = correlation;
                best_lag = lag;
            }
        }

        if best_correlation > 0.7 {
            Some(best_lag)
        } else {
            None
        }
    }

    fn autocorrelation(&self, samples: &[f64], lag: usize) -> f64 {
        if lag >= samples.len() {
            return 0.0;
        }

        let mean = samples.iter().sum::<f64>() / samples.len() as f64;
        let mut numerator = 0.0;
        let mut denominator = 0.0;

        for i in 0..(samples.len() - lag) {
            numerator += (samples[i] - mean) * (samples[i + lag] - mean);
        }

        for sample in samples {
            denominator += (sample - mean).powi(2);
        }

        if denominator == 0.0 {
            0.0
        } else {
            numerator / denominator
        }
    }

    fn calculate_volatility(&self, samples: &[f64]) -> f64 {
        if samples.len() < 2 {
            return 0.0;
        }

        let mean = samples.iter().sum::<f64>() / samples.len() as f64;
        let variance = samples.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / samples.len() as f64;
        
        variance.sqrt() / mean.max(1.0)
    }

    fn calculate_linear_trend(&self, samples: &[f64]) -> f64 {
        if samples.len() < 2 {
            return 0.0;
        }

        let n = samples.len() as f64;
        let x_mean = (n - 1.0) / 2.0;
        let y_mean = samples.iter().sum::<f64>() / n;

        let mut numerator = 0.0;
        let mut denominator = 0.0;

        for (i, &y) in samples.iter().enumerate() {
            let x = i as f64;
            numerator += (x - x_mean) * (y - y_mean);
            denominator += (x - x_mean).powi(2);
        }

        if denominator == 0.0 {
            0.0
        } else {
            numerator / denominator
        }
    }

    fn calculate_variance(&self, samples: &[f64]) -> f64 {
        if samples.len() < 2 {
            return 0.0;
        }

        let mean = samples.iter().sum::<f64>() / samples.len() as f64;
        samples.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / samples.len() as f64
    }

    fn calculate_confidence(&self, samples: &[f64]) -> f64 {
        // Confidence based on sample size and consistency
        let size_factor = (samples.len() as f64 / 100.0).min(1.0);
        let variance = self.calculate_variance(samples);
        let consistency_factor = 1.0 / (1.0 + variance);
        
        (size_factor + consistency_factor) / 2.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedPattern {
    pub link_id: String,
    pub pattern_type: PatternType,
    pub periodicity: Option<usize>,
    pub volatility: f64,
    pub confidence: f64,
    pub sample_count: usize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum PatternType {
    Stable,
    Increasing,
    Decreasing,
    Bursty,
    Periodic,
}

impl Default for PatternDetector {
    fn default() -> Self {
        Self::new()
    }
}

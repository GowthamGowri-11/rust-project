use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Jain's Fairness Index Calculator
/// 
/// Measures fairness of resource allocation across flows
/// Range: [0, 1] where 1 = perfectly fair
pub struct JainFairnessCalculator;

impl JainFairnessCalculator {
    /// Calculate Jain's Fairness Index
    /// 
    /// Formula: (Σxi)² / (n * Σxi²)
    /// where xi is the throughput of flow i, n is number of flows
    pub fn calculate(throughputs: &[f64]) -> Result<FairnessMetrics> {
        if throughputs.is_empty() {
            return Err(crate::error::BenchmarkError::InvalidConfig(
                "Cannot calculate fairness for empty throughput data".to_string(),
            ));
        }

        let n = throughputs.len() as f64;
        let sum: f64 = throughputs.iter().sum();
        let sum_squared: f64 = throughputs.iter().map(|x| x * x).sum();

        let jain_index = if sum_squared > 0.0 {
            (sum * sum) / (n * sum_squared)
        } else {
            0.0
        };

        let min = throughputs
            .iter()
            .copied()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        
        let max = throughputs
            .iter()
            .copied()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        
        let avg = sum / n;
        
        let variance = throughputs
            .iter()
            .map(|x| (x - avg).powi(2))
            .sum::<f64>() / n;
        
        let stddev = variance.sqrt();
        let coefficient_of_variation = if avg > 0.0 { stddev / avg } else { 0.0 };

        Ok(FairnessMetrics {
            jain_index,
            min_throughput: min,
            max_throughput: max,
            avg_throughput: avg,
            stddev_throughput: stddev,
            coefficient_of_variation,
            num_flows: throughputs.len(),
        })
    }

    /// Calculate fairness for multiple scenarios
    pub fn compare_scenarios(
        baseline: &[f64],
        optimized: &[f64],
    ) -> Result<FairnessComparison> {
        let baseline_metrics = Self::calculate(baseline)?;
        let optimized_metrics = Self::calculate(optimized)?;

        let jain_improvement = optimized_metrics.jain_index - baseline_metrics.jain_index;
        let cv_improvement = baseline_metrics.coefficient_of_variation
            - optimized_metrics.coefficient_of_variation;

        Ok(FairnessComparison {
            baseline: baseline_metrics,
            optimized: optimized_metrics,
            jain_improvement,
            cv_improvement,
        })
    }

    /// Interpret Jain's index
    pub fn interpret(jain_index: f64) -> FairnessLevel {
        if jain_index >= 0.95 {
            FairnessLevel::Excellent
        } else if jain_index >= 0.85 {
            FairnessLevel::Good
        } else if jain_index >= 0.70 {
            FairnessLevel::Fair
        } else if jain_index >= 0.50 {
            FairnessLevel::Poor
        } else {
            FairnessLevel::VeryPoor
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FairnessMetrics {
    pub jain_index: f64,
    pub min_throughput: f64,
    pub max_throughput: f64,
    pub avg_throughput: f64,
    pub stddev_throughput: f64,
    pub coefficient_of_variation: f64,
    pub num_flows: usize,
}

impl FairnessMetrics {
    pub fn level(&self) -> FairnessLevel {
        JainFairnessCalculator::interpret(self.jain_index)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FairnessComparison {
    pub baseline: FairnessMetrics,
    pub optimized: FairnessMetrics,
    pub jain_improvement: f64,
    pub cv_improvement: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FairnessLevel {
    Excellent,  // >= 0.95
    Good,       // >= 0.85
    Fair,       // >= 0.70
    Poor,       // >= 0.50
    VeryPoor,   // < 0.50
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perfect_fairness() {
        let throughputs = vec![100.0, 100.0, 100.0, 100.0];
        let metrics = JainFairnessCalculator::calculate(&throughputs).unwrap();
        assert!((metrics.jain_index - 1.0).abs() < 0.001);
        assert_eq!(metrics.level(), FairnessLevel::Excellent);
    }

    #[test]
    fn test_unfair_distribution() {
        let throughputs = vec![100.0, 10.0, 10.0, 10.0];
        let metrics = JainFairnessCalculator::calculate(&throughputs).unwrap();
        assert!(metrics.jain_index < 0.7);
    }

    #[test]
    fn test_empty_input() {
        let throughputs: Vec<f64> = vec![];
        let result = JainFairnessCalculator::calculate(&throughputs);
        assert!(result.is_err());
    }
}

use crate::types::*;
use std::collections::HashMap;
use tracing::debug;

/// Load balancing strategy
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    WeightedRoundRobin,
    LeastLoaded,
    PowerOfTwoChoices,
    ECMP, // Equal-Cost Multi-Path
}

/// Traffic load balancer
pub struct LoadBalancer {
    strategy: LoadBalancingStrategy,
    path_weights: HashMap<String, f32>,
    current_index: usize,
}

impl LoadBalancer {
    pub fn new(strategy: LoadBalancingStrategy) -> Self {
        Self {
            strategy,
            path_weights: HashMap::new(),
            current_index: 0,
        }
    }

    /// Distribute traffic across multiple paths
    pub fn distribute_traffic(
        &mut self,
        paths: &[Path],
        total_traffic: u64,
    ) -> Vec<TrafficAllocation> {
        match self.strategy {
            LoadBalancingStrategy::RoundRobin => self.round_robin(paths, total_traffic),
            LoadBalancingStrategy::WeightedRoundRobin => {
                self.weighted_round_robin(paths, total_traffic)
            }
            LoadBalancingStrategy::LeastLoaded => self.least_loaded(paths, total_traffic),
            LoadBalancingStrategy::PowerOfTwoChoices => {
                self.power_of_two_choices(paths, total_traffic)
            }
            LoadBalancingStrategy::ECMP => self.ecmp(paths, total_traffic),
        }
    }

    /// Select next path for a flow
    pub fn select_path(&mut self, paths: &[Path]) -> Option<usize> {
        if paths.is_empty() {
            return None;
        }

        match self.strategy {
            LoadBalancingStrategy::RoundRobin => {
                let index = self.current_index % paths.len();
                self.current_index += 1;
                Some(index)
            }
            LoadBalancingStrategy::WeightedRoundRobin => {
                self.select_weighted_path(paths)
            }
            LoadBalancingStrategy::LeastLoaded => {
                self.select_least_loaded_path(paths)
            }
            LoadBalancingStrategy::PowerOfTwoChoices => {
                self.select_power_of_two_path(paths)
            }
            LoadBalancingStrategy::ECMP => {
                // Hash-based selection for ECMP
                Some(self.current_index % paths.len())
            }
        }
    }

    /// Round-robin distribution
    fn round_robin(&self, paths: &[Path], total_traffic: u64) -> Vec<TrafficAllocation> {
        if paths.is_empty() {
            return vec![];
        }

        let per_path = total_traffic / paths.len() as u64;
        let remainder = total_traffic % paths.len() as u64;

        paths
            .iter()
            .enumerate()
            .map(|(i, path)| {
                let traffic = if i == 0 {
                    per_path + remainder
                } else {
                    per_path
                };

                TrafficAllocation {
                    path_id: format!("path_{}", i),
                    traffic_bps: traffic,
                    percentage: (traffic as f32 / total_traffic as f32) * 100.0,
                }
            })
            .collect()
    }

    /// Weighted round-robin distribution
    fn weighted_round_robin(&self, paths: &[Path], total_traffic: u64) -> Vec<TrafficAllocation> {
        if paths.is_empty() {
            return vec![];
        }

        // Calculate weights based on bandwidth
        let total_bandwidth: u64 = paths.iter().map(|p| p.bandwidth).sum();
        
        if total_bandwidth == 0 {
            return self.round_robin(paths, total_traffic);
        }

        paths
            .iter()
            .enumerate()
            .map(|(i, path)| {
                let weight = path.bandwidth as f64 / total_bandwidth as f64;
                let traffic = (total_traffic as f64 * weight) as u64;

                TrafficAllocation {
                    path_id: format!("path_{}", i),
                    traffic_bps: traffic,
                    percentage: (weight * 100.0) as f32,
                }
            })
            .collect()
    }

    /// Least loaded distribution
    fn least_loaded(&self, paths: &[Path], total_traffic: u64) -> Vec<TrafficAllocation> {
        // TODO: Track actual load per path
        // For now, use weighted distribution
        self.weighted_round_robin(paths, total_traffic)
    }

    /// Power of two choices
    fn power_of_two_choices(&self, paths: &[Path], total_traffic: u64) -> Vec<TrafficAllocation> {
        // TODO: Implement power of two choices
        self.weighted_round_robin(paths, total_traffic)
    }

    /// Equal-Cost Multi-Path (ECMP)
    fn ecmp(&self, paths: &[Path], total_traffic: u64) -> Vec<TrafficAllocation> {
        // Filter paths with equal cost
        if paths.is_empty() {
            return vec![];
        }

        let min_cost = paths.iter().map(|p| p.cost).fold(f64::INFINITY, f64::min);
        let equal_cost_paths: Vec<&Path> = paths
            .iter()
            .filter(|p| (p.cost - min_cost).abs() < 1e-6)
            .collect();

        if equal_cost_paths.is_empty() {
            return vec![];
        }

        let per_path = total_traffic / equal_cost_paths.len() as u64;

        equal_cost_paths
            .iter()
            .enumerate()
            .map(|(i, _path)| TrafficAllocation {
                path_id: format!("path_{}", i),
                traffic_bps: per_path,
                percentage: 100.0 / equal_cost_paths.len() as f32,
            })
            .collect()
    }

    fn select_weighted_path(&self, paths: &[Path]) -> Option<usize> {
        let total_bandwidth: u64 = paths.iter().map(|p| p.bandwidth).sum();
        if total_bandwidth == 0 {
            return Some(0);
        }

        // TODO: Implement proper weighted selection
        Some(0)
    }

    fn select_least_loaded_path(&self, paths: &[Path]) -> Option<usize> {
        // Select path with highest available bandwidth
        paths
            .iter()
            .enumerate()
            .max_by_key(|(_, p)| p.bandwidth)
            .map(|(i, _)| i)
    }

    fn select_power_of_two_path(&self, paths: &[Path]) -> Option<usize> {
        if paths.len() < 2 {
            return Some(0);
        }

        // TODO: Implement power of two choices
        Some(0)
    }
}

#[derive(Debug, Clone)]
pub struct TrafficAllocation {
    pub path_id: String,
    pub traffic_bps: u64,
    pub percentage: f32,
}

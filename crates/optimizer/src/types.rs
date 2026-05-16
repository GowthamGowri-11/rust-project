use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Path {
    pub nodes: Vec<String>,
    pub cost: f64,
    pub bandwidth: u64,
    pub latency_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub primary_path: Path,
    pub backup_paths: Vec<Path>,
    pub load_distribution: Vec<LoadDistribution>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadDistribution {
    pub path_id: String,
    pub traffic_percentage: f32,
}

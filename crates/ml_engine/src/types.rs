use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub model_path: String,
    pub input_shape: Vec<usize>,
    pub output_shape: Vec<usize>,
    pub batch_size: usize,
    pub num_threads: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceInput {
    pub features: Vec<f32>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceOutput {
    pub predictions: Vec<f32>,
    pub confidence: f32,
    pub latency_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionResult {
    pub congestion_probability: f32,
    pub predicted_bandwidth: f64,
    pub recommended_action: RecommendedAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendedAction {
    NoAction,
    Reroute { alternative_path: Vec<String> },
    LoadBalance { split_ratio: f32 },
    ThrottleFlow { flow_id: String, rate_limit: u64 },
}

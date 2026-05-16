use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub api: ApiConfig,
    pub controller: ControllerConfig,
    pub monitoring: MonitoringConfig,
    pub ml_engine: MlEngineConfig,
    pub optimizer: OptimizerConfig,
    pub resilience: ResilienceConfig,
    pub metrics: MetricsConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ControllerConfig {
    pub host: String,
    pub port: u16,
    pub openflow_version: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MonitoringConfig {
    pub ebpf_enabled: bool,
    pub interval_ms: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MlEngineConfig {
    pub model_path: String,
    pub batch_size: usize,
    pub num_threads: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OptimizerConfig {
    pub algorithm: String,
    pub rebalance_threshold: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ResilienceConfig {
    pub failure_timeout_ms: u64,
    pub backup_path_count: usize,
    pub auto_recovery: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MetricsConfig {
    pub prometheus_port: u16,
    pub export_interval_ms: u64,
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        let config = Config::builder()
            // Start with default configuration
            .add_source(File::with_name("configs/default").required(false))
            // Add environment-specific config
            .add_source(File::with_name(&format!("configs/{}", run_mode)).required(false))
            // Add local config (not committed to git)
            .add_source(File::with_name("configs/local").required(false))
            // Override with environment variables (with prefix APP)
            .add_source(Environment::with_prefix("APP").separator("__"))
            .build()?;

        config.try_deserialize()
    }

    pub fn load_or_default() -> Self {
        Self::load().unwrap_or_else(|_| Self::default())
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            api: ApiConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
            },
            controller: ControllerConfig {
                host: "0.0.0.0".to_string(),
                port: 6653,
                openflow_version: "1.3".to_string(),
            },
            monitoring: MonitoringConfig {
                ebpf_enabled: false,
                interval_ms: 1000,
            },
            ml_engine: MlEngineConfig {
                model_path: "/models/traffic_predictor.onnx".to_string(),
                batch_size: 32,
                num_threads: 4,
            },
            optimizer: OptimizerConfig {
                algorithm: "shortest_path".to_string(),
                rebalance_threshold: 0.8,
            },
            resilience: ResilienceConfig {
                failure_timeout_ms: 3000,
                backup_path_count: 2,
                auto_recovery: true,
            },
            metrics: MetricsConfig {
                prometheus_port: 9090,
                export_interval_ms: 5000,
            },
        }
    }
}

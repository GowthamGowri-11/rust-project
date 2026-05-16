use lazy_static::lazy_static;
use prometheus::{Counter, Gauge, Histogram, Registry, TextEncoder};
use tracing::info;

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();
    
    pub static ref FLOW_INSTALL_COUNTER: Counter = Counter::new(
        "rustflow_flow_installs_total",
        "Total number of flow installations"
    ).unwrap();
    
    pub static ref ACTIVE_FLOWS_GAUGE: Gauge = Gauge::new(
        "rustflow_active_flows",
        "Number of currently active flows"
    ).unwrap();
    
    pub static ref BANDWIDTH_GAUGE: Gauge = Gauge::new(
        "rustflow_bandwidth_bps",
        "Current network bandwidth in bits per second"
    ).unwrap();
    
    pub static ref LATENCY_HISTOGRAM: Histogram = Histogram::with_opts(
        prometheus::HistogramOpts::new(
            "rustflow_latency_ms",
            "Network latency in milliseconds"
        )
    ).unwrap();
    
    pub static ref ML_INFERENCE_HISTOGRAM: Histogram = Histogram::with_opts(
        prometheus::HistogramOpts::new(
            "rustflow_ml_inference_duration_ms",
            "ML inference duration in milliseconds"
        )
    ).unwrap();
}

pub struct PrometheusExporter;

impl PrometheusExporter {
    pub fn init() {
        info!("Initializing Prometheus metrics");
        
        REGISTRY.register(Box::new(FLOW_INSTALL_COUNTER.clone())).unwrap();
        REGISTRY.register(Box::new(ACTIVE_FLOWS_GAUGE.clone())).unwrap();
        REGISTRY.register(Box::new(BANDWIDTH_GAUGE.clone())).unwrap();
        REGISTRY.register(Box::new(LATENCY_HISTOGRAM.clone())).unwrap();
        REGISTRY.register(Box::new(ML_INFERENCE_HISTOGRAM.clone())).unwrap();
    }

    pub fn export() -> String {
        let encoder = TextEncoder::new();
        let metric_families = REGISTRY.gather();
        encoder.encode_to_string(&metric_families).unwrap()
    }
}

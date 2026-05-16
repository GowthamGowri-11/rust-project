use crate::error::Result;
use crate::collectors::MetricCollector;
use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::info;

/// Latency collector for network paths
pub struct LatencyCollector {
    latency_samples: Arc<DashMap<String, LatencySamples>>,
    running: Arc<AtomicBool>,
}

#[derive(Debug)]
pub struct LatencySamples {
    samples: parking_lot::Mutex<Vec<f64>>,
    max_samples: usize,
}

impl LatencySamples {
    pub fn new(max_samples: usize) -> Self {
        Self {
            samples: parking_lot::Mutex::new(Vec::with_capacity(max_samples)),
            max_samples,
        }
    }

    pub fn add_sample(&self, latency_ms: f64) {
        let mut samples = self.samples.lock();
        if samples.len() >= self.max_samples {
            samples.remove(0);
        }
        samples.push(latency_ms);
    }

    pub fn get_average(&self) -> f64 {
        let samples = self.samples.lock();
        if samples.is_empty() {
            return 0.0;
        }
        samples.iter().sum::<f64>() / samples.len() as f64
    }

    pub fn get_percentile(&self, percentile: f64) -> f64 {
        let mut samples = self.samples.lock().clone();
        if samples.is_empty() {
            return 0.0;
        }
        samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let index = ((percentile / 100.0) * samples.len() as f64) as usize;
        samples[index.min(samples.len() - 1)]
    }

    pub fn get_min(&self) -> f64 {
        self.samples.lock().iter().cloned().fold(f64::INFINITY, f64::min)
    }

    pub fn get_max(&self) -> f64 {
        self.samples.lock().iter().cloned().fold(f64::NEG_INFINITY, f64::max)
    }
}

impl LatencyCollector {
    pub fn new() -> Self {
        Self {
            latency_samples: Arc::new(DashMap::new()),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Record latency sample for a link
    pub fn record_latency(&self, link_id: &str, latency_ms: f64) {
        let samples = self.latency_samples
            .entry(link_id.to_string())
            .or_insert_with(|| LatencySamples::new(1000));
        samples.add_sample(latency_ms);
    }

    /// Get average latency for a link
    pub fn get_average_latency(&self, link_id: &str) -> Option<f64> {
        self.latency_samples.get(link_id).map(|s| s.get_average())
    }

    /// Get latency percentile
    pub fn get_latency_percentile(&self, link_id: &str, percentile: f64) -> Option<f64> {
        self.latency_samples.get(link_id).map(|s| s.get_percentile(percentile))
    }

    /// Get latency statistics
    pub fn get_latency_stats(&self, link_id: &str) -> Option<LatencyStats> {
        self.latency_samples.get(link_id).map(|samples| {
            LatencyStats {
                avg: samples.get_average(),
                min: samples.get_min(),
                max: samples.get_max(),
                p50: samples.get_percentile(50.0),
                p95: samples.get_percentile(95.0),
                p99: samples.get_percentile(99.0),
            }
        })
    }
}

#[derive(Debug, Clone)]
pub struct LatencyStats {
    pub avg: f64,
    pub min: f64,
    pub max: f64,
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
}

#[async_trait]
impl MetricCollector for LatencyCollector {
    async fn start(&self) -> Result<()> {
        info!("Starting latency collector");
        self.running.store(true, Ordering::Relaxed);
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        info!("Stopping latency collector");
        self.running.store(false, Ordering::Relaxed);
        Ok(())
    }

    fn name(&self) -> &str {
        "latency"
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
}

impl Default for LatencyCollector {
    fn default() -> Self {
        Self::new()
    }
}

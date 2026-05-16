use crate::error::Result;
use crate::types::*;
use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::time::{interval, Duration, Instant};
use tracing::{debug, warn};

/// Failure detection trait
#[async_trait]
pub trait FailureDetector: Send + Sync {
    async fn detect(&self) -> Result<Vec<FailureEvent>>;
    async fn start(&self) -> Result<()>;
    async fn stop(&self) -> Result<()>;
}

/// Link failure detector
pub struct LinkFailureDetector {
    link_health: Arc<DashMap<String, LinkHealth>>,
    timeout_ms: u64,
    check_interval_ms: u64,
    running: Arc<std::sync::atomic::AtomicBool>,
}

impl LinkFailureDetector {
    pub fn new(timeout_ms: u64, check_interval_ms: u64) -> Self {
        Self {
            link_health: Arc::new(DashMap::new()),
            timeout_ms,
            check_interval_ms,
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Update link heartbeat
    pub fn heartbeat(&self, link_id: &str) {
        let health = self.link_health
            .entry(link_id.to_string())
            .or_insert_with(LinkHealth::new);
        health.update_heartbeat();
    }

    /// Check if link is alive
    pub fn is_alive(&self, link_id: &str) -> bool {
        self.link_health
            .get(link_id)
            .map(|h| h.is_alive(self.timeout_ms))
            .unwrap_or(false)
    }

    /// Get failed links
    pub fn get_failed_links(&self) -> Vec<String> {
        self.link_health
            .iter()
            .filter(|entry| !entry.value().is_alive(self.timeout_ms))
            .map(|entry| entry.key().clone())
            .collect()
    }

    async fn detection_loop(&self) {
        let mut ticker = interval(Duration::from_millis(self.check_interval_ms));
        
        while self.running.load(Ordering::Relaxed) {
            ticker.tick().await;
            
            let failed = self.get_failed_links();
            if !failed.is_empty() {
                warn!("Detected {} failed links: {:?}", failed.len(), failed);
            }
        }
    }
}

#[async_trait]
impl FailureDetector for LinkFailureDetector {
    async fn detect(&self) -> Result<Vec<FailureEvent>> {
        let failed_links = self.get_failed_links();
        
        Ok(failed_links
            .into_iter()
            .map(|link_id| FailureEvent {
                event_id: uuid::Uuid::new_v4().to_string(),
                failure_type: FailureType::LinkDown,
                affected_component: link_id,
                timestamp: chrono::Utc::now(),
                severity: FailureSeverity::Major,
            })
            .collect())
    }

    async fn start(&self) -> Result<()> {
        debug!("Starting link failure detector");
        self.running.store(true, Ordering::Relaxed);
        
        let detector = self.clone();
        tokio::spawn(async move {
            detector.detection_loop().await;
        });
        
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        debug!("Stopping link failure detector");
        self.running.store(false, Ordering::Relaxed);
        Ok(())
    }
}

impl Clone for LinkFailureDetector {
    fn clone(&self) -> Self {
        Self {
            link_health: Arc::clone(&self.link_health),
            timeout_ms: self.timeout_ms,
            check_interval_ms: self.check_interval_ms,
            running: Arc::clone(&self.running),
        }
    }
}

/// Traffic spike detector
pub struct TrafficSpikeDetector {
    baseline: Arc<DashMap<String, TrafficBaseline>>,
    threshold_multiplier: f64,
}

impl TrafficSpikeDetector {
    pub fn new(threshold_multiplier: f64) -> Self {
        Self {
            baseline: Arc::new(DashMap::new()),
            threshold_multiplier,
        }
    }

    /// Update traffic baseline
    pub fn update_baseline(&self, link_id: &str, traffic_bps: u64) {
        let baseline = self.baseline
            .entry(link_id.to_string())
            .or_insert_with(TrafficBaseline::new);
        baseline.add_sample(traffic_bps);
    }

    /// Detect traffic spike
    pub fn detect_spike(&self, link_id: &str, current_traffic: u64) -> bool {
        if let Some(baseline) = self.baseline.get(link_id) {
            let avg = baseline.get_average();
            let threshold = avg * self.threshold_multiplier;
            current_traffic as f64 > threshold
        } else {
            false
        }
    }
}

#[derive(Debug)]
pub struct LinkHealth {
    last_heartbeat: parking_lot::Mutex<Instant>,
    failure_count: AtomicU64,
}

impl LinkHealth {
    pub fn new() -> Self {
        Self {
            last_heartbeat: parking_lot::Mutex::new(Instant::now()),
            failure_count: AtomicU64::new(0),
        }
    }

    pub fn update_heartbeat(&self) {
        *self.last_heartbeat.lock() = Instant::now();
    }

    pub fn is_alive(&self, timeout_ms: u64) -> bool {
        self.last_heartbeat.lock().elapsed().as_millis() < timeout_ms as u128
    }

    pub fn record_failure(&self) {
        self.failure_count.fetch_add(1, Ordering::Relaxed);
    }
}

#[derive(Debug)]
pub struct TrafficBaseline {
    samples: parking_lot::Mutex<Vec<u64>>,
    max_samples: usize,
}

impl TrafficBaseline {
    pub fn new() -> Self {
        Self {
            samples: parking_lot::Mutex::new(Vec::with_capacity(100)),
            max_samples: 100,
        }
    }

    pub fn add_sample(&self, traffic: u64) {
        let mut samples = self.samples.lock();
        if samples.len() >= self.max_samples {
            samples.remove(0);
        }
        samples.push(traffic);
    }

    pub fn get_average(&self) -> f64 {
        let samples = self.samples.lock();
        if samples.is_empty() {
            return 0.0;
        }
        samples.iter().sum::<u64>() as f64 / samples.len() as f64
    }
}

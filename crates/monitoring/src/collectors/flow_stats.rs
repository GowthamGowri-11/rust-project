use crate::error::Result;
use crate::collectors::MetricCollector;
use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tracing::info;

/// Flow statistics collector
pub struct FlowStatsCollector {
    flow_stats: Arc<DashMap<String, FlowStats>>,
    running: Arc<AtomicBool>,
}

#[derive(Debug)]
pub struct FlowStats {
    pub packet_count: AtomicU64,
    pub byte_count: AtomicU64,
    pub start_time: parking_lot::Mutex<std::time::Instant>,
    pub last_seen: parking_lot::Mutex<std::time::Instant>,
}

impl FlowStats {
    pub fn new() -> Self {
        let now = std::time::Instant::now();
        Self {
            packet_count: AtomicU64::new(0),
            byte_count: AtomicU64::new(0),
            start_time: parking_lot::Mutex::new(now),
            last_seen: parking_lot::Mutex::new(now),
        }
    }

    pub fn update(&self, packets: u64, bytes: u64) {
        self.packet_count.fetch_add(packets, Ordering::Relaxed);
        self.byte_count.fetch_add(bytes, Ordering::Relaxed);
        *self.last_seen.lock() = std::time::Instant::now();
    }

    pub fn get_throughput_bps(&self) -> u64 {
        let duration = self.start_time.lock().elapsed().as_secs_f64();
        if duration <= 0.0 {
            return 0;
        }
        let bytes = self.byte_count.load(Ordering::Relaxed);
        ((bytes as f64 * 8.0) / duration) as u64
    }
}

impl FlowStatsCollector {
    pub fn new() -> Self {
        Self {
            flow_stats: Arc::new(DashMap::new()),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn record_flow(&self, flow_id: &str, packets: u64, bytes: u64) {
        let stats = self.flow_stats
            .entry(flow_id.to_string())
            .or_insert_with(FlowStats::new);
        stats.update(packets, bytes);
    }

    pub fn get_flow_throughput(&self, flow_id: &str) -> Option<u64> {
        self.flow_stats.get(flow_id).map(|s| s.get_throughput_bps())
    }
}

#[async_trait]
impl MetricCollector for FlowStatsCollector {
    async fn start(&self) -> Result<()> {
        info!("Starting flow stats collector");
        self.running.store(true, Ordering::Relaxed);
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        info!("Stopping flow stats collector");
        self.running.store(false, Ordering::Relaxed);
        Ok(())
    }

    fn name(&self) -> &str {
        "flow_stats"
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
}

impl Default for FlowStatsCollector {
    fn default() -> Self {
        Self::new()
    }
}

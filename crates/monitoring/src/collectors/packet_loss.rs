use crate::error::Result;
use crate::collectors::MetricCollector;
use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tracing::info;

/// Packet loss collector
pub struct PacketLossCollector {
    link_stats: Arc<DashMap<String, PacketLossStats>>,
    running: Arc<AtomicBool>,
}

#[derive(Debug)]
pub struct PacketLossStats {
    pub sent: AtomicU64,
    pub received: AtomicU64,
    pub lost: AtomicU64,
}

impl PacketLossStats {
    pub fn new() -> Self {
        Self {
            sent: AtomicU64::new(0),
            received: AtomicU64::new(0),
            lost: AtomicU64::new(0),
        }
    }

    pub fn record_sent(&self, count: u64) {
        self.sent.fetch_add(count, Ordering::Relaxed);
    }

    pub fn record_received(&self, count: u64) {
        self.received.fetch_add(count, Ordering::Relaxed);
    }

    pub fn record_lost(&self, count: u64) {
        self.lost.fetch_add(count, Ordering::Relaxed);
    }

    pub fn get_loss_rate(&self) -> f64 {
        let sent = self.sent.load(Ordering::Relaxed);
        if sent == 0 {
            return 0.0;
        }
        let lost = self.lost.load(Ordering::Relaxed);
        (lost as f64 / sent as f64) * 100.0
    }

    pub fn get_sent(&self) -> u64 {
        self.sent.load(Ordering::Relaxed)
    }

    pub fn get_received(&self) -> u64 {
        self.received.load(Ordering::Relaxed)
    }

    pub fn get_lost(&self) -> u64 {
        self.lost.load(Ordering::Relaxed)
    }
}

impl PacketLossCollector {
    pub fn new() -> Self {
        Self {
            link_stats: Arc::new(DashMap::new()),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Record sent packets
    pub fn record_sent(&self, link_id: &str, count: u64) {
        let stats = self.link_stats
            .entry(link_id.to_string())
            .or_insert_with(PacketLossStats::new);
        stats.record_sent(count);
    }

    /// Record received packets
    pub fn record_received(&self, link_id: &str, count: u64) {
        let stats = self.link_stats
            .entry(link_id.to_string())
            .or_insert_with(PacketLossStats::new);
        stats.record_received(count);
    }

    /// Record lost packets
    pub fn record_lost(&self, link_id: &str, count: u64) {
        let stats = self.link_stats
            .entry(link_id.to_string())
            .or_insert_with(PacketLossStats::new);
        stats.record_lost(count);
    }

    /// Get packet loss rate for a link
    pub fn get_loss_rate(&self, link_id: &str) -> Option<f64> {
        self.link_stats.get(link_id).map(|s| s.get_loss_rate())
    }

    /// Get packet loss statistics
    pub fn get_stats(&self, link_id: &str) -> Option<(u64, u64, u64, f64)> {
        self.link_stats.get(link_id).map(|stats| {
            (
                stats.get_sent(),
                stats.get_received(),
                stats.get_lost(),
                stats.get_loss_rate(),
            )
        })
    }
}

#[async_trait]
impl MetricCollector for PacketLossCollector {
    async fn start(&self) -> Result<()> {
        info!("Starting packet loss collector");
        self.running.store(true, Ordering::Relaxed);
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        info!("Stopping packet loss collector");
        self.running.store(false, Ordering::Relaxed);
        Ok(())
    }

    fn name(&self) -> &str {
        "packet_loss"
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
}

impl Default for PacketLossCollector {
    fn default() -> Self {
        Self::new()
    }
}

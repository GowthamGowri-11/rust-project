use crate::error::Result;
use crate::collectors::MetricCollector;
use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tokio::time::{interval, Duration};
use tracing::{debug, info};

/// Bandwidth collector for network interfaces
pub struct BandwidthCollector {
    interface_stats: Arc<DashMap<String, InterfaceStats>>,
    running: Arc<AtomicBool>,
    interval_ms: u64,
}

#[derive(Debug)]
pub struct InterfaceStats {
    pub rx_bytes: AtomicU64,
    pub tx_bytes: AtomicU64,
    pub rx_packets: AtomicU64,
    pub tx_packets: AtomicU64,
    pub last_update: parking_lot::Mutex<std::time::Instant>,
}

impl InterfaceStats {
    pub fn new() -> Self {
        Self {
            rx_bytes: AtomicU64::new(0),
            tx_bytes: AtomicU64::new(0),
            rx_packets: AtomicU64::new(0),
            tx_packets: AtomicU64::new(0),
            last_update: parking_lot::Mutex::new(std::time::Instant::now()),
        }
    }

    pub fn update_rx(&self, bytes: u64, packets: u64) {
        self.rx_bytes.fetch_add(bytes, Ordering::Relaxed);
        self.rx_packets.fetch_add(packets, Ordering::Relaxed);
        *self.last_update.lock() = std::time::Instant::now();
    }

    pub fn update_tx(&self, bytes: u64, packets: u64) {
        self.tx_bytes.fetch_add(bytes, Ordering::Relaxed);
        self.tx_packets.fetch_add(packets, Ordering::Relaxed);
        *self.last_update.lock() = std::time::Instant::now();
    }

    pub fn get_rx_bytes(&self) -> u64 {
        self.rx_bytes.load(Ordering::Relaxed)
    }

    pub fn get_tx_bytes(&self) -> u64 {
        self.tx_bytes.load(Ordering::Relaxed)
    }

    pub fn get_total_bytes(&self) -> u64 {
        self.get_rx_bytes() + self.get_tx_bytes()
    }

    pub fn calculate_bandwidth_bps(&self, duration_secs: f64) -> u64 {
        if duration_secs <= 0.0 {
            return 0;
        }
        ((self.get_total_bytes() as f64 * 8.0) / duration_secs) as u64
    }
}

impl BandwidthCollector {
    pub fn new(interval_ms: u64) -> Self {
        Self {
            interface_stats: Arc::new(DashMap::new()),
            running: Arc::new(AtomicBool::new(false)),
            interval_ms,
        }
    }

    /// Record bandwidth for interface
    pub fn record_rx(&self, interface: &str, bytes: u64, packets: u64) {
        let stats = self.interface_stats
            .entry(interface.to_string())
            .or_insert_with(InterfaceStats::new);
        stats.update_rx(bytes, packets);
    }

    pub fn record_tx(&self, interface: &str, bytes: u64, packets: u64) {
        let stats = self.interface_stats
            .entry(interface.to_string())
            .or_insert_with(InterfaceStats::new);
        stats.update_tx(bytes, packets);
    }

    /// Get bandwidth for interface in bps
    pub fn get_bandwidth(&self, interface: &str) -> Option<u64> {
        self.interface_stats.get(interface).map(|stats| {
            let elapsed = stats.last_update.lock().elapsed().as_secs_f64();
            stats.calculate_bandwidth_bps(elapsed)
        })
    }

    /// Get total bandwidth across all interfaces
    pub fn get_total_bandwidth(&self) -> u64 {
        self.interface_stats.iter().map(|entry| {
            let stats = entry.value();
            let elapsed = stats.last_update.lock().elapsed().as_secs_f64();
            stats.calculate_bandwidth_bps(elapsed)
        }).sum()
    }

    /// Get all interface statistics
    pub fn get_all_stats(&self) -> Vec<(String, u64, u64)> {
        self.interface_stats.iter().map(|entry| {
            let interface = entry.key().clone();
            let stats = entry.value();
            (interface, stats.get_rx_bytes(), stats.get_tx_bytes())
        }).collect()
    }

    async fn collect_loop(&self) {
        let mut ticker = interval(Duration::from_millis(self.interval_ms));
        
        while self.running.load(Ordering::Relaxed) {
            ticker.tick().await;
            
            // TODO: Read actual interface statistics from /proc/net/dev or netlink
            // For now, this is a placeholder for the collection logic
            
            debug!("Bandwidth collection tick");
        }
    }
}

#[async_trait]
impl MetricCollector for BandwidthCollector {
    async fn start(&self) -> Result<()> {
        info!("Starting bandwidth collector (interval: {}ms)", self.interval_ms);
        self.running.store(true, Ordering::Relaxed);
        
        let collector = self.clone();
        tokio::spawn(async move {
            collector.collect_loop().await;
        });
        
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        info!("Stopping bandwidth collector");
        self.running.store(false, Ordering::Relaxed);
        Ok(())
    }

    fn name(&self) -> &str {
        "bandwidth"
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
}

impl Clone for BandwidthCollector {
    fn clone(&self) -> Self {
        Self {
            interface_stats: Arc::clone(&self.interface_stats),
            running: Arc::clone(&self.running),
            interval_ms: self.interval_ms,
        }
    }
}

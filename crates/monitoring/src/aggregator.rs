use crate::types::{LinkMetrics, NetworkMetrics};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::debug;

/// Metrics aggregation layer
pub struct MetricsAggregator {
    link_metrics: Arc<DashMap<String, LinkMetrics>>,
    aggregation_interval_ms: u64,
}

impl MetricsAggregator {
    pub fn new(aggregation_interval_ms: u64) -> Self {
        Self {
            link_metrics: Arc::new(DashMap::new()),
            aggregation_interval_ms,
        }
    }

    /// Update link metrics
    pub fn update_link(&self, link_id: String, metrics: LinkMetrics) {
        self.link_metrics.insert(link_id, metrics);
    }

    /// Get link metrics
    pub fn get_link(&self, link_id: &str) -> Option<LinkMetrics> {
        self.link_metrics.get(link_id).map(|m| m.clone())
    }

    /// Get all link metrics
    pub fn get_all_links(&self) -> Vec<LinkMetrics> {
        self.link_metrics.iter().map(|entry| entry.value().clone()).collect()
    }

    /// Aggregate network-wide metrics
    pub fn aggregate_network(&self) -> NetworkMetrics {
        let links = self.get_all_links();
        
        let total_bandwidth = links.iter().map(|l| l.bandwidth_bps).sum();
        let avg_latency = if !links.is_empty() {
            links.iter().map(|l| l.latency_ms).sum::<f64>() / links.len() as f64
        } else {
            0.0
        };
        let avg_loss = if !links.is_empty() {
            links.iter().map(|l| l.packet_loss).sum::<f64>() / links.len() as f64
        } else {
            0.0
        };

        NetworkMetrics {
            total_bandwidth,
            active_flows: 0, // TODO: Count from flow collector
            avg_latency_ms: avg_latency,
            packet_loss_rate: avg_loss,
            link_metrics: links,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Start aggregation loop
    pub async fn start_aggregation(&self) {
        let mut ticker = interval(Duration::from_millis(self.aggregation_interval_ms));
        
        loop {
            ticker.tick().await;
            debug!("Aggregating metrics");
            
            // Perform aggregation
            let _network_metrics = self.aggregate_network();
            
            // TODO: Push to time-series database or metrics store
        }
    }
}

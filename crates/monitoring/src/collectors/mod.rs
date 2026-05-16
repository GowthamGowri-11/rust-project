pub mod bandwidth;
pub mod latency;
pub mod packet_loss;
pub mod flow_stats;

pub use bandwidth::BandwidthCollector;
pub use latency::LatencyCollector;
pub use packet_loss::PacketLossCollector;
pub use flow_stats::FlowStatsCollector;

use crate::error::Result;
use async_trait::async_trait;

/// Trait for metric collectors
#[async_trait]
pub trait MetricCollector: Send + Sync {
    /// Start collecting metrics
    async fn start(&self) -> Result<()>;
    
    /// Stop collecting metrics
    async fn stop(&self) -> Result<()>;
    
    /// Get collector name
    fn name(&self) -> &str;
    
    /// Check if collector is running
    fn is_running(&self) -> bool;
}

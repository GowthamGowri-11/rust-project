use crate::{error::Result, types::*};
use crate::collectors::{BandwidthCollector, LatencyCollector, PacketLossCollector, FlowStatsCollector, MetricCollector};
use crate::ebpf::EbpfManager;
use crate::aggregator::MetricsAggregator;
use async_trait::async_trait;
use chrono::Utc;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{debug, info, warn};

#[async_trait]
pub trait Monitor: Send + Sync {
    async fn start(&self) -> Result<()>;
    async fn stop(&self) -> Result<()>;
    async fn get_network_metrics(&self) -> Result<NetworkMetrics>;
    async fn get_link_metrics(&self, link_id: &str) -> Result<LinkMetrics>;
}

pub struct MonitoringService {
    // Collectors
    bandwidth_collector: Arc<BandwidthCollector>,
    latency_collector: Arc<LatencyCollector>,
    packet_loss_collector: Arc<PacketLossCollector>,
    flow_stats_collector: Arc<FlowStatsCollector>,
    
    // eBPF manager
    ebpf_manager: Arc<EbpfManager>,
    
    // Aggregator
    aggregator: Arc<MetricsAggregator>,
    
    // Configuration
    interval_ms: u64,
    ebpf_enabled: bool,
    
    // State
    link_metrics: Arc<DashMap<String, LinkMetrics>>,
    traffic_samples: Arc<DashMap<String, Vec<TrafficSample>>>,
}

impl MonitoringService {
    pub fn new(interval_ms: u64, ebpf_enabled: bool) -> Self {
        Self {
            bandwidth_collector: Arc::new(BandwidthCollector::new(interval_ms)),
            latency_collector: Arc::new(LatencyCollector::new()),
            packet_loss_collector: Arc::new(PacketLossCollector::new()),
            flow_stats_collector: Arc::new(FlowStatsCollector::new()),
            ebpf_manager: Arc::new(EbpfManager::new(ebpf_enabled)),
            aggregator: Arc::new(MetricsAggregator::new(interval_ms)),
            interval_ms,
            ebpf_enabled,
            link_metrics: Arc::new(DashMap::new()),
            traffic_samples: Arc::new(DashMap::new()),
        }
    }

    async fn collect_metrics(&self) -> Result<()> {
        debug!("Collecting network metrics");
        
        // Collect from all sources
        let bandwidth = self.bandwidth_collector.get_total_bandwidth();
        
        // Update aggregator
        // TODO: Collect from actual interfaces and update aggregator
        
        Ok(())
    }

    async fn attach_ebpf_probes(&self) -> Result<()> {
        if !self.ebpf_enabled {
            info!("eBPF monitoring disabled");
            return Ok(());
        }

        info!("Attaching eBPF probes");
        self.ebpf_manager.init().await?;
        
        // Attach to default interfaces
        // TODO: Get actual interface list
        self.ebpf_manager.attach_probe("eth0").await?;
        
        Ok(())
    }

    async fn detach_ebpf_probes(&self) -> Result<()> {
        if !self.ebpf_enabled {
            return Ok(());
        }

        info!("Detaching eBPF probes");
        self.ebpf_manager.detach_all().await?;
        Ok(())
    }

    /// Get bandwidth collector
    pub fn bandwidth_collector(&self) -> &BandwidthCollector {
        &self.bandwidth_collector
    }

    /// Get latency collector
    pub fn latency_collector(&self) -> &LatencyCollector {
        &self.latency_collector
    }

    /// Get packet loss collector
    pub fn packet_loss_collector(&self) -> &PacketLossCollector {
        &self.packet_loss_collector
    }

    /// Get flow stats collector
    pub fn flow_stats_collector(&self) -> &FlowStatsCollector {
        &self.flow_stats_collector
    }
}

#[async_trait]
impl Monitor for MonitoringService {
    async fn start(&self) -> Result<()> {
        info!("Starting monitoring service (interval: {}ms, eBPF: {})", 
              self.interval_ms, self.ebpf_enabled);
        
        // Start eBPF if enabled
        self.attach_ebpf_probes().await?;

        // Start all collectors
        self.bandwidth_collector.start().await?;
        self.latency_collector.start().await?;
        self.packet_loss_collector.start().await?;
        self.flow_stats_collector.start().await?;

        // Start collection loop
        let service = self.clone();
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_millis(service.interval_ms));
            loop {
                ticker.tick().await;
                if let Err(e) = service.collect_metrics().await {
                    warn!("Metric collection failed: {}", e);
                }
            }
        });

        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        info!("Stopping monitoring service");
        
        // Stop collectors
        self.bandwidth_collector.stop().await?;
        self.latency_collector.stop().await?;
        self.packet_loss_collector.stop().await?;
        self.flow_stats_collector.stop().await?;
        
        // Detach eBPF
        self.detach_ebpf_probes().await?;
        
        Ok(())
    }

    async fn get_network_metrics(&self) -> Result<NetworkMetrics> {
        Ok(self.aggregator.aggregate_network())
    }

    async fn get_link_metrics(&self, link_id: &str) -> Result<LinkMetrics> {
        self.aggregator.get_link(link_id)
            .ok_or_else(|| {
                crate::error::MonitoringError::InvalidInterface(link_id.to_string())
            })
    }
}

impl Clone for MonitoringService {
    fn clone(&self) -> Self {
        Self {
            bandwidth_collector: Arc::clone(&self.bandwidth_collector),
            latency_collector: Arc::clone(&self.latency_collector),
            packet_loss_collector: Arc::clone(&self.packet_loss_collector),
            flow_stats_collector: Arc::clone(&self.flow_stats_collector),
            ebpf_manager: Arc::clone(&self.ebpf_manager),
            aggregator: Arc::clone(&self.aggregator),
            interval_ms: self.interval_ms,
            ebpf_enabled: self.ebpf_enabled,
            link_metrics: Arc::clone(&self.link_metrics),
            traffic_samples: Arc::clone(&self.traffic_samples),
        }
    }
}

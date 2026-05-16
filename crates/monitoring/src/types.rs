use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficSample {
    pub timestamp: DateTime<Utc>,
    pub src_ip: IpAddr,
    pub dst_ip: IpAddr,
    pub src_port: u16,
    pub dst_port: u16,
    pub protocol: u8,
    pub packet_size: u32,
    pub flow_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkMetrics {
    pub link_id: String,
    pub bandwidth_bps: u64,
    pub utilization: f64,
    pub latency_ms: f64,
    pub packet_loss: f64,
    pub jitter_ms: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub total_bandwidth: u64,
    pub active_flows: u32,
    pub avg_latency_ms: f64,
    pub packet_loss_rate: f64,
    pub link_metrics: Vec<LinkMetrics>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowMetrics {
    pub flow_id: String,
    pub throughput_bps: u64,
    pub packet_count: u64,
    pub byte_count: u64,
    pub duration_ms: u64,
}

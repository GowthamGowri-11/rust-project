use serde::{Deserialize, Serialize};

/// Configuration for eBPF probes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeConfig {
    pub interface: String,
    pub sample_rate: u32,
    pub buffer_size: usize,
    pub capture_payload: bool,
}

impl Default for ProbeConfig {
    fn default() -> Self {
        Self {
            interface: "eth0".to_string(),
            sample_rate: 1000, // Sample every 1000th packet
            buffer_size: 8192,
            capture_payload: false,
        }
    }
}

/// Packet probe for eBPF monitoring
pub struct PacketProbe {
    config: ProbeConfig,
}

impl PacketProbe {
    pub fn new(config: ProbeConfig) -> Self {
        Self { config }
    }

    pub fn interface(&self) -> &str {
        &self.config.interface
    }

    pub fn sample_rate(&self) -> u32 {
        self.config.sample_rate
    }
}

/// eBPF program types
#[derive(Debug, Clone, Copy)]
pub enum ProbeType {
    Xdp,        // XDP (eXpress Data Path) - earliest hook point
    TcIngress,  // TC ingress - after XDP
    TcEgress,   // TC egress - outgoing packets
    Kprobe,     // Kernel probe - function entry
    Kretprobe,  // Kernel return probe - function exit
}

/// Probe attachment point
#[derive(Debug, Clone)]
pub struct AttachPoint {
    pub probe_type: ProbeType,
    pub interface: Option<String>,
    pub function: Option<String>,
}

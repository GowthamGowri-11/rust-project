use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};
use uuid::Uuid;

pub type SwitchId = u64;
pub type FlowId = Uuid;
pub type PortNumber = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpenFlowVersion {
    V1_0 = 0x01,
    V1_3 = 0x04,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchInfo {
    pub datapath_id: SwitchId,
    pub ip_address: IpAddr,
    pub port: u16,
    pub version: OpenFlowVersion,
    pub num_ports: u32,
    pub connected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortInfo {
    pub port_number: PortNumber,
    pub hw_addr: [u8; 6],
    pub name: String,
    pub state: PortState,
    pub curr_speed: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortState {
    Up,
    Down,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowMatch {
    pub in_port: Option<PortNumber>,
    pub eth_src: Option<[u8; 6]>,
    pub eth_dst: Option<[u8; 6]>,
    pub eth_type: Option<u16>,
    pub vlan_id: Option<u16>,
    pub ip_src: Option<Ipv4Addr>,
    pub ip_dst: Option<Ipv4Addr>,
    pub ip_proto: Option<u8>,
    pub tcp_src: Option<u16>,
    pub tcp_dst: Option<u16>,
    pub udp_src: Option<u16>,
    pub udp_dst: Option<u16>,
}

impl Default for FlowMatch {
    fn default() -> Self {
        Self {
            in_port: None,
            eth_src: None,
            eth_dst: None,
            eth_type: None,
            vlan_id: None,
            ip_src: None,
            ip_dst: None,
            ip_proto: None,
            tcp_src: None,
            tcp_dst: None,
            udp_src: None,
            udp_dst: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FlowAction {
    Output { port: PortNumber },
    SetVlan { vlan_id: u16 },
    SetQueue { queue_id: u32 },
    SetDlSrc { addr: [u8; 6] },
    SetDlDst { addr: [u8; 6] },
    SetNwSrc { addr: Ipv4Addr },
    SetNwDst { addr: Ipv4Addr },
    Drop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowRule {
    pub id: FlowId,
    pub switch_id: SwitchId,
    pub priority: u16,
    pub match_fields: FlowMatch,
    pub actions: Vec<FlowAction>,
    pub idle_timeout: u16,
    pub hard_timeout: u16,
    pub cookie: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowStats {
    pub flow_id: FlowId,
    pub packet_count: u64,
    pub byte_count: u64,
    pub duration_sec: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub src_switch: SwitchId,
    pub src_port: PortNumber,
    pub dst_switch: SwitchId,
    pub dst_port: PortNumber,
    pub latency_ms: f64,
    pub bandwidth_mbps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTopology {
    pub switches: Vec<SwitchInfo>,
    pub links: Vec<Link>,
}

use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use uuid::Uuid;

pub type SwitchId = String;
pub type FlowId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Switch {
    pub id: SwitchId,
    pub datapath_id: u64,
    pub ip_address: IpAddr,
    pub port: u16,
    pub connected: bool,
    pub num_ports: u32,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowRule {
    pub id: FlowId,
    pub switch_id: SwitchId,
    pub priority: u16,
    pub match_fields: MatchFields,
    pub actions: Vec<Action>,
    pub idle_timeout: u16,
    pub hard_timeout: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchFields {
    pub in_port: Option<u32>,
    pub eth_src: Option<String>,
    pub eth_dst: Option<String>,
    pub eth_type: Option<u16>,
    pub ip_src: Option<IpAddr>,
    pub ip_dst: Option<IpAddr>,
    pub ip_proto: Option<u8>,
    pub tcp_src: Option<u16>,
    pub tcp_dst: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    Output { port: u32 },
    SetVlan { vlan_id: u16 },
    SetQueue { queue_id: u32 },
    Drop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowStats {
    pub flow_id: FlowId,
    pub packet_count: u64,
    pub byte_count: u64,
    pub duration_sec: u32,
}

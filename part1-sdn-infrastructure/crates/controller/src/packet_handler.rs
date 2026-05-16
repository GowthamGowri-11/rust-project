use shared::*;
use tracing::{debug, info};

pub struct PacketHandler;

impl PacketHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn handle_packet_in(&self, switch_id: SwitchId, packet: PacketInMessage) -> Result<Vec<FlowAction>> {
        debug!(
            "Handling packet-in from switch {}, port {}, reason {:?}",
            switch_id, packet.in_port, packet.reason
        );

        // Simple L2 learning switch logic
        let actions = vec![FlowAction::Output {
            port: 0xFFFF, // OFPP_FLOOD
        }];

        Ok(actions)
    }

    pub fn parse_ethernet_frame(&self, data: &[u8]) -> Option<EthernetFrame> {
        if data.len() < 14 {
            return None;
        }

        let mut dst_mac = [0u8; 6];
        let mut src_mac = [0u8; 6];

        dst_mac.copy_from_slice(&data[0..6]);
        src_mac.copy_from_slice(&data[6..12]);

        let eth_type = u16::from_be_bytes([data[12], data[13]]);

        Some(EthernetFrame {
            dst_mac,
            src_mac,
            eth_type,
            payload: data[14..].to_vec(),
        })
    }
}

impl Default for PacketHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct EthernetFrame {
    pub dst_mac: [u8; 6],
    pub src_mac: [u8; 6],
    pub eth_type: u16,
    pub payload: Vec<u8>,
}

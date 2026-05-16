use bytes::{BufMut, BytesMut};
use dashmap::DashMap;
use shared::*;
use std::sync::Arc;
use tracing::{debug, info, warn};

pub struct SwitchManager {
    switches: Arc<DashMap<SwitchId, SwitchInfo>>,
}

impl SwitchManager {
    pub fn new() -> Self {
        Self {
            switches: Arc::new(DashMap::new()),
        }
    }

    pub fn register_switch(&self, switch: SwitchInfo) {
        info!("Registering switch {}", switch.datapath_id);
        self.switches.insert(switch.datapath_id, switch);
    }

    pub fn unregister_switch(&self, switch_id: SwitchId) {
        info!("Unregistering switch {}", switch_id);
        self.switches.remove(&switch_id);
    }

    pub fn get_switch(&self, switch_id: SwitchId) -> Option<SwitchInfo> {
        self.switches.get(&switch_id).map(|s| s.clone())
    }

    pub fn get_all_switches(&self) -> Vec<SwitchInfo> {
        self.switches
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    pub fn is_connected(&self, switch_id: SwitchId) -> bool {
        self.switches
            .get(&switch_id)
            .map(|s| s.connected)
            .unwrap_or(false)
    }

    pub fn update_switch_state(&self, switch_id: SwitchId, connected: bool) {
        if let Some(mut switch) = self.switches.get_mut(&switch_id) {
            switch.connected = connected;
        }
    }

    pub fn build_features_request(&self) -> BytesMut {
        let mut buf = BytesMut::new();

        let header = OpenFlowHeader::new(
            0x01, // OpenFlow 1.0
            MessageType::FeaturesRequest,
            8,    // Header only
            1,    // XID
        );
        header.encode(&mut buf);

        buf
    }

    pub fn build_hello_message(&self) -> BytesMut {
        let mut buf = BytesMut::new();

        let header = OpenFlowHeader::new(
            0x01, // OpenFlow 1.0
            MessageType::Hello,
            8,    // Header only
            0,    // XID
        );
        header.encode(&mut buf);

        buf
    }

    pub fn switch_count(&self) -> usize {
        self.switches.len()
    }
}

impl Default for SwitchManager {
    fn default() -> Self {
        Self::new()
    }
}

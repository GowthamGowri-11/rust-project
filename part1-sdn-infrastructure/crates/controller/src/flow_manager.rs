use bytes::{BufMut, BytesMut};
use dashmap::DashMap;
use shared::*;
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;

pub struct FlowManager {
    flows: Arc<DashMap<FlowId, FlowRule>>,
    switch_flows: Arc<DashMap<SwitchId, Vec<FlowId>>>,
}

impl FlowManager {
    pub fn new() -> Self {
        Self {
            flows: Arc::new(DashMap::new()),
            switch_flows: Arc::new(DashMap::new()),
        }
    }

    pub fn install_flow(&self, rule: FlowRule) -> Result<FlowId> {
        let flow_id = rule.id;
        let switch_id = rule.switch_id;

        info!(
            "Installing flow {} on switch {} with priority {}",
            flow_id, switch_id, rule.priority
        );

        self.flows.insert(flow_id, rule);

        self.switch_flows
            .entry(switch_id)
            .or_insert_with(Vec::new)
            .push(flow_id);

        Ok(flow_id)
    }

    pub fn remove_flow(&self, flow_id: FlowId) -> Result<()> {
        if let Some((_, rule)) = self.flows.remove(&flow_id) {
            info!("Removing flow {} from switch {}", flow_id, rule.switch_id);

            if let Some(mut flows) = self.switch_flows.get_mut(&rule.switch_id) {
                flows.retain(|&id| id != flow_id);
            }

            Ok(())
        } else {
            Err(Error::NotFound(format!("Flow {} not found", flow_id)))
        }
    }

    pub fn get_flow(&self, flow_id: FlowId) -> Option<FlowRule> {
        self.flows.get(&flow_id).map(|f| f.clone())
    }

    pub fn get_switch_flows(&self, switch_id: SwitchId) -> Vec<FlowRule> {
        if let Some(flow_ids) = self.switch_flows.get(&switch_id) {
            flow_ids
                .iter()
                .filter_map(|id| self.flows.get(id).map(|f| f.clone()))
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn clear_switch_flows(&self, switch_id: SwitchId) {
        if let Some((_, flow_ids)) = self.switch_flows.remove(&switch_id) {
            for flow_id in flow_ids {
                self.flows.remove(&flow_id);
            }
        }
    }

    pub fn build_flow_mod_message(&self, rule: &FlowRule) -> BytesMut {
        let mut buf = BytesMut::new();

        // OpenFlow header
        let header = OpenFlowHeader::new(
            0x01, // OpenFlow 1.0
            MessageType::FlowMod,
            88,   // Base length
            0,    // XID
        );
        header.encode(&mut buf);

        // Flow mod body (simplified)
        buf.put_u64(rule.cookie);
        buf.put_u16(rule.priority);
        buf.put_u16(rule.idle_timeout);
        buf.put_u16(rule.hard_timeout);

        buf
    }

    pub fn flow_count(&self) -> usize {
        self.flows.len()
    }
}

impl Default for FlowManager {
    fn default() -> Self {
        Self::new()
    }
}

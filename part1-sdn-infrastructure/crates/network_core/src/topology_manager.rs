use dashmap::DashMap;
use shared::{Link, NetworkTopology, PortInfo, Result, SwitchId, SwitchInfo};
use std::sync::Arc;
use tracing::{debug, info};

pub struct TopologyManager {
    switches: Arc<DashMap<SwitchId, SwitchInfo>>,
    links: Arc<DashMap<(SwitchId, SwitchId), Link>>,
}

impl TopologyManager {
    pub fn new() -> Self {
        Self {
            switches: Arc::new(DashMap::new()),
            links: Arc::new(DashMap::new()),
        }
    }

    pub fn add_switch(&self, switch: SwitchInfo) {
        info!("Adding switch {} to topology", switch.datapath_id);
        self.switches.insert(switch.datapath_id, switch);
    }

    pub fn remove_switch(&self, switch_id: SwitchId) {
        info!("Removing switch {} from topology", switch_id);
        self.switches.remove(&switch_id);
        
        // Remove all links connected to this switch
        self.links.retain(|(src, dst), _| *src != switch_id && *dst != switch_id);
    }

    pub fn add_link(&self, link: Link) {
        debug!(
            "Adding link: {} -> {}",
            link.src_switch, link.dst_switch
        );
        let key = (link.src_switch, link.dst_switch);
        self.links.insert(key, link);
    }

    pub fn remove_link(&self, src: SwitchId, dst: SwitchId) {
        debug!("Removing link: {} -> {}", src, dst);
        self.links.remove(&(src, dst));
    }

    pub fn get_switch(&self, switch_id: SwitchId) -> Option<SwitchInfo> {
        self.switches.get(&switch_id).map(|s| s.clone())
    }

    pub fn get_topology(&self) -> NetworkTopology {
        let switches: Vec<SwitchInfo> = self
            .switches
            .iter()
            .map(|entry| entry.value().clone())
            .collect();

        let links: Vec<Link> = self
            .links
            .iter()
            .map(|entry| entry.value().clone())
            .collect();

        NetworkTopology { switches, links }
    }

    pub fn get_neighbors(&self, switch_id: SwitchId) -> Vec<SwitchId> {
        self.links
            .iter()
            .filter_map(|entry| {
                let (src, dst) = entry.key();
                if *src == switch_id {
                    Some(*dst)
                } else if *dst == switch_id {
                    Some(*src)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn switch_count(&self) -> usize {
        self.switches.len()
    }

    pub fn link_count(&self) -> usize {
        self.links.len()
    }
}

impl Default for TopologyManager {
    fn default() -> Self {
        Self::new()
    }
}

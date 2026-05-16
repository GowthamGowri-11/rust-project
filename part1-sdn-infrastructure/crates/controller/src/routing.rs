use shared::*;
use std::collections::{HashMap, VecDeque};
use tracing::debug;

pub struct RoutingEngine {
    topology: NetworkTopology,
}

impl RoutingEngine {
    pub fn new() -> Self {
        Self {
            topology: NetworkTopology {
                switches: Vec::new(),
                links: Vec::new(),
            },
        }
    }

    pub fn update_topology(&mut self, topology: NetworkTopology) {
        self.topology = topology;
    }

    pub fn find_shortest_path(&self, src: SwitchId, dst: SwitchId) -> Option<Vec<SwitchId>> {
        if src == dst {
            return Some(vec![src]);
        }

        let mut queue = VecDeque::new();
        let mut visited = HashMap::new();
        let mut parent = HashMap::new();

        queue.push_back(src);
        visited.insert(src, true);

        while let Some(current) = queue.pop_front() {
            if current == dst {
                return Some(self.reconstruct_path(&parent, src, dst));
            }

            for link in &self.topology.links {
                let next = if link.src_switch == current {
                    link.dst_switch
                } else if link.dst_switch == current {
                    link.src_switch
                } else {
                    continue;
                };

                if !visited.contains_key(&next) {
                    visited.insert(next, true);
                    parent.insert(next, current);
                    queue.push_back(next);
                }
            }
        }

        None
    }

    fn reconstruct_path(
        &self,
        parent: &HashMap<SwitchId, SwitchId>,
        src: SwitchId,
        dst: SwitchId,
    ) -> Vec<SwitchId> {
        let mut path = Vec::new();
        let mut current = dst;

        while current != src {
            path.push(current);
            current = *parent.get(&current).unwrap();
        }

        path.push(src);
        path.reverse();
        path
    }

    pub fn compute_flow_rules(&self, path: &[SwitchId]) -> Vec<FlowRule> {
        let mut rules = Vec::new();

        for i in 0..path.len() - 1 {
            let current_switch = path[i];
            let next_switch = path[i + 1];

            if let Some(out_port) = self.find_output_port(current_switch, next_switch) {
                let rule = FlowRule {
                    id: uuid::Uuid::new_v4(),
                    switch_id: current_switch,
                    priority: 100,
                    match_fields: FlowMatch::default(),
                    actions: vec![FlowAction::Output { port: out_port }],
                    idle_timeout: 0,
                    hard_timeout: 0,
                    cookie: 0,
                };

                rules.push(rule);
            }
        }

        rules
    }

    fn find_output_port(&self, src: SwitchId, dst: SwitchId) -> Option<PortNumber> {
        self.topology
            .links
            .iter()
            .find(|link| link.src_switch == src && link.dst_switch == dst)
            .map(|link| link.src_port)
    }
}

impl Default for RoutingEngine {
    fn default() -> Self {
        Self::new()
    }
}

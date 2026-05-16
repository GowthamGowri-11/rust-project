use std::collections::HashMap;

/// Network graph representation
pub struct NetworkGraph {
    nodes: HashMap<String, Node>,
    adjacency: HashMap<String, Vec<(String, Link)>>,
}

impl NetworkGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            adjacency: HashMap::new(),
        }
    }

    /// Add node to graph
    pub fn add_node(&mut self, node: Node) {
        let node_id = node.id.clone();
        self.nodes.insert(node_id.clone(), node);
        self.adjacency.entry(node_id).or_insert_with(Vec::new);
    }

    /// Add link between nodes
    pub fn add_link(&mut self, from: String, to: String, link: Link) {
        self.adjacency
            .entry(from.clone())
            .or_insert_with(Vec::new)
            .push((to.clone(), link.clone()));
        
        // Add reverse link for undirected graph
        self.adjacency
            .entry(to)
            .or_insert_with(Vec::new)
            .push((from, link));
    }

    /// Get neighbors of a node
    pub fn get_neighbors(&self, node_id: &str) -> Option<&Vec<(String, Link)>> {
        self.adjacency.get(node_id)
    }

    /// Get link between two nodes
    pub fn get_link(&self, from: &str, to: &str) -> Option<&Link> {
        self.adjacency.get(from)?.iter()
            .find(|(neighbor, _)| neighbor == to)
            .map(|(_, link)| link)
    }

    /// Get all nodes
    pub fn get_nodes(&self) -> Vec<&Node> {
        self.nodes.values().collect()
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    pub id: String,
    pub name: String,
    pub node_type: NodeType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NodeType {
    Switch,
    Router,
    Host,
}

#[derive(Debug, Clone)]
pub struct Link {
    pub id: String,
    pub cost: f64,
    pub capacity: u64,
    pub available_bandwidth: u64,
    pub latency_ms: f64,
    pub packet_loss: f64,
}

impl Default for NetworkGraph {
    fn default() -> Self {
        Self::new()
    }
}

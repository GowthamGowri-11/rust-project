use crate::error::Result;
use crate::graph::{NetworkGraph, Link};
use crate::types::*;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::cmp::Ordering;
use tracing::debug;

/// Path selection engine
pub struct PathSelector {
    algorithm: PathAlgorithm,
}

impl PathSelector {
    pub fn new(algorithm: PathAlgorithm) -> Self {
        Self { algorithm }
    }

    /// Find optimal path between source and destination
    pub fn find_path(
        &self,
        graph: &NetworkGraph,
        source: &str,
        destination: &str,
        constraints: &PathConstraints,
    ) -> Result<Option<Path>> {
        match self.algorithm {
            PathAlgorithm::Dijkstra => self.dijkstra(graph, source, destination, constraints),
            PathAlgorithm::AStar => self.astar(graph, source, destination, constraints),
            PathAlgorithm::ConstraintBased => {
                self.constraint_based(graph, source, destination, constraints)
            }
        }
    }

    /// Find K shortest paths
    pub fn find_k_paths(
        &self,
        graph: &NetworkGraph,
        source: &str,
        destination: &str,
        k: usize,
        constraints: &PathConstraints,
    ) -> Result<Vec<Path>> {
        debug!("Finding {} shortest paths from {} to {}", k, source, destination);
        
        let mut paths = Vec::new();
        let mut excluded_edges = HashSet::new();

        for i in 0..k {
            if let Some(path) = self.find_path_excluding(
                graph,
                source,
                destination,
                constraints,
                &excluded_edges,
            )? {
                // Exclude one edge from this path for next iteration
                if i < k - 1 && path.nodes.len() > 1 {
                    let edge = (path.nodes[0].clone(), path.nodes[1].clone());
                    excluded_edges.insert(edge);
                }
                paths.push(path);
            } else {
                break;
            }
        }

        Ok(paths)
    }

    /// Dijkstra's shortest path algorithm
    fn dijkstra(
        &self,
        graph: &NetworkGraph,
        source: &str,
        destination: &str,
        constraints: &PathConstraints,
    ) -> Result<Option<Path>> {
        let mut distances: HashMap<String, f64> = HashMap::new();
        let mut previous: HashMap<String, String> = HashMap::new();
        let mut heap = BinaryHeap::new();

        distances.insert(source.to_string(), 0.0);
        heap.push(State {
            cost: 0.0,
            node: source.to_string(),
        });

        while let Some(State { cost, node }) = heap.pop() {
            if node == destination {
                return Ok(Some(self.reconstruct_path(
                    &previous,
                    source,
                    destination,
                    graph,
                )?));
            }

            if cost > *distances.get(&node).unwrap_or(&f64::INFINITY) {
                continue;
            }

            if let Some(neighbors) = graph.get_neighbors(&node) {
                for (neighbor, link) in neighbors {
                    if !self.satisfies_constraints(link, constraints) {
                        continue;
                    }

                    let new_cost = cost + link.cost;
                    let neighbor_cost = *distances.get(neighbor).unwrap_or(&f64::INFINITY);

                    if new_cost < neighbor_cost {
                        distances.insert(neighbor.clone(), new_cost);
                        previous.insert(neighbor.clone(), node.clone());
                        heap.push(State {
                            cost: new_cost,
                            node: neighbor.clone(),
                        });
                    }
                }
            }
        }

        Ok(None)
    }

    /// A* pathfinding algorithm
    fn astar(
        &self,
        graph: &NetworkGraph,
        source: &str,
        destination: &str,
        constraints: &PathConstraints,
    ) -> Result<Option<Path>> {
        // TODO: Implement A* with heuristic
        // For now, fall back to Dijkstra
        self.dijkstra(graph, source, destination, constraints)
    }

    /// Constraint-based routing
    fn constraint_based(
        &self,
        graph: &NetworkGraph,
        source: &str,
        destination: &str,
        constraints: &PathConstraints,
    ) -> Result<Option<Path>> {
        // Find path that satisfies all constraints
        self.dijkstra(graph, source, destination, constraints)
    }

    /// Find path excluding certain edges
    fn find_path_excluding(
        &self,
        graph: &NetworkGraph,
        source: &str,
        destination: &str,
        constraints: &PathConstraints,
        excluded: &HashSet<(String, String)>,
    ) -> Result<Option<Path>> {
        // TODO: Implement with edge exclusion
        self.dijkstra(graph, source, destination, constraints)
    }

    /// Check if link satisfies constraints
    fn satisfies_constraints(&self, link: &Link, constraints: &PathConstraints) -> bool {
        if let Some(max_latency) = constraints.max_latency_ms {
            if link.latency_ms > max_latency {
                return false;
            }
        }

        if let Some(min_bandwidth) = constraints.min_bandwidth_bps {
            if link.available_bandwidth < min_bandwidth {
                return false;
            }
        }

        if let Some(max_loss) = constraints.max_packet_loss {
            if link.packet_loss > max_loss {
                return false;
            }
        }

        true
    }

    /// Reconstruct path from previous map
    fn reconstruct_path(
        &self,
        previous: &HashMap<String, String>,
        source: &str,
        destination: &str,
        graph: &NetworkGraph,
    ) -> Result<Path> {
        let mut nodes = Vec::new();
        let mut current = destination.to_string();

        while current != source {
            nodes.push(current.clone());
            current = previous
                .get(&current)
                .ok_or_else(|| crate::error::OptimizerError::NoPathFound {
                    src: source.to_string(),
                    dst: destination.to_string(),
                })?
                .clone();
        }
        nodes.push(source.to_string());
        nodes.reverse();

        // Calculate path metrics
        let mut total_cost = 0.0;
        let mut total_latency = 0.0;
        let mut min_bandwidth = u64::MAX;

        for i in 0..nodes.len() - 1 {
            if let Some(link) = graph.get_link(&nodes[i], &nodes[i + 1]) {
                total_cost += link.cost;
                total_latency += link.latency_ms;
                min_bandwidth = min_bandwidth.min(link.available_bandwidth);
            }
        }

        Ok(Path {
            nodes,
            cost: total_cost,
            bandwidth: min_bandwidth,
            latency_ms: total_latency,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathAlgorithm {
    Dijkstra,
    AStar,
    ConstraintBased,
}

#[derive(Debug, Clone)]
pub struct PathConstraints {
    pub max_latency_ms: Option<f64>,
    pub min_bandwidth_bps: Option<u64>,
    pub max_packet_loss: Option<f64>,
    pub max_hops: Option<usize>,
}

impl Default for PathConstraints {
    fn default() -> Self {
        Self {
            max_latency_ms: None,
            min_bandwidth_bps: None,
            max_packet_loss: None,
            max_hops: None,
        }
    }
}

#[derive(Debug)]
struct State {
    cost: f64,
    node: String,
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost
            .partial_cmp(&self.cost)
            .unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}

impl Eq for State {}

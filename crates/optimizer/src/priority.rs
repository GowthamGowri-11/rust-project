use crate::types::*;
use std::cmp::Ordering;

/// Traffic priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Critical = 5,
    High = 4,
    Medium = 3,
    Low = 2,
    BestEffort = 1,
}

/// Traffic prioritization engine
pub struct PriorityEngine {
    default_priority: Priority,
}

impl PriorityEngine {
    pub fn new() -> Self {
        Self {
            default_priority: Priority::Medium,
        }
    }

    /// Assign priority to flow based on traffic class
    pub fn assign_priority(&self, traffic_class: &TrafficClass) -> Priority {
        match traffic_class {
            TrafficClass::RealTime => Priority::Critical,
            TrafficClass::Interactive => Priority::High,
            TrafficClass::Streaming => Priority::Medium,
            TrafficClass::BestEffort => Priority::Low,
            TrafficClass::Background => Priority::BestEffort,
        }
    }

    /// Compare two flows by priority
    pub fn compare_flows(&self, flow1: &PrioritizedFlow, flow2: &PrioritizedFlow) -> Ordering {
        flow1.priority.cmp(&flow2.priority)
    }

    /// Sort flows by priority (highest first)
    pub fn sort_flows(&self, flows: &mut [PrioritizedFlow]) {
        flows.sort_by(|a, b| b.priority.cmp(&a.priority));
    }
}

#[derive(Debug, Clone)]
pub struct PrioritizedFlow {
    pub flow_id: String,
    pub priority: Priority,
    pub traffic_class: TrafficClass,
    pub bandwidth_requirement: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrafficClass {
    RealTime,
    Interactive,
    Streaming,
    BestEffort,
    Background,
}

impl Default for PriorityEngine {
    fn default() -> Self {
        Self::new()
    }
}

use crate::error::Result;
use crate::rules::Rule;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// Policy engine for routing decisions
pub struct PolicyEngine {
    policies: HashMap<String, Policy>,
}

impl PolicyEngine {
    pub fn new() -> Self {
        Self {
            policies: HashMap::new(),
        }
    }

    /// Add policy
    pub fn add_policy(&mut self, policy: Policy) -> Result<()> {
        info!("Adding policy: {}", policy.name);
        self.policies.insert(policy.id.clone(), policy);
        Ok(())
    }

    /// Remove policy
    pub fn remove_policy(&mut self, policy_id: &str) -> Result<()> {
        info!("Removing policy: {}", policy_id);
        self.policies.remove(policy_id);
        Ok(())
    }

    /// Get policy
    pub fn get_policy(&self, policy_id: &str) -> Option<&Policy> {
        self.policies.get(policy_id)
    }

    /// Evaluate policies for a routing decision
    pub fn evaluate(&self, context: &RoutingContext) -> Result<PolicyDecision> {
        debug!("Evaluating policies for flow: {}", context.flow_id);

        let mut applicable_policies: Vec<&Policy> = self
            .policies
            .values()
            .filter(|p| p.enabled && self.matches_context(p, context))
            .collect();

        // Sort by priority (highest first)
        applicable_policies.sort_by(|a, b| b.priority.cmp(&a.priority));

        // Apply first matching policy
        for policy in applicable_policies {
            if let Some(decision) = self.apply_policy(policy, context) {
                return Ok(decision);
            }
        }

        // Default decision
        Ok(PolicyDecision::default())
    }

    fn matches_context(&self, policy: &Policy, context: &RoutingContext) -> bool {
        // Check if policy applies to this context
        for rule in &policy.rules {
            if rule.matches(context) {
                return true;
            }
        }
        false
    }

    fn apply_policy(&self, policy: &Policy, context: &RoutingContext) -> Option<PolicyDecision> {
        debug!("Applying policy: {}", policy.name);

        Some(PolicyDecision {
            policy_id: policy.id.clone(),
            allow: true,
            constraints: policy.constraints.clone(),
            priority: policy.priority,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub id: String,
    pub name: String,
    pub policy_type: PolicyType,
    pub priority: u32,
    pub enabled: bool,
    pub rules: Vec<Rule>,
    pub constraints: PolicyConstraints,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PolicyType {
    Sla,           // Service Level Agreement
    QoS,           // Quality of Service
    Security,      // Security policies
    LoadBalancing, // Load balancing policies
    Routing,       // Routing policies
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConstraints {
    pub max_latency_ms: Option<f64>,
    pub min_bandwidth_bps: Option<u64>,
    pub max_packet_loss: Option<f64>,
    pub required_availability: Option<f64>,
    pub allowed_paths: Option<Vec<String>>,
    pub forbidden_paths: Option<Vec<String>>,
}

impl Default for PolicyConstraints {
    fn default() -> Self {
        Self {
            max_latency_ms: None,
            min_bandwidth_bps: None,
            max_packet_loss: None,
            required_availability: None,
            allowed_paths: None,
            forbidden_paths: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RoutingContext {
    pub flow_id: String,
    pub source: String,
    pub destination: String,
    pub traffic_class: String,
    pub bandwidth_requirement: u64,
}

#[derive(Debug, Clone)]
pub struct PolicyDecision {
    pub policy_id: String,
    pub allow: bool,
    pub constraints: PolicyConstraints,
    pub priority: u32,
}

impl Default for PolicyDecision {
    fn default() -> Self {
        Self {
            policy_id: "default".to_string(),
            allow: true,
            constraints: PolicyConstraints::default(),
            priority: 0,
        }
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}

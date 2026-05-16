use crate::policies::RoutingContext;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub condition: RuleCondition,
    pub action: RuleAction,
}

impl Rule {
    pub fn matches(&self, context: &RoutingContext) -> bool {
        self.condition.evaluate(context)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleCondition {
    TrafficClass(String),
    SourcePrefix(String),
    DestinationPrefix(String),
    BandwidthGreaterThan(u64),
    BandwidthLessThan(u64),
    And(Vec<RuleCondition>),
    Or(Vec<RuleCondition>),
    Not(Box<RuleCondition>),
}

impl RuleCondition {
    pub fn evaluate(&self, context: &RoutingContext) -> bool {
        match self {
            RuleCondition::TrafficClass(class) => &context.traffic_class == class,
            RuleCondition::SourcePrefix(prefix) => context.source.starts_with(prefix),
            RuleCondition::DestinationPrefix(prefix) => context.destination.starts_with(prefix),
            RuleCondition::BandwidthGreaterThan(threshold) => {
                context.bandwidth_requirement > *threshold
            }
            RuleCondition::BandwidthLessThan(threshold) => {
                context.bandwidth_requirement < *threshold
            }
            RuleCondition::And(conditions) => {
                conditions.iter().all(|c| c.evaluate(context))
            }
            RuleCondition::Or(conditions) => {
                conditions.iter().any(|c| c.evaluate(context))
            }
            RuleCondition::Not(condition) => !condition.evaluate(context),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleAction {
    Allow,
    Deny,
    SetPriority(u32),
    SetPath(String),
    SetConstraints {
        max_latency_ms: Option<f64>,
        min_bandwidth_bps: Option<u64>,
    },
}

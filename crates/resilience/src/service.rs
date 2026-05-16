use crate::{
    detection::{FailureDetector, LinkFailureDetector, TrafficSpikeDetector},
    error::Result,
    recovery::{AutoRecoveryEngine, RecoveryEngine, RecoveryStrategy},
    types::*,
};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, info, warn};

#[async_trait]
pub trait Resilience: Send + Sync {
    async fn detect_failures(&self) -> Result<Vec<FailureEvent>>;
    async fn recover_from_failure(&self, event: FailureEvent) -> Result<RecoveryAction>;
    async fn get_backup_path(&self, failed_link: String) -> Result<Vec<String>>;
    async fn start_monitoring(&self) -> Result<()>;
    async fn stop_monitoring(&self) -> Result<()>;
}

pub struct ResilienceService {
    link_detector: Arc<LinkFailureDetector>,
    spike_detector: Arc<TrafficSpikeDetector>,
    recovery_engine: Arc<AutoRecoveryEngine>,
}

impl ResilienceService {
    pub fn new() -> Self {
        Self {
            link_detector: Arc::new(LinkFailureDetector::new(5000, 1000)),
            spike_detector: Arc::new(TrafficSpikeDetector::new(2.0)),
            recovery_engine: Arc::new(AutoRecoveryEngine::new(
                RecoveryStrategy::FastFailover,
                30000,
            )),
        }
    }

    pub fn with_config(
        link_timeout_ms: u64,
        check_interval_ms: u64,
        spike_threshold: f64,
        recovery_strategy: RecoveryStrategy,
    ) -> Self {
        Self {
            link_detector: Arc::new(LinkFailureDetector::new(link_timeout_ms, check_interval_ms)),
            spike_detector: Arc::new(TrafficSpikeDetector::new(spike_threshold)),
            recovery_engine: Arc::new(AutoRecoveryEngine::new(recovery_strategy, 30000)),
        }
    }

    /// Update link heartbeat
    pub fn heartbeat(&self, link_id: &str) {
        self.link_detector.heartbeat(link_id);
    }

    /// Update traffic baseline
    pub fn update_traffic(&self, link_id: &str, traffic_bps: u64) {
        self.spike_detector.update_baseline(link_id, traffic_bps);
    }

    /// Check for traffic spike
    pub fn check_spike(&self, link_id: &str, current_traffic: u64) -> bool {
        self.spike_detector.detect_spike(link_id, current_traffic)
    }
}

#[async_trait]
impl Resilience for ResilienceService {
    async fn detect_failures(&self) -> Result<Vec<FailureEvent>> {
        debug!("Detecting network failures");
        
        // Detect link failures
        let mut failures = self.link_detector.detect().await?;
        
        info!("Detected {} failures", failures.len());
        Ok(failures)
    }

    async fn recover_from_failure(&self, event: FailureEvent) -> Result<RecoveryAction> {
        warn!("Recovering from failure: {:?}", event.failure_type);
        
        // Use recovery engine to handle failure
        let action = self.recovery_engine.recover(event).await?;
        
        // Validate recovery
        let success = self.recovery_engine.validate_recovery(&action).await?;
        
        if success {
            info!("Recovery action {} completed successfully", action.action_id);
        } else {
            warn!("Recovery action {} validation failed", action.action_id);
        }
        
        Ok(action)
    }

    async fn get_backup_path(&self, failed_link: String) -> Result<Vec<String>> {
        debug!("Finding backup path for: {}", failed_link);
        
        // Get backup path from recovery engine
        if let Some(backup) = self.recovery_engine.get_backup_path(&failed_link) {
            Ok(backup.clone())
        } else {
            Ok(vec![])
        }
    }

    async fn start_monitoring(&self) -> Result<()> {
        info!("Starting resilience monitoring");
        self.link_detector.start().await?;
        Ok(())
    }

    async fn stop_monitoring(&self) -> Result<()> {
        info!("Stopping resilience monitoring");
        self.link_detector.stop().await?;
        Ok(())
    }
}

impl Default for ResilienceService {
    fn default() -> Self {
        Self::new()
    }
}

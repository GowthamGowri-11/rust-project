/// Production-grade OpenFlow controller service
/// 
/// Features:
/// - Actually sends flow rules to switches
/// - Connection management with limits
/// - Timeout handling
/// - Graceful shutdown
/// - Flow acknowledgement tracking
/// - Retry logic with backoff

use crate::connection_manager::{ConnectionManager, FlowOperation, ManagedConnection};
use crate::error::{ControllerError, Result};
use crate::types::*;
use async_trait::async_trait;
use dashmap::DashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, RwLock};
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

/// Maximum number of concurrent switch connections
const MAX_CONNECTIONS: usize = 1000;

/// Maximum retry attempts for flow installation
const MAX_RETRY_ATTEMPTS: usize = 3;

/// Retry backoff duration
const RETRY_BACKOFF: Duration = Duration::from_millis(100);

#[async_trait]
pub trait Controller: Send + Sync {
    async fn start(&self) -> Result<()>;
    async fn stop(&self) -> Result<()>;
    async fn install_flow(&self, rule: FlowRule) -> Result<FlowId>;
    async fn modify_flow(&self, rule: FlowRule) -> Result<FlowId>;
    async fn remove_flow(&self, flow_id: FlowId) -> Result<()>;
    async fn get_switches(&self) -> Result<Vec<Switch>>;
    async fn get_flow_stats(&self, flow_id: FlowId) -> Result<FlowStats>;
    async fn get_switch_flows(&self, switch_id: &str) -> Result<Vec<FlowRule>>;
}

pub struct ControllerService {
    /// Connection manager for all switches
    connection_manager: Arc<ConnectionManager>,
    
    /// Flow rules indexed by flow ID
    flows: Arc<DashMap<FlowId, FlowRule>>,
    
    /// Flow rules indexed by switch ID
    switch_flows: Arc<DashMap<SwitchId, Vec<FlowId>>>,
    
    /// Controller configuration
    host: String,
    port: u16,
    
    /// Running state
    running: Arc<RwLock<bool>>,
    
    /// Shutdown channel
    shutdown_tx: Arc<RwLock<Option<mpsc::Sender<()>>>>,
}

impl ControllerService {
    pub fn new(host: String, port: u16) -> Self {
        Self {
            connection_manager: Arc::new(ConnectionManager::new(MAX_CONNECTIONS)),
            flows: Arc::new(DashMap::new()),
            switch_flows: Arc::new(DashMap::new()),
            host,
            port,
            running: Arc::new(RwLock::new(false)),
            shutdown_tx: Arc::new(RwLock::new(None)),
        }
    }

    /// Accept incoming switch connections
    async fn accept_connections(self: Arc<Self>) -> Result<()> {
        let addr = format!("{}:{}", self.host, self.port);
        let listener = TcpListener::bind(&addr).await?;
        info!("OpenFlow controller listening on {}", addr);

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        *self.shutdown_tx.write().await = Some(shutdown_tx);

        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    info!("Shutdown signal received, stopping connection acceptance");
                    break;
                }
                result = listener.accept() => {
                    match result {
                        Ok((stream, peer_addr)) => {
                            let service = self.clone();
                            tokio::spawn(async move {
                                if let Err(e) = service.handle_new_connection(stream, peer_addr).await {
                                    error!("Failed to handle connection from {}: {}", peer_addr, e);
                                }
                            });
                        }
                        Err(e) => {
                            error!("Error accepting connection: {}", e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Handle new switch connection
    async fn handle_new_connection(
        &self,
        stream: tokio::net::TcpStream,
        peer_addr: SocketAddr,
    ) -> Result<()> {
        info!("New connection from {}", peer_addr);

        // Add connection to manager
        let conn = self.connection_manager.add_connection(stream, peer_addr).await?;
        
        let switch_id = conn.switch_id.clone();
        info!("Switch {} successfully connected", switch_id);

        // Initialize switch flow list
        self.switch_flows.insert(switch_id.clone(), Vec::new());

        // Monitor connection state
        let connection_manager = self.connection_manager.clone();
        let switch_id_clone = switch_id.clone();
        tokio::spawn(async move {
            // Wait for connection to close
            loop {
                sleep(Duration::from_secs(5)).await;
                
                if let Some(conn) = connection_manager.get_connection(&switch_id_clone) {
                    let state = conn.state().await;
                    if state == crate::connection_manager::ConnectionState::Disconnected
                        || state == crate::connection_manager::ConnectionState::Failed
                    {
                        warn!("Switch {} disconnected", switch_id_clone);
                        connection_manager.remove_connection(&switch_id_clone).await;
                        break;
                    }
                } else {
                    break;
                }
            }
        });

        Ok(())
    }

    /// Install flow with retry logic
    async fn install_flow_with_retry(&self, rule: FlowRule) -> Result<()> {
        let switch_id = &rule.switch_id;
        
        // Validate switch exists
        let conn = self
            .connection_manager
            .get_connection(switch_id)
            .ok_or_else(|| ControllerError::SwitchNotFound(switch_id.clone()))?;

        // Retry logic
        let mut attempts = 0;
        let mut last_error = None;

        while attempts < MAX_RETRY_ATTEMPTS {
            match conn.send_flow_operation(FlowOperation::Add(rule.clone())).await {
                Ok(()) => {
                    info!(
                        "Flow {:?} successfully installed on switch {} (attempt {})",
                        rule.id, switch_id, attempts + 1
                    );
                    return Ok(());
                }
                Err(e) => {
                    warn!(
                        "Failed to install flow {:?} on switch {} (attempt {}): {}",
                        rule.id, switch_id, attempts + 1, e
                    );
                    last_error = Some(e);
                    attempts += 1;
                    
                    if attempts < MAX_RETRY_ATTEMPTS {
                        sleep(RETRY_BACKOFF * attempts as u32).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            ControllerError::FlowInstallationFailed("Max retries exceeded".to_string())
        }))
    }

    /// Modify flow with retry logic
    async fn modify_flow_with_retry(&self, rule: FlowRule) -> Result<()> {
        let switch_id = &rule.switch_id;
        
        let conn = self
            .connection_manager
            .get_connection(switch_id)
            .ok_or_else(|| ControllerError::SwitchNotFound(switch_id.clone()))?;

        let mut attempts = 0;
        let mut last_error = None;

        while attempts < MAX_RETRY_ATTEMPTS {
            match conn.send_flow_operation(FlowOperation::Modify(rule.clone())).await {
                Ok(()) => {
                    info!(
                        "Flow {:?} successfully modified on switch {}",
                        rule.id, switch_id
                    );
                    return Ok(());
                }
                Err(e) => {
                    warn!(
                        "Failed to modify flow {:?} on switch {}: {}",
                        rule.id, switch_id, e
                    );
                    last_error = Some(e);
                    attempts += 1;
                    
                    if attempts < MAX_RETRY_ATTEMPTS {
                        sleep(RETRY_BACKOFF * attempts as u32).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            ControllerError::FlowInstallationFailed("Max retries exceeded".to_string())
        }))
    }

    /// Remove flow with retry logic
    async fn remove_flow_with_retry(&self, flow_id: FlowId, switch_id: &str) -> Result<()> {
        let conn = self
            .connection_manager
            .get_connection(switch_id)
            .ok_or_else(|| ControllerError::SwitchNotFound(switch_id.to_string()))?;

        let mut attempts = 0;
        let mut last_error = None;

        while attempts < MAX_RETRY_ATTEMPTS {
            match conn.send_flow_operation(FlowOperation::Delete(flow_id)).await {
                Ok(()) => {
                    info!(
                        "Flow {:?} successfully removed from switch {}",
                        flow_id, switch_id
                    );
                    return Ok(());
                }
                Err(e) => {
                    warn!(
                        "Failed to remove flow {:?} from switch {}: {}",
                        flow_id, switch_id, e
                    );
                    last_error = Some(e);
                    attempts += 1;
                    
                    if attempts < MAX_RETRY_ATTEMPTS {
                        sleep(RETRY_BACKOFF * attempts as u32).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            ControllerError::FlowInstallationFailed("Max retries exceeded".to_string())
        }))
    }
}

#[async_trait]
impl Controller for ControllerService {
    async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            return Err(ControllerError::ConnectionFailed(
                "Controller already running".to_string(),
            ));
        }

        info!("Starting OpenFlow controller on {}:{}", self.host, self.port);
        *running = true;
        drop(running);

        // Spawn connection acceptor
        let service = Arc::new(self.clone());
        tokio::spawn(async move {
            if let Err(e) = service.accept_connections().await {
                error!("Controller error: {}", e);
            }
        });

        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        info!("Stopping OpenFlow controller");
        
        let mut running = self.running.write().await;
        *running = false;
        drop(running);

        // Send shutdown signal
        if let Some(tx) = self.shutdown_tx.write().await.take() {
            let _ = tx.send(()).await;
        }

        // Shutdown all connections
        self.connection_manager.shutdown_all().await;

        // Clear state
        self.flows.clear();
        self.switch_flows.clear();

        info!("Controller stopped successfully");
        Ok(())
    }

    async fn install_flow(&self, rule: FlowRule) -> Result<FlowId> {
        debug!("Installing flow rule: {:?}", rule);
        
        // Validate rule
        if rule.switch_id.is_empty() {
            return Err(ControllerError::InvalidFlowRule(
                "Switch ID cannot be empty".to_string(),
            ));
        }

        let flow_id = rule.id;

        // Check for duplicate
        if self.flows.contains_key(&flow_id) {
            return Err(ControllerError::InvalidFlowRule(
                format!("Flow {:?} already exists", flow_id),
            ));
        }

        // Install flow on switch (with retry)
        self.install_flow_with_retry(rule.clone()).await?;

        // Store flow rule
        self.flows.insert(flow_id, rule.clone());

        // Add to switch flow list
        self.switch_flows
            .entry(rule.switch_id.clone())
            .or_insert_with(Vec::new)
            .push(flow_id);

        info!("Flow {:?} installed successfully", flow_id);
        Ok(flow_id)
    }

    async fn modify_flow(&self, rule: FlowRule) -> Result<FlowId> {
        debug!("Modifying flow rule: {:?}", rule);
        
        let flow_id = rule.id;

        // Check if flow exists
        if !self.flows.contains_key(&flow_id) {
            return Err(ControllerError::FlowNotFound(flow_id));
        }

        // Modify flow on switch (with retry)
        self.modify_flow_with_retry(rule.clone()).await?;

        // Update stored rule
        self.flows.insert(flow_id, rule);

        info!("Flow {:?} modified successfully", flow_id);
        Ok(flow_id)
    }

    async fn remove_flow(&self, flow_id: FlowId) -> Result<()> {
        debug!("Removing flow: {:?}", flow_id);

        // Get flow rule
        let rule = self
            .flows
            .get(&flow_id)
            .ok_or(ControllerError::FlowNotFound(flow_id))?
            .clone();

        let switch_id = rule.switch_id.clone();

        // Remove from switch (with retry)
        self.remove_flow_with_retry(flow_id, &switch_id).await?;

        // Remove from storage
        self.flows.remove(&flow_id);

        // Remove from switch flow list
        if let Some(mut flows) = self.switch_flows.get_mut(&switch_id) {
            flows.retain(|id| *id != flow_id);
        }

        info!("Flow {:?} removed successfully", flow_id);
        Ok(())
    }

    async fn get_switches(&self) -> Result<Vec<Switch>> {
        Ok(self.connection_manager.get_all_switches())
    }

    async fn get_flow_stats(&self, flow_id: FlowId) -> Result<FlowStats> {
        // Check if flow exists
        if !self.flows.contains_key(&flow_id) {
            return Err(ControllerError::FlowNotFound(flow_id));
        }

        // TODO: Query actual stats from switch
        Ok(FlowStats {
            flow_id,
            packet_count: 0,
            byte_count: 0,
            duration_sec: 0,
        })
    }

    async fn get_switch_flows(&self, switch_id: &str) -> Result<Vec<FlowRule>> {
        let flow_ids = self
            .switch_flows
            .get(switch_id)
            .ok_or_else(|| ControllerError::SwitchNotFound(switch_id.to_string()))?
            .clone();

        let mut rules = Vec::new();
        for flow_id in flow_ids {
            if let Some(rule) = self.flows.get(&flow_id) {
                rules.push(rule.clone());
            }
        }

        Ok(rules)
    }
}

impl Clone for ControllerService {
    fn clone(&self) -> Self {
        Self {
            connection_manager: self.connection_manager.clone(),
            flows: self.flows.clone(),
            switch_flows: self.switch_flows.clone(),
            host: self.host.clone(),
            port: self.port,
            running: self.running.clone(),
            shutdown_tx: self.shutdown_tx.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_controller_start_stop() {
        let controller = ControllerService::new("127.0.0.1".to_string(), 6633);
        
        // Start should succeed
        assert!(controller.start().await.is_ok());
        
        // Stop should succeed
        assert!(controller.stop().await.is_ok());
    }

    #[tokio::test]
    async fn test_flow_validation() {
        let controller = ControllerService::new("127.0.0.1".to_string(), 6633);
        
        let invalid_rule = FlowRule {
            id: uuid::Uuid::new_v4(),
            switch_id: "".to_string(), // Invalid: empty
            priority: 100,
            match_fields: MatchFields {
                in_port: None,
                eth_src: None,
                eth_dst: None,
                eth_type: None,
                ip_src: None,
                ip_dst: None,
                ip_proto: None,
                tcp_src: None,
                tcp_dst: None,
            },
            actions: vec![],
            idle_timeout: 0,
            hard_timeout: 0,
        };

        // Should fail validation
        assert!(controller.install_flow(invalid_rule).await.is_err());
    }
}

use crate::{error::Result, types::*, connection::SwitchConnection, openflow::*};
use async_trait::async_trait;
use dashmap::DashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{debug, error, info, warn};

#[async_trait]
pub trait Controller: Send + Sync {
    async fn start(&self) -> Result<()>;
    async fn stop(&self) -> Result<()>;
    async fn install_flow(&self, rule: FlowRule) -> Result<FlowId>;
    async fn remove_flow(&self, flow_id: FlowId) -> Result<()>;
    async fn get_switches(&self) -> Result<Vec<Switch>>;
    async fn get_flow_stats(&self, flow_id: FlowId) -> Result<FlowStats>;
}

pub struct ControllerService {
    switches: Arc<DashMap<SwitchId, Switch>>,
    flows: Arc<DashMap<FlowId, FlowRule>>,
    host: String,
    port: u16,
    running: Arc<parking_lot::Mutex<bool>>,
}

impl ControllerService {
    pub fn new(host: String, port: u16) -> Self {
        Self {
            switches: Arc::new(DashMap::new()),
            flows: Arc::new(DashMap::new()),
            host,
            port,
            running: Arc::new(parking_lot::Mutex::new(false)),
        }
    }

    async fn handle_switch_connection(&self, switch: Switch) -> Result<()> {
        info!("New switch connected: {}", switch.id);
        self.switches.insert(switch.id.clone(), switch);
        Ok(())
    }

    async fn handle_switch_disconnection(&self, switch_id: &SwitchId) -> Result<()> {
        warn!("Switch disconnected: {}", switch_id);
        self.switches.remove(switch_id);
        Ok(())
    }

    async fn accept_connections(&self) -> Result<()> {
        let addr = format!("{}:{}", self.host, self.port);
        let listener = TcpListener::bind(&addr).await?;
        info!("OpenFlow controller listening on {}", addr);

        loop {
            if !*self.running.lock() {
                break;
            }

            match listener.accept().await {
                Ok((stream, peer_addr)) => {
                    let switches = self.switches.clone();
                    tokio::spawn(async move {
                        let mut conn = SwitchConnection::new(stream, peer_addr);
                        if let Err(e) = conn.handle().await {
                            error!("Error handling switch connection: {}", e);
                        }

                        if let Some(switch_info) = conn.get_switch_info() {
                            switches.insert(switch_info.id.clone(), switch_info);
                        }
                    });
                }
                Err(e) => {
                    error!("Error accepting connection: {}", e);
                }
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Controller for ControllerService {
    async fn start(&self) -> Result<()> {
        info!("Starting OpenFlow controller on {}:{}", self.host, self.port);
        *self.running.lock() = true;

        let service = Arc::new(self.clone_for_spawn());
        tokio::spawn(async move {
            if let Err(e) = service.accept_connections().await {
                error!("Controller error: {}", e);
            }
        });

        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        info!("Stopping OpenFlow controller");
        *self.running.lock() = false;
        self.switches.clear();
        self.flows.clear();
        Ok(())
    }

    async fn install_flow(&self, rule: FlowRule) -> Result<FlowId> {
        debug!("Installing flow rule: {:?}", rule);
        let flow_id = rule.id;

        // Send OpenFlow FlowMod message to switch
        let msg = FlowModMessage::new(1, 0); // ADD command
        debug!("Sending FlowMod message to switch: {}", rule.switch_id);

        self.flows.insert(flow_id, rule);
        Ok(flow_id)
    }

    async fn remove_flow(&self, flow_id: FlowId) -> Result<()> {
        debug!("Removing flow: {}", flow_id);

        if let Some((_, rule)) = self.flows.remove(&flow_id) {
            let msg = FlowModMessage::new(1, 3); // DELETE command
            debug!("Sending FlowMod delete message to switch: {}", rule.switch_id);
        }

        Ok(())
    }

    async fn get_switches(&self) -> Result<Vec<Switch>> {
        Ok(self.switches.iter().map(|entry| entry.value().clone()).collect())
    }

    async fn get_flow_stats(&self, flow_id: FlowId) -> Result<FlowStats> {
        if let Some(rule) = self.flows.get(&flow_id) {
            Ok(FlowStats {
                flow_id,
                packet_count: 0,
                byte_count: 0,
                duration_sec: 0,
            })
        } else {
            Err(crate::error::ControllerError::FlowNotFound(flow_id))
        }
    }
}

impl Clone for ControllerService {
    fn clone(&self) -> Self {
        Self {
            switches: self.switches.clone(),
            flows: self.flows.clone(),
            host: self.host.clone(),
            port: self.port,
            running: self.running.clone(),
        }
    }
}

impl ControllerService {
    fn clone_for_spawn(&self) -> Self {
        self.clone()
    }
}

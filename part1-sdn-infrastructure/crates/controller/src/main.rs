use async_trait::async_trait;
use bytes::Bytes;
use controller::*;
use network_core::*;
use shared::*;
use std::sync::Arc;
use tracing::{error, info};
use tracing_subscriber;

struct ControllerApp {
    switch_manager: Arc<SwitchManager>,
    flow_manager: Arc<FlowManager>,
    topology_manager: Arc<TopologyManager>,
    routing_engine: Arc<parking_lot::RwLock<RoutingEngine>>,
    packet_handler: Arc<PacketHandler>,
    connection_manager: Arc<ConnectionManager>,
}

impl ControllerApp {
    fn new(connection_manager: Arc<ConnectionManager>) -> Self {
        Self {
            switch_manager: Arc::new(SwitchManager::new()),
            flow_manager: Arc::new(FlowManager::new()),
            topology_manager: Arc::new(TopologyManager::new()),
            routing_engine: Arc::new(parking_lot::RwLock::new(RoutingEngine::new())),
            packet_handler: Arc::new(PacketHandler::new()),
            connection_manager,
        }
    }
}

#[async_trait]
impl connection::ConnectionHandler for ControllerApp {
    async fn handle_message(&self, switch_id: SwitchId, message: Bytes) -> Result<()> {
        info!("Received message from switch {}", switch_id);
        Ok(())
    }

    async fn handle_disconnect(&self, switch_id: SwitchId) {
        info!("Switch {} disconnected", switch_id);
        self.switch_manager.unregister_switch(switch_id);
        self.topology_manager.remove_switch(switch_id);
        self.flow_manager.clear_switch_flows(switch_id);
    }
}

#[async_trait]
impl dispatcher::EventHandler for ControllerApp {
    async fn on_switch_connected(&self, switch_id: SwitchId) {
        info!("Switch {} connected", switch_id);

        let hello_msg = self.switch_manager.build_hello_message();
        if let Err(e) = self
            .connection_manager
            .send_message(switch_id, hello_msg.freeze())
        {
            error!("Failed to send hello: {}", e);
        }

        let features_req = self.switch_manager.build_features_request();
        if let Err(e) = self
            .connection_manager
            .send_message(switch_id, features_req.freeze())
        {
            error!("Failed to send features request: {}", e);
        }
    }

    async fn on_switch_disconnected(&self, switch_id: SwitchId) {
        info!("Switch {} disconnected", switch_id);
        self.switch_manager.unregister_switch(switch_id);
    }

    async fn on_packet_in(&self, switch_id: SwitchId, _message: SwitchMessage) {
        info!("Packet-in from switch {}", switch_id);
    }

    async fn on_features_reply(&self, switch_id: SwitchId, _message: SwitchMessage) {
        info!("Features reply from switch {}", switch_id);
    }

    async fn on_port_status(&self, switch_id: SwitchId, _message: SwitchMessage) {
        info!("Port status from switch {}", switch_id);
    }

    async fn on_flow_removed(&self, switch_id: SwitchId, _message: SwitchMessage) {
        info!("Flow removed from switch {}", switch_id);
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info,controller=debug")
        .init();

    info!("Starting RustFlow-AI OpenFlow Controller");

    let connection_manager = Arc::new(ConnectionManager::new("0.0.0.0:6653".to_string()));
    let controller = Arc::new(ControllerApp::new(Arc::clone(&connection_manager)));

    connection_manager.start(Arc::clone(&controller) as Arc<dyn connection::ConnectionHandler>).await?;

    info!("Controller started successfully");
    info!("Listening for OpenFlow connections on 0.0.0.0:6653");

    tokio::signal::ctrl_c().await?;
    info!("Shutting down controller");

    Ok(())
}

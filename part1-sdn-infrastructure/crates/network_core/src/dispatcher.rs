use async_trait::async_trait;
use bytes::Bytes;
use shared::{MessageType, OpenFlowHeader, Result, SwitchId, SwitchMessage};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, warn};

#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn on_switch_connected(&self, switch_id: SwitchId);
    async fn on_switch_disconnected(&self, switch_id: SwitchId);
    async fn on_packet_in(&self, switch_id: SwitchId, message: SwitchMessage);
    async fn on_features_reply(&self, switch_id: SwitchId, message: SwitchMessage);
    async fn on_port_status(&self, switch_id: SwitchId, message: SwitchMessage);
    async fn on_flow_removed(&self, switch_id: SwitchId, message: SwitchMessage);
}

pub enum Event {
    SwitchConnected(SwitchId),
    SwitchDisconnected(SwitchId),
    Message {
        switch_id: SwitchId,
        message: Bytes,
    },
}

pub struct EventDispatcher {
    tx: mpsc::UnboundedSender<Event>,
    rx: Option<mpsc::UnboundedReceiver<Event>>,
}

impl EventDispatcher {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self { tx, rx: Some(rx) }
    }

    pub fn sender(&self) -> mpsc::UnboundedSender<Event> {
        self.tx.clone()
    }

    pub async fn run<H: EventHandler>(mut self, handler: Arc<H>) {
        let mut rx = self.rx.take().expect("Dispatcher already running");

        while let Some(event) = rx.recv().await {
            match event {
                Event::SwitchConnected(switch_id) => {
                    debug!("Dispatching switch connected event: {}", switch_id);
                    handler.on_switch_connected(switch_id).await;
                }
                Event::SwitchDisconnected(switch_id) => {
                    debug!("Dispatching switch disconnected event: {}", switch_id);
                    handler.on_switch_disconnected(switch_id).await;
                }
                Event::Message { switch_id, message } => {
                    if let Err(e) = Self::dispatch_message(&handler, switch_id, message).await {
                        error!("Error dispatching message: {}", e);
                    }
                }
            }
        }
    }

    async fn dispatch_message<H: EventHandler>(
        handler: &Arc<H>,
        switch_id: SwitchId,
        mut message: Bytes,
    ) -> Result<()> {
        let header = OpenFlowHeader::decode(&mut message)
            .ok_or_else(|| shared::Error::InvalidMessage("Invalid header".to_string()))?;

        debug!(
            "Dispatching message type {:?} from switch {}",
            header.msg_type, switch_id
        );

        match header.msg_type {
            MessageType::Hello => {
                debug!("Received Hello from switch {}", switch_id);
            }
            MessageType::FeaturesReply => {
                let msg = SwitchMessage::Hello; // Placeholder
                handler.on_features_reply(switch_id, msg).await;
            }
            MessageType::PacketIn => {
                let msg = SwitchMessage::Hello; // Placeholder
                handler.on_packet_in(switch_id, msg).await;
            }
            MessageType::PortStatus => {
                let msg = SwitchMessage::Hello; // Placeholder
                handler.on_port_status(switch_id, msg).await;
            }
            MessageType::FlowRemoved => {
                let msg = SwitchMessage::Hello; // Placeholder
                handler.on_flow_removed(switch_id, msg).await;
            }
            MessageType::EchoRequest => {
                debug!("Received Echo Request from switch {}", switch_id);
                // TODO: Send Echo Reply
            }
            _ => {
                warn!("Unhandled message type: {:?}", header.msg_type);
            }
        }

        Ok(())
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

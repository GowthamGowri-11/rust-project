use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use tokio::sync::mpsc;

/// Packet event from eBPF probe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketEvent {
    pub timestamp: DateTime<Utc>,
    pub interface: String,
    pub src_ip: IpAddr,
    pub dst_ip: IpAddr,
    pub src_port: u16,
    pub dst_port: u16,
    pub protocol: u8,
    pub packet_size: u32,
    pub direction: PacketDirection,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PacketDirection {
    Ingress,
    Egress,
}

/// Event stream for async packet processing
pub struct EventStream {
    receiver: mpsc::Receiver<PacketEvent>,
}

impl EventStream {
    pub fn new(receiver: mpsc::Receiver<PacketEvent>) -> Self {
        Self { receiver }
    }

    /// Receive next packet event
    pub async fn recv(&mut self) -> Option<PacketEvent> {
        self.receiver.recv().await
    }

    /// Try to receive without blocking
    pub fn try_recv(&mut self) -> Result<PacketEvent, mpsc::error::TryRecvError> {
        self.receiver.try_recv()
    }
}

/// Event producer for eBPF data
pub struct EventProducer {
    sender: mpsc::Sender<PacketEvent>,
}

impl EventProducer {
    pub fn new(sender: mpsc::Sender<PacketEvent>) -> Self {
        Self { sender }
    }

    /// Send packet event (non-blocking)
    pub async fn send(&self, event: PacketEvent) -> Result<(), mpsc::error::SendError<PacketEvent>> {
        self.sender.send(event).await
    }

    /// Try to send without blocking
    pub fn try_send(&self, event: PacketEvent) -> Result<(), mpsc::error::TrySendError<PacketEvent>> {
        self.sender.try_send(event)
    }
}

/// Create event channel for packet streaming
pub fn create_event_channel(buffer_size: usize) -> (EventProducer, EventStream) {
    let (sender, receiver) = mpsc::channel(buffer_size);
    (EventProducer::new(sender), EventStream::new(receiver))
}

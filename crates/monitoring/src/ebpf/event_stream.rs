use crate::error::Result;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::info;

/// Represents a packet event from eBPF
#[derive(Debug, Clone)]
pub struct PacketEvent {
    pub timestamp: u64,
    pub src_ip: u32,
    pub dst_ip: u32,
    pub src_port: u16,
    pub dst_port: u16,
    pub protocol: u8,
    pub packet_len: u16,
    pub bytes: u64,
}

/// Represents a latency event from eBPF
#[derive(Debug, Clone)]
pub struct LatencyEvent {
    pub timestamp: u64,
    pub latency_ns: u64,
    pub src_ip: u32,
    pub dst_ip: u32,
}

/// Event stream handler for eBPF events
pub struct EventStream {
    packet_tx: mpsc::UnboundedSender<PacketEvent>,
    packet_rx: Arc<tokio::sync::Mutex<mpsc::UnboundedReceiver<PacketEvent>>>,
    latency_tx: mpsc::UnboundedSender<LatencyEvent>,
    latency_rx: Arc<tokio::sync::Mutex<mpsc::UnboundedReceiver<LatencyEvent>>>,
    event_count: Arc<DashMap<String, u64>>,
    running: Arc<parking_lot::Mutex<bool>>,
}

impl EventStream {
    pub fn new() -> Self {
        let (packet_tx, packet_rx) = mpsc::unbounded_channel();
        let (latency_tx, latency_rx) = mpsc::unbounded_channel();

        Self {
            packet_tx,
            packet_rx: Arc::new(tokio::sync::Mutex::new(packet_rx)),
            latency_tx,
            latency_rx: Arc::new(tokio::sync::Mutex::new(latency_rx)),
            event_count: Arc::new(DashMap::new()),
            running: Arc::new(parking_lot::Mutex::new(false)),
        }
    }

    /// Start the event stream
    pub async fn start(&self) -> Result<()> {
        info!("Starting event stream");
        *self.running.lock() = true;
        Ok(())
    }

    /// Stop the event stream
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping event stream");
        *self.running.lock() = false;
        Ok(())
    }

    /// Send a packet event
    pub fn send_packet_event(&self, event: PacketEvent) -> Result<()> {
        if !*self.running.lock() {
            return Ok(());
        }

        self.packet_tx.send(event).ok();
        if let Some(mut entry) = self.event_count.get_mut("packet_events") {
            *entry += 1;
        } else {
            self.event_count.insert("packet_events".to_string(), 1);
        }

        Ok(())
    }

    /// Send a latency event
    pub fn send_latency_event(&self, event: LatencyEvent) -> Result<()> {
        if !*self.running.lock() {
            return Ok(());
        }

        self.latency_tx.send(event).ok();
        if let Some(mut entry) = self.event_count.get_mut("latency_events") {
            *entry += 1;
        } else {
            self.event_count.insert("latency_events".to_string(), 1);
        }

        Ok(())
    }

    /// Receive packet events (non-blocking)
    pub async fn recv_packet_event(&self) -> Option<PacketEvent> {
        let mut rx = self.packet_rx.lock().await;
        rx.recv().await
    }

    /// Receive latency events (non-blocking)
    pub async fn recv_latency_event(&self) -> Option<LatencyEvent> {
        let mut rx = self.latency_rx.lock().await;
        rx.recv().await
    }

    /// Get event statistics
    pub fn get_stats(&self) -> Vec<(String, u64)> {
        self.event_count
            .iter()
            .map(|entry| (entry.key().clone(), *entry.value()))
            .collect()
    }

    /// Reset event counters
    pub fn reset_stats(&self) {
        self.event_count.clear();
    }

    /// Process packet events in batch
    pub async fn process_packet_events<F>(&self, mut handler: F) -> Result<()>
    where
        F: FnMut(PacketEvent) -> Result<()>,
    {
        while let Some(event) = self.recv_packet_event().await {
            handler(event)?;
        }
        Ok(())
    }

    /// Process latency events in batch
    pub async fn process_latency_events<F>(&self, mut handler: F) -> Result<()>
    where
        F: FnMut(LatencyEvent) -> Result<()>,
    {
        while let Some(event) = self.recv_latency_event().await {
            handler(event)?;
        }
        Ok(())
    }
}

impl Default for EventStream {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for EventStream {
    fn clone(&self) -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_stream_creation() {
        let stream = EventStream::new();
        assert!(stream.start().await.is_ok());
        assert!(stream.stop().await.is_ok());
    }

    #[tokio::test]
    async fn test_packet_event_send_recv() {
        let stream = EventStream::new();
        stream.start().await.unwrap();

        let event = PacketEvent {
            timestamp: 1000,
            src_ip: 0x7F000001,
            dst_ip: 0x7F000002,
            src_port: 8080,
            dst_port: 80,
            protocol: 6,
            packet_len: 1024,
            bytes: 1024,
        };

        stream.send_packet_event(event.clone()).unwrap();
        let received = stream.recv_packet_event().await;
        assert!(received.is_some());
    }

    #[tokio::test]
    async fn test_event_statistics() {
        let stream = EventStream::new();
        stream.start().await.unwrap();

        let event = PacketEvent {
            timestamp: 1000,
            src_ip: 0x7F000001,
            dst_ip: 0x7F000002,
            src_port: 8080,
            dst_port: 80,
            protocol: 6,
            packet_len: 1024,
            bytes: 1024,
        };

        stream.send_packet_event(event).unwrap();
        let stats = stream.get_stats();
        assert!(!stats.is_empty());
    }
}

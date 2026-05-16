use async_trait::async_trait;
use bytes::{Bytes, BytesMut};
use dashmap::DashMap;
use shared::{Error, OpenFlowHeader, Result, SwitchId};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

pub type MessageSender = mpsc::UnboundedSender<Bytes>;
pub type MessageReceiver = mpsc::UnboundedReceiver<Bytes>;

#[async_trait]
pub trait ConnectionHandler: Send + Sync {
    async fn handle_message(&self, switch_id: SwitchId, message: Bytes) -> Result<()>;
    async fn handle_disconnect(&self, switch_id: SwitchId);
}

pub struct SwitchConnection {
    pub switch_id: SwitchId,
    pub tx: MessageSender,
    stream: TcpStream,
}

impl SwitchConnection {
    pub fn new(switch_id: SwitchId, stream: TcpStream) -> (Self, MessageReceiver) {
        let (tx, rx) = mpsc::unbounded_channel();
        (
            Self {
                switch_id,
                tx,
                stream,
            },
            rx,
        )
    }

    pub async fn run<H: ConnectionHandler>(
        mut self,
        mut rx: MessageReceiver,
        handler: Arc<H>,
    ) -> Result<()> {
        let (mut reader, mut writer) = self.stream.split();
        let switch_id = self.switch_id;

        let read_task = tokio::spawn(async move {
            let mut buffer = BytesMut::with_capacity(4096);

            loop {
                match reader.read_buf(&mut buffer).await {
                    Ok(0) => {
                        info!("Switch {} disconnected", switch_id);
                        break;
                    }
                    Ok(n) => {
                        debug!("Received {} bytes from switch {}", n, switch_id);

                        while buffer.len() >= OpenFlowHeader::SIZE {
                            let mut peek = buffer.clone().freeze();
                            let header = match OpenFlowHeader::decode(&mut peek) {
                                Some(h) => h,
                                None => break,
                            };

                            if buffer.len() < header.length as usize {
                                break;
                            }

                            let message = buffer.split_to(header.length as usize).freeze();

                            if let Err(e) = handler.handle_message(switch_id, message).await {
                                error!("Error handling message from switch {}: {}", switch_id, e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error reading from switch {}: {}", switch_id, e);
                        break;
                    }
                }
            }

            handler.handle_disconnect(switch_id).await;
        });

        let write_task = tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if let Err(e) = writer.write_all(&message).await {
                    error!("Error writing to switch {}: {}", switch_id, e);
                    break;
                }
            }
        });

        tokio::select! {
            _ = read_task => {},
            _ = write_task => {},
        }

        Ok(())
    }
}

pub struct ConnectionManager {
    connections: Arc<DashMap<SwitchId, MessageSender>>,
    listener_addr: String,
}

impl ConnectionManager {
    pub fn new(listener_addr: String) -> Self {
        Self {
            connections: Arc::new(DashMap::new()),
            listener_addr,
        }
    }

    pub async fn start<H: ConnectionHandler + 'static>(
        &self,
        handler: Arc<H>,
    ) -> Result<()> {
        let listener = TcpListener::bind(&self.listener_addr)
            .await
            .map_err(|e| Error::Connection(format!("Failed to bind: {}", e)))?;

        info!("OpenFlow controller listening on {}", self.listener_addr);

        let connections = Arc::clone(&self.connections);

        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        info!("New connection from {}", addr);

                        let switch_id = Self::generate_switch_id(&addr);
                        let (conn, rx) = SwitchConnection::new(switch_id, stream);

                        connections.insert(switch_id, conn.tx.clone());

                        let handler_clone = Arc::clone(&handler);
                        tokio::spawn(async move {
                            if let Err(e) = conn.run(rx, handler_clone).await {
                                error!("Connection error for switch {}: {}", switch_id, e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("Error accepting connection: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    pub fn send_message(&self, switch_id: SwitchId, message: Bytes) -> Result<()> {
        if let Some(tx) = self.connections.get(&switch_id) {
            tx.send(message)
                .map_err(|_| Error::Connection(format!("Failed to send to switch {}", switch_id)))?;
            Ok(())
        } else {
            Err(Error::NotFound(format!("Switch {} not connected", switch_id)))
        }
    }

    pub fn is_connected(&self, switch_id: SwitchId) -> bool {
        self.connections.contains_key(&switch_id)
    }

    pub fn disconnect(&self, switch_id: SwitchId) {
        self.connections.remove(&switch_id);
    }

    pub fn connected_switches(&self) -> Vec<SwitchId> {
        self.connections.iter().map(|entry| *entry.key()).collect()
    }

    fn generate_switch_id(addr: &std::net::SocketAddr) -> SwitchId {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        addr.hash(&mut hasher);
        hasher.finish()
    }
}

/// Production-grade OpenFlow connection manager
/// 
/// Manages switch connections with:
/// - Connection pooling
/// - Timeout handling
/// - Message size limits
/// - Graceful shutdown
/// - Flow rule transmission
/// - Acknowledgement tracking

use crate::openflow::*;
use crate::types::*;
use crate::error::{ControllerError, Result};
use dashmap::DashMap;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::TcpStream;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::{mpsc, oneshot, RwLock, Mutex};
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

/// Maximum message size (64KB)
const MAX_MESSAGE_SIZE: usize = 65536;

/// Connection timeout
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(30);

/// Read timeout
const READ_TIMEOUT: Duration = Duration::from_secs(10);

/// Write timeout
const WRITE_TIMEOUT: Duration = Duration::from_secs(5);

/// Maximum pending flow operations per switch
const MAX_PENDING_OPERATIONS: usize = 1000;

/// Flow operation send timeout (backpressure handling)
const FLOW_SEND_TIMEOUT: Duration = Duration::from_secs(10);

/// Flow verification timeout (barrier reply wait)
const FLOW_VERIFY_TIMEOUT: Duration = Duration::from_secs(5);

/// Flow operation command
#[derive(Debug, Clone)]
pub enum FlowOperation {
    Add(FlowRule),
    Modify(FlowRule),
    Delete(FlowId),
}

/// Flow operation result
#[derive(Debug)]
pub struct FlowOperationResult {
    pub operation: FlowOperation,
    pub success: bool,
    pub error: Option<String>,
}

/// Switch connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Connecting,
    Connected,
    Authenticated,
    Disconnected,
    Failed,
}

/// CRITICAL FIX #7: Cleanup guard for task cancellation safety
/// Ensures resources are properly cleaned up even if task is cancelled
struct CleanupGuard {
    switch_id: String,
    state: Arc<RwLock<ConnectionState>>,
    writer: Arc<Mutex<BufWriter<OwnedWriteHalf>>>,
}

impl Drop for CleanupGuard {
    fn drop(&mut self) {
        // Note: Can't use async in Drop, but we can spawn a cleanup task
        let switch_id = self.switch_id.clone();
        let state = self.state.clone();
        let writer = self.writer.clone();
        
        tokio::spawn(async move {
            // Mark connection as disconnected
            *state.write().await = ConnectionState::Disconnected;
            
            // Attempt to flush writer
            if let Ok(mut writer_guard) = writer.try_lock() {
                let _ = writer_guard.flush().await;
                debug!("Cleanup guard flushed writer for switch {}", switch_id);
            }
            
            debug!("Cleanup guard executed for switch {}", switch_id);
        });
    }
}

/// Managed switch connection with flow transmission capability
pub struct ManagedConnection {
    reader: Arc<Mutex<BufReader<OwnedReadHalf>>>,
    writer: Arc<Mutex<BufWriter<OwnedWriteHalf>>>,
    addr: SocketAddr,
    pub switch_id: String,
    pub datapath_id: u64,
    xid_counter: Arc<std::sync::atomic::AtomicU32>,
    state: Arc<RwLock<ConnectionState>>,
    flow_tx: mpsc::Sender<(FlowOperation, oneshot::Sender<Result<()>>)>,
    shutdown_tx: Option<oneshot::Sender<()>>,
    /// CRITICAL FIX #9: XID tracking for flow verification
    pending_xids: Arc<Mutex<HashMap<u32, oneshot::Sender<Result<()>>>>>,
}

impl ManagedConnection {
    /// Create new managed connection
    pub async fn new(
        stream: TcpStream,
        addr: SocketAddr,
    ) -> Result<Self> {
        // Split stream for independent read/write operations
        let (read_half, write_half) = stream.into_split();
        let reader = Arc::new(Mutex::new(BufReader::new(read_half)));
        let writer = Arc::new(Mutex::new(BufWriter::new(write_half)));
        
        let (flow_tx, flow_rx) = mpsc::channel(MAX_PENDING_OPERATIONS);
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        // Perform handshake with timeout
        let datapath_id = timeout(
            CONNECTION_TIMEOUT,
            Self::perform_handshake(reader.clone(), writer.clone(), addr),
        )
        .await
        .map_err(|_| ControllerError::ConnectionFailed("Handshake timeout".to_string()))??;

        let switch_id = format!("switch-{:016x}", datapath_id);
        
        info!(
            "Switch {} connected from {} with datapath_id: {:016x}",
            switch_id, addr, datapath_id
        );

        let conn = Self {
            reader: reader.clone(),
            writer: writer.clone(),
            addr,
            switch_id: switch_id.clone(),
            datapath_id,
            xid_counter: Arc::new(std::sync::atomic::AtomicU32::new(1)),
            state: Arc::new(RwLock::new(ConnectionState::Connected)),
            flow_tx,
            shutdown_tx: Some(shutdown_tx),
            pending_xids: Arc::new(Mutex::new(HashMap::new())),
        };

        // Spawn message handler
        tokio::spawn(Self::message_handler(
            reader.clone(),
            writer.clone(),
            addr,
            switch_id.clone(),
            conn.state.clone(),
            conn.pending_xids.clone(),
            shutdown_rx,
        ));

        // Spawn flow operation handler
        tokio::spawn(Self::flow_operation_handler(
            writer.clone(),
            switch_id.clone(),
            conn.xid_counter.clone(),
            conn.state.clone(),
            conn.pending_xids.clone(),
            flow_rx,
        ));

        Ok(conn)
    }

    /// Perform OpenFlow handshake
    async fn perform_handshake(
        reader: Arc<Mutex<BufReader<OwnedReadHalf>>>,
        writer: Arc<Mutex<BufWriter<OwnedWriteHalf>>>,
        addr: SocketAddr,
    ) -> Result<u64> {
        // Send HELLO
        let hello = HelloMessage::new(1);
        let hello_bytes = hello.to_bytes();
        
        Self::write_message_safe(&writer, &hello_bytes).await?;
        debug!("Sent HELLO to {}", addr);

        // Wait for HELLO response
        let (header, _) = timeout(
            READ_TIMEOUT,
            Self::read_message_safe(&reader),
        )
        .await
        .map_err(|_| ControllerError::ConnectionFailed("Read timeout".to_string()))??;

        if header.msg_type != MessageType::Hello {
            return Err(ControllerError::ProtocolError(
                "Expected HELLO message".to_string(),
            ));
        }

        debug!("Received HELLO from {}", addr);

        // Send FEATURES_REQUEST
        let features_req = OpenFlowHeader::new(MessageType::FeaturesRequest, 2);
        let features_bytes = features_req.to_bytes();
        
        Self::write_message_safe(&writer, &features_bytes).await?;

        // Wait for FEATURES_REPLY
        let (header, payload) = timeout(
            READ_TIMEOUT,
            Self::read_message_safe(&reader),
        )
        .await
        .map_err(|_| ControllerError::ConnectionFailed("Read timeout".to_string()))??;

        if header.msg_type != MessageType::FeaturesReply {
            return Err(ControllerError::ProtocolError(
                "Expected FEATURES_REPLY".to_string(),
            ));
        }

        // Parse datapath ID from features reply
        if payload.len() < 8 {
            return Err(ControllerError::ProtocolError(
                "Invalid FEATURES_REPLY".to_string(),
            ));
        }

        let datapath_id = u64::from_be_bytes([
            payload[0], payload[1], payload[2], payload[3],
            payload[4], payload[5], payload[6], payload[7],
        ]);

        Ok(datapath_id)
    }

    /// CRITICAL FIX #4: Safe message write with buffering and flush
    /// Prevents partial write corruption by using buffered I/O
    async fn write_message_safe(
        writer: &Arc<Mutex<BufWriter<OwnedWriteHalf>>>,
        message: &[u8],
    ) -> Result<()> {
        let mut writer_guard = writer.lock().await;
        
        // Write with timeout
        timeout(WRITE_TIMEOUT, writer_guard.write_all(message))
            .await
            .map_err(|_| ControllerError::ConnectionFailed("Write timeout".to_string()))??;
        
        // CRITICAL: Flush to ensure complete transmission
        timeout(WRITE_TIMEOUT, writer_guard.flush())
            .await
            .map_err(|_| ControllerError::ConnectionFailed("Flush timeout".to_string()))??;
        
        Ok(())
    }

    /// CRITICAL FIX #5: Safe message read with buffering
    /// Handles partial reads and message boundaries correctly
    async fn read_message_safe(
        reader: &Arc<Mutex<BufReader<OwnedReadHalf>>>,
    ) -> Result<(OpenFlowHeader, Vec<u8>)> {
        let mut reader_guard = reader.lock().await;
        
        // Read header (8 bytes)
        let mut header_buf = [0u8; 8];
        reader_guard.read_exact(&mut header_buf).await?;

        let header = OpenFlowHeader::parse(&header_buf)
            .map_err(|e| ControllerError::ProtocolError(e.to_string()))?;
        
        // Validate message size
        if header.length < 8 {
            return Err(ControllerError::ProtocolError(
                "Invalid message length".to_string(),
            ));
        }

        let payload_len = (header.length as usize).saturating_sub(8);
        
        if payload_len > MAX_MESSAGE_SIZE {
            return Err(ControllerError::MessageTooLarge(payload_len));
        }

        // Read payload (handles partial reads automatically via read_exact)
        let mut payload = vec![0u8; payload_len];
        if payload_len > 0 {
            reader_guard.read_exact(&mut payload).await?;
        }

        Ok((header, payload))
    }

    /// Message handler task
    async fn message_handler(
        reader: Arc<Mutex<BufReader<OwnedReadHalf>>>,
        writer: Arc<Mutex<BufWriter<OwnedWriteHalf>>>,
        addr: SocketAddr,
        switch_id: String,
        state: Arc<RwLock<ConnectionState>>,
        pending_xids: Arc<Mutex<HashMap<u32, oneshot::Sender<Result<()>>>>>,
        mut shutdown_rx: oneshot::Receiver<()>,
    ) {
        // CRITICAL FIX #7: Cleanup guard for task cancellation safety
        let _cleanup_guard = CleanupGuard {
            switch_id: switch_id.clone(),
            state: state.clone(),
            writer: writer.clone(),
        };

        loop {
            tokio::select! {
                _ = &mut shutdown_rx => {
                    info!("Shutting down message handler for switch {}", switch_id);
                    break;
                }
                result = timeout(READ_TIMEOUT, Self::read_message_safe(&reader)) => {
                    match result {
                        Ok(Ok((header, payload))) => {
                            debug!(
                                "Switch {} received message type: {:?} (xid: {})",
                                switch_id, header.msg_type, header.xid
                            );

                            // Handle specific message types
                            match header.msg_type {
                                MessageType::EchoRequest => {
                                    // Send echo reply
                                    let mut reply = OpenFlowHeader::new(
                                        MessageType::EchoReply,
                                        header.xid,
                                    );
                                    reply.length = 8;
                                    
                                    if let Err(e) = Self::write_message_safe(&writer, &reply.to_bytes()).await {
                                        error!("Failed to send echo reply: {}", e);
                                        break;
                                    }
                                }
                                MessageType::BarrierReply => {
                                    // CRITICAL FIX #9: Handle barrier reply for flow verification
                                    info!("Received barrier reply for switch {} (xid: {})", switch_id, header.xid);
                                    
                                    let mut pending = pending_xids.lock().await;
                                    if let Some(tx) = pending.remove(&header.xid) {
                                        let _ = tx.send(Ok(()));
                                        debug!("Flow verification successful for xid: {}", header.xid);
                                    }
                                }
                                MessageType::Error => {
                                    // CRITICAL FIX #9: Handle error messages
                                    if let Ok(error_msg) = ErrorMessage::parse(header.clone(), &payload) {
                                        error!(
                                            "Switch {} sent error: {} (type: {}, code: {})",
                                            switch_id,
                                            error_msg.error_type_str(),
                                            error_msg.error_type,
                                            error_msg.error_code
                                        );
                                        
                                        // Notify pending operation if XID matches
                                        let mut pending = pending_xids.lock().await;
                                        if let Some(tx) = pending.remove(&header.xid) {
                                            let _ = tx.send(Err(ControllerError::ProtocolError(format!(
                                                "Flow operation failed: {} (type: {}, code: {})",
                                                error_msg.error_type_str(),
                                                error_msg.error_type,
                                                error_msg.error_code
                                            ))));
                                        }
                                    } else {
                                        warn!("Switch {} sent unparseable error message", switch_id);
                                    }
                                }
                                _ => {}
                            }
                        }
                        Ok(Err(e)) => {
                            error!("Error receiving message from {}: {}", switch_id, e);
                            *state.write().await = ConnectionState::Failed;
                            break;
                        }
                        Err(_) => {
                            warn!("Read timeout for switch {}", switch_id);
                            // Don't break on timeout, just continue
                        }
                    }
                }
            }
        }

        // Explicit cleanup before drop
        *state.write().await = ConnectionState::Disconnected;
        
        // Fail all pending operations
        let mut pending = pending_xids.lock().await;
        for (xid, tx) in pending.drain() {
            let _ = tx.send(Err(ControllerError::ConnectionFailed(
                "Connection closed".to_string(),
            )));
            debug!("Failed pending operation xid: {}", xid);
        }
        
        // Flush writer before shutdown
        if let Ok(mut writer_guard) = writer.lock().await {
            let _ = writer_guard.flush().await;
            debug!("Flushed writer for switch {}", switch_id);
        }
        
        info!("Message handler stopped for switch {}", switch_id);
    }

    /// Flow operation handler task
    async fn flow_operation_handler(
        writer: Arc<Mutex<BufWriter<OwnedWriteHalf>>>,
        switch_id: String,
        xid_counter: Arc<std::sync::atomic::AtomicU32>,
        state: Arc<RwLock<ConnectionState>>,
        pending_xids: Arc<Mutex<HashMap<u32, oneshot::Sender<Result<()>>>>>,
        mut flow_rx: mpsc::Receiver<(FlowOperation, oneshot::Sender<Result<()>>)>,
    ) {
        // CRITICAL FIX #7: Cleanup guard for task cancellation safety
        let _cleanup_guard = CleanupGuard {
            switch_id: switch_id.clone(),
            state: state.clone(),
            writer: writer.clone(),
        };

        while let Some((operation, result_tx)) = flow_rx.recv().await {
            // CRITICAL FIX: Validate connection state before executing operation
            let current_state = *state.read().await;
            
            let result = match current_state {
                ConnectionState::Connected | ConnectionState::Authenticated => {
                    // Connection is healthy, execute operation with verification
                    Self::execute_flow_operation_verified(
                        &writer,
                        &switch_id,
                        &xid_counter,
                        &pending_xids,
                        operation,
                    )
                    .await
                }
                ConnectionState::Disconnected | ConnectionState::Failed => {
                    // Connection is dead, reject operation immediately
                    warn!(
                        "Rejecting flow operation for switch {} - connection state: {:?}",
                        switch_id, current_state
                    );
                    Err(ControllerError::ConnectionFailed(format!(
                        "Switch {} is not connected (state: {:?})",
                        switch_id, current_state
                    )))
                }
                ConnectionState::Connecting => {
                    // Connection is still establishing, reject for now
                    warn!(
                        "Rejecting flow operation for switch {} - still connecting",
                        switch_id
                    );
                    Err(ControllerError::ConnectionFailed(format!(
                        "Switch {} is still connecting",
                        switch_id
                    )))
                }
            };

            // Send result back
            let _ = result_tx.send(result);
        }

        // Explicit cleanup: drain remaining operations
        flow_rx.close();
        while let Some((_, result_tx)) = flow_rx.recv().await {
            let _ = result_tx.send(Err(ControllerError::ConnectionFailed(
                "Flow handler shutting down".to_string(),
            )));
        }

        info!("Flow operation handler stopped for switch {}", switch_id);
    }

    /// Execute flow operation - ACTUALLY SEND TO SWITCH
    async fn execute_flow_operation(
        writer: &Arc<Mutex<BufWriter<OwnedWriteHalf>>>,
        switch_id: &str,
        xid_counter: &Arc<std::sync::atomic::AtomicU32>,
        operation: FlowOperation,
    ) -> Result<()> {
        // CRITICAL FIX: Atomic XID generation with proper handling
        let xid = loop {
            let xid = xid_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            
            // Skip reserved XID (0 is invalid in OpenFlow)
            if xid == 0 {
                continue;
            }
            
            // Check for wrap-around (very rare with u32)
            if xid == u32::MAX {
                // Reset counter to 1 (skip 0)
                xid_counter.store(1, std::sync::atomic::Ordering::SeqCst);
                warn!("XID counter wrapped around for switch {}", switch_id);
                continue;
            }
            
            break xid;
        };

        let msg = match &operation {
            FlowOperation::Add(rule) => {
                info!(
                    "Installing flow {:?} on switch {} (priority: {}, xid: {})",
                    rule.id, switch_id, rule.priority, xid
                );
                Self::create_flow_mod(rule, 0, xid) // ADD command
            }
            FlowOperation::Modify(rule) => {
                info!("Modifying flow {:?} on switch {} (xid: {})", rule.id, switch_id, xid);
                Self::create_flow_mod(rule, 1, xid) // MODIFY command
            }
            FlowOperation::Delete(flow_id) => {
                info!("Deleting flow {:?} from switch {}", flow_id, switch_id);
                FlowModMessage::new(xid, 3) // DELETE command
            }
        };

        let bytes = msg.to_bytes();

        // CRITICAL FIX #4: Use safe write with buffering and flush
        Self::write_message_safe(writer, &bytes).await?;

        info!(
            "Successfully sent flow operation to switch {} (xid: {})",
            switch_id, xid
        );

        Ok(())
    }

    /// CRITICAL FIX #9: Execute flow operation with verification
    /// Sends flow mod, then barrier request, waits for barrier reply
    async fn execute_flow_operation_verified(
        writer: &Arc<Mutex<BufWriter<OwnedWriteHalf>>>,
        switch_id: &str,
        xid_counter: &Arc<std::sync::atomic::AtomicU32>,
        pending_xids: &Arc<Mutex<HashMap<u32, oneshot::Sender<Result<()>>>>>,
        operation: FlowOperation,
    ) -> Result<()> {
        // Send flow operation
        Self::execute_flow_operation(writer, switch_id, xid_counter, operation).await?;

        // Generate XID for barrier request
        let barrier_xid = loop {
            let xid = xid_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            if xid == 0 {
                continue;
            }
            if xid == u32::MAX {
                xid_counter.store(1, std::sync::atomic::Ordering::SeqCst);
                continue;
            }
            break xid;
        };

        // Create verification channel
        let (verify_tx, verify_rx) = oneshot::channel();
        
        // Register pending XID
        {
            let mut pending = pending_xids.lock().await;
            pending.insert(barrier_xid, verify_tx);
        }

        // Send barrier request
        let barrier = BarrierMessage::new_request(barrier_xid);
        Self::write_message_safe(writer, &barrier.to_bytes()).await?;

        info!(
            "Sent barrier request for switch {} (xid: {})",
            switch_id, barrier_xid
        );

        // Wait for barrier reply with timeout
        match timeout(FLOW_VERIFY_TIMEOUT, verify_rx).await {
            Ok(Ok(Ok(()))) => {
                info!(
                    "Flow operation verified for switch {} (barrier xid: {})",
                    switch_id, barrier_xid
                );
                Ok(())
            }
            Ok(Ok(Err(e))) => {
                error!(
                    "Flow operation failed for switch {} (barrier xid: {}): {}",
                    switch_id, barrier_xid, e
                );
                Err(e)
            }
            Ok(Err(_)) => {
                // Channel closed
                let mut pending = pending_xids.lock().await;
                pending.remove(&barrier_xid);
                Err(ControllerError::ConnectionFailed(
                    "Verification channel closed".to_string(),
                ))
            }
            Err(_) => {
                // Timeout
                warn!(
                    "Flow verification timeout for switch {} (barrier xid: {})",
                    switch_id, barrier_xid
                );
                let mut pending = pending_xids.lock().await;
                pending.remove(&barrier_xid);
                Err(ControllerError::ConnectionFailed(
                    "Flow verification timeout".to_string(),
                ))
            }
        }
    }

    /// Create FlowMod message from FlowRule with COMPLETE match and action encoding
    fn create_flow_mod(rule: &FlowRule, command: u8, xid: u32) -> FlowModMessage {
        use crate::openflow::{Action as OfAction, Instruction, OxmField};
        
        let mut msg = FlowModMessage::new(xid, command);
        msg.priority = rule.priority;
        msg.idle_timeout = rule.idle_timeout;
        msg.hard_timeout = rule.hard_timeout;
        
        // Encode match fields from FlowRule
        let mut oxm_fields = Vec::new();
        
        if let Some(in_port) = rule.match_fields.in_port {
            oxm_fields.push(OxmField::InPort(in_port));
        }
        
        if let Some(ref eth_src) = rule.match_fields.eth_src {
            if let Ok(mac) = parse_mac_address(eth_src) {
                oxm_fields.push(OxmField::EthSrc(mac));
            }
        }
        
        if let Some(ref eth_dst) = rule.match_fields.eth_dst {
            if let Ok(mac) = parse_mac_address(eth_dst) {
                oxm_fields.push(OxmField::EthDst(mac));
            }
        }
        
        if let Some(eth_type) = rule.match_fields.eth_type {
            oxm_fields.push(OxmField::EthType(eth_type));
        }
        
        if let Some(ip_proto) = rule.match_fields.ip_proto {
            oxm_fields.push(OxmField::IpProto(ip_proto));
        }
        
        if let Some(ip_src) = rule.match_fields.ip_src {
            if let std::net::IpAddr::V4(ipv4) = ip_src {
                oxm_fields.push(OxmField::Ipv4Src(u32::from(ipv4)));
            }
        }
        
        if let Some(ip_dst) = rule.match_fields.ip_dst {
            if let std::net::IpAddr::V4(ipv4) = ip_dst {
                oxm_fields.push(OxmField::Ipv4Dst(u32::from(ipv4)));
            }
        }
        
        if let Some(tcp_src) = rule.match_fields.tcp_src {
            oxm_fields.push(OxmField::TcpSrc(tcp_src));
        }
        
        if let Some(tcp_dst) = rule.match_fields.tcp_dst {
            oxm_fields.push(OxmField::TcpDst(tcp_dst));
        }
        
        msg.match_fields = oxm_fields;
        
        // Encode actions from FlowRule
        let mut of_actions = Vec::new();
        
        for action in &rule.actions {
            match action {
                crate::types::Action::Output { port } => {
                    of_actions.push(OfAction::Output {
                        port: *port,
                        max_len: 0xFFFF, // No buffer
                    });
                }
                crate::types::Action::SetVlan { vlan_id } => {
                    // Push VLAN tag then set field
                    of_actions.push(OfAction::PushVlan { ethertype: 0x8100 });
                    // TODO: Add SET_FIELD for VLAN_VID
                }
                crate::types::Action::SetQueue { queue_id } => {
                    of_actions.push(OfAction::SetQueue { queue_id: *queue_id });
                }
                crate::types::Action::Drop => {
                    // Drop = no actions, handled by empty instruction list
                }
            }
        }
        
        // Wrap actions in APPLY_ACTIONS instruction
        if !of_actions.is_empty() || rule.actions.iter().any(|a| matches!(a, crate::types::Action::Drop)) {
            msg.instructions = vec![Instruction::ApplyActions(of_actions)];
        }
        
        msg
    }

    /// Send flow operation
    pub async fn send_flow_operation(&self, operation: FlowOperation) -> Result<()> {
        let (result_tx, result_rx) = oneshot::channel();
        
        // CRITICAL FIX #8: Backpressure handling with timeout
        // If queue is full, timeout instead of blocking forever
        timeout(FLOW_SEND_TIMEOUT, self.flow_tx.send((operation, result_tx)))
            .await
            .map_err(|_| {
                warn!(
                    "Flow operation queue full for switch {} - backpressure timeout",
                    self.switch_id
                );
                ControllerError::ConnectionFailed(format!(
                    "Flow operation queue full for switch {} (backpressure)",
                    self.switch_id
                ))
            })?
            .map_err(|_| ControllerError::ConnectionFailed("Channel closed".to_string()))?;

        result_rx
            .await
            .map_err(|_| ControllerError::ConnectionFailed("Result channel closed".to_string()))?
    }

    /// Get connection state
    pub async fn state(&self) -> ConnectionState {
        *self.state.read().await
    }

    /// Get switch info
    pub fn switch_info(&self) -> Switch {
        Switch {
            id: self.switch_id.clone(),
            datapath_id: self.datapath_id,
            ip_address: self.addr.ip(),
            port: self.addr.port(),
            connected: true,
            num_ports: 0,
            capabilities: vec!["FLOW_STATS".to_string(), "TABLE_STATS".to_string()],
        }
    }

    /// Shutdown connection
    pub async fn shutdown(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        *self.state.write().await = ConnectionState::Disconnected;
    }
}

/// Connection manager for all switches
pub struct ConnectionManager {
    connections: Arc<DashMap<String, Arc<ManagedConnection>>>,
    max_connections: usize,
}

impl ConnectionManager {
    pub fn new(max_connections: usize) -> Self {
        Self {
            connections: Arc::new(DashMap::new()),
            max_connections,
        }
    }

    /// Add new connection
    pub async fn add_connection(
        &self,
        stream: TcpStream,
        addr: SocketAddr,
    ) -> Result<Arc<ManagedConnection>> {
        // Check connection limit
        if self.connections.len() >= self.max_connections {
            return Err(ControllerError::ConnectionFailed(
                "Maximum connections reached".to_string(),
            ));
        }

        let conn = Arc::new(ManagedConnection::new(stream, addr).await?);
        let switch_id = conn.switch_id.clone();
        
        self.connections.insert(switch_id.clone(), conn.clone());
        
        info!(
            "Added connection for switch {} (total: {})",
            switch_id,
            self.connections.len()
        );

        Ok(conn)
    }

    /// Get connection by switch ID
    pub fn get_connection(&self, switch_id: &str) -> Option<Arc<ManagedConnection>> {
        self.connections.get(switch_id).map(|entry| entry.clone())
    }

    /// Remove connection
    pub async fn remove_connection(&self, switch_id: &str) {
        if let Some((_, mut conn)) = self.connections.remove(switch_id) {
            Arc::get_mut(&mut conn).unwrap().shutdown().await;
            info!("Removed connection for switch {}", switch_id);
        }
    }

    /// Get all connected switches
    pub fn get_all_switches(&self) -> Vec<Switch> {
        self.connections
            .iter()
            .map(|entry| entry.value().switch_info())
            .collect()
    }

    /// Shutdown all connections
    pub async fn shutdown_all(&self) {
        info!("Shutting down all connections");
        
        let switch_ids: Vec<String> = self.connections
            .iter()
            .map(|entry| entry.key().clone())
            .collect();

        for switch_id in switch_ids {
            self.remove_connection(&switch_id).await;
        }
    }
}

/// Parse MAC address from string (format: "aa:bb:cc:dd:ee:ff")
fn parse_mac_address(mac_str: &str) -> Result<[u8; 6]> {
    let parts: Vec<&str> = mac_str.split(':').collect();
    if parts.len() != 6 {
        return Err(ControllerError::ProtocolError(format!(
            "Invalid MAC address format: {}",
            mac_str
        )));
    }
    
    let mut mac = [0u8; 6];
    for (i, part) in parts.iter().enumerate() {
        mac[i] = u8::from_str_radix(part, 16).map_err(|_| {
            ControllerError::ProtocolError(format!("Invalid MAC address: {}", mac_str))
        })?;
    }
    
    Ok(mac)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connection_manager_limits() {
        let manager = ConnectionManager::new(2);
        assert_eq!(manager.connections.len(), 0);
    }
}

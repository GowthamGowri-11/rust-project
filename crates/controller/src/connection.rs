use crate::openflow::{
    FeaturesReplyMessage, HelloMessage, MessageType, OpenFlowError, OpenFlowHeader, OpenFlowMessage,
    OpenFlowResult,
};
use crate::types::{Switch, SwitchId};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{debug, error, info, warn};

pub struct SwitchConnection {
    stream: TcpStream,
    addr: SocketAddr,
    switch_id: Option<SwitchId>,
    datapath_id: Option<u64>,
    xid_counter: u32,
}

impl SwitchConnection {
    pub fn new(stream: TcpStream, addr: SocketAddr) -> Self {
        Self {
            stream,
            addr,
            switch_id: None,
            datapath_id: None,
            xid_counter: 1,
        }
    }

    pub async fn handle(&mut self) -> OpenFlowResult<()> {
        info!("New switch connection from {}", self.addr);

        // Send HELLO message
        self.send_hello().await?;

        // Main message loop
        loop {
            match self.receive_message().await {
                Ok((header, payload)) => {
                    debug!(
                        "Received message type: {:?} from {}",
                        header.msg_type, self.addr
                    );

                    match header.msg_type {
                        MessageType::Hello => {
                            self.handle_hello(&header).await?;
                        }
                        MessageType::FeaturesRequest => {
                            self.handle_features_request(&header).await?;
                        }
                        MessageType::EchoRequest => {
                            self.handle_echo_request(&header).await?;
                        }
                        MessageType::PacketIn => {
                            debug!("Received PacketIn from switch");
                        }
                        MessageType::FlowRemoved => {
                            debug!("Received FlowRemoved from switch");
                        }
                        MessageType::PortStatus => {
                            debug!("Received PortStatus from switch");
                        }
                        _ => {
                            warn!("Unhandled message type: {:?}", header.msg_type);
                        }
                    }
                }
                Err(e) => {
                    error!("Error receiving message: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    async fn send_hello(&mut self) -> OpenFlowResult<()> {
        let msg = HelloMessage::new(self.next_xid());
        let bytes = msg.to_bytes();
        self.stream.write_all(&bytes).await?;
        debug!("Sent HELLO message to {}", self.addr);
        Ok(())
    }

    async fn handle_hello(&mut self, header: &OpenFlowHeader) -> OpenFlowResult<()> {
        debug!("Received HELLO from switch");
        Ok(())
    }

    async fn handle_features_request(&mut self, header: &OpenFlowHeader) -> OpenFlowResult<()> {
        // Generate a datapath ID based on switch address
        let datapath_id = self.generate_datapath_id();
        self.datapath_id = Some(datapath_id);

        let msg = FeaturesReplyMessage::new(header.xid, datapath_id);
        let bytes = msg.to_bytes();
        self.stream.write_all(&bytes).await?;

        info!(
            "Sent FeaturesReply to switch {} with datapath_id: {}",
            self.addr, datapath_id
        );

        Ok(())
    }

    async fn handle_echo_request(&mut self, header: &OpenFlowHeader) -> OpenFlowResult<()> {
        let mut reply_header = OpenFlowHeader::new(MessageType::EchoReply, header.xid);
        reply_header.length = OpenFlowHeader::HEADER_SIZE as u16;

        let bytes = reply_header.to_bytes();
        self.stream.write_all(&bytes).await?;
        debug!("Sent EchoReply to {}", self.addr);
        Ok(())
    }

    async fn receive_message(&mut self) -> OpenFlowResult<(OpenFlowHeader, Vec<u8>)> {
        let mut header_buf = [0u8; 8];
        self.stream.read_exact(&mut header_buf).await?;

        let header = OpenFlowHeader::parse(&header_buf)?;
        let payload_len = (header.length as usize).saturating_sub(8);

        let mut payload = vec![0u8; payload_len];
        if payload_len > 0 {
            self.stream.read_exact(&mut payload).await?;
        }

        Ok((header, payload))
    }

    pub async fn send_message(&mut self, msg: OpenFlowMessage) -> OpenFlowResult<()> {
        let bytes = msg.to_bytes();
        self.stream.write_all(&bytes).await?;
        Ok(())
    }

    fn next_xid(&mut self) -> u32 {
        let xid = self.xid_counter;
        self.xid_counter = self.xid_counter.wrapping_add(1);
        xid
    }

    fn generate_datapath_id(&self) -> u64 {
        // Generate datapath ID from IP address
        let ip_bytes = match self.addr.ip() {
            std::net::IpAddr::V4(ip) => ip.octets().to_vec(),
            std::net::IpAddr::V6(_) => vec![0, 0, 0, 0],
        };

        let mut id = 0u64;
        for (i, byte) in ip_bytes.iter().enumerate().take(4) {
            id |= (*byte as u64) << (8 * (3 - i));
        }
        id |= (self.addr.port() as u64) << 32;
        id
    }

    pub fn get_switch_info(&self) -> Option<Switch> {
        self.datapath_id.map(|datapath_id| Switch {
            id: format!("switch-{}", datapath_id),
            datapath_id,
            ip_address: self.addr.ip(),
            port: self.addr.port(),
            connected: true,
            num_ports: 0,
            capabilities: vec!["FLOW_STATS".to_string(), "TABLE_STATS".to_string()],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_datapath_id_generation() {
        let stream = TcpStream::connect("127.0.0.1:0")
            .await
            .unwrap_or_else(|_| panic!("Failed to connect"));
        let addr = "127.0.0.1:6633".parse().unwrap();
        let conn = SwitchConnection::new(stream, addr);
        let datapath_id = conn.generate_datapath_id();
        assert!(datapath_id > 0);
    }
}

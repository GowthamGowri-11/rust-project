use bytes::{Bytes, BytesMut};
use std::io::Cursor;
use thiserror::Error;
use tracing::{debug, error};

#[derive(Error, Debug)]
pub enum OpenFlowError {
    #[error("Invalid message type: {0}")]
    InvalidMessageType(u8),
    #[error("Invalid message length: {0}")]
    InvalidMessageLength(usize),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type OpenFlowResult<T> = Result<T, OpenFlowError>;

// OpenFlow 1.3 Message Types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    Hello = 0,
    Error = 1,
    EchoRequest = 2,
    EchoReply = 3,
    Experimenter = 4,
    FeaturesRequest = 5,
    FeaturesReply = 6,
    GetConfigRequest = 7,
    GetConfigReply = 8,
    SetConfig = 9,
    PacketIn = 10,
    FlowRemoved = 11,
    PortStatus = 12,
    PacketOut = 13,
    FlowMod = 14,
    GroupMod = 15,
    PortMod = 16,
    TableMod = 17,
    MultipartRequest = 18,
    MultipartReply = 19,
    BarrierRequest = 20,
    BarrierReply = 21,
    QueueGetConfigRequest = 22,
    QueueGetConfigReply = 23,
    RoleRequest = 24,
    RoleReply = 25,
    GetAsyncRequest = 26,
    GetAsyncReply = 27,
    SetAsync = 28,
    MeterMod = 29,
}

impl MessageType {
    pub fn from_u8(value: u8) -> OpenFlowResult<Self> {
        match value {
            0 => Ok(MessageType::Hello),
            1 => Ok(MessageType::Error),
            2 => Ok(MessageType::EchoRequest),
            3 => Ok(MessageType::EchoReply),
            4 => Ok(MessageType::Experimenter),
            5 => Ok(MessageType::FeaturesRequest),
            6 => Ok(MessageType::FeaturesReply),
            7 => Ok(MessageType::GetConfigRequest),
            8 => Ok(MessageType::GetConfigReply),
            9 => Ok(MessageType::SetConfig),
            10 => Ok(MessageType::PacketIn),
            11 => Ok(MessageType::FlowRemoved),
            12 => Ok(MessageType::PortStatus),
            13 => Ok(MessageType::PacketOut),
            14 => Ok(MessageType::FlowMod),
            15 => Ok(MessageType::GroupMod),
            16 => Ok(MessageType::PortMod),
            17 => Ok(MessageType::TableMod),
            18 => Ok(MessageType::MultipartRequest),
            19 => Ok(MessageType::MultipartReply),
            20 => Ok(MessageType::BarrierRequest),
            21 => Ok(MessageType::BarrierReply),
            22 => Ok(MessageType::QueueGetConfigRequest),
            23 => Ok(MessageType::QueueGetConfigReply),
            24 => Ok(MessageType::RoleRequest),
            25 => Ok(MessageType::RoleReply),
            26 => Ok(MessageType::GetAsyncRequest),
            27 => Ok(MessageType::GetAsyncReply),
            28 => Ok(MessageType::SetAsync),
            29 => Ok(MessageType::MeterMod),
            _ => Err(OpenFlowError::InvalidMessageType(value)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct OpenFlowHeader {
    pub version: u8,
    pub msg_type: MessageType,
    pub length: u16,
    pub xid: u32,
}

impl OpenFlowHeader {
    pub const VERSION: u8 = 4; // OpenFlow 1.3
    pub const HEADER_SIZE: usize = 8;

    pub fn new(msg_type: MessageType, xid: u32) -> Self {
        Self {
            version: Self::VERSION,
            msg_type,
            length: Self::HEADER_SIZE as u16,
            xid,
        }
    }

    pub fn parse(data: &[u8]) -> OpenFlowResult<Self> {
        if data.len() < Self::HEADER_SIZE {
            return Err(OpenFlowError::InvalidMessageLength(data.len()));
        }

        let version = data[0];
        let msg_type = MessageType::from_u8(data[1])?;
        let length = u16::from_be_bytes([data[2], data[3]]);
        let xid = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);

        if version != Self::VERSION {
            return Err(OpenFlowError::ParseError(format!(
                "Unsupported OpenFlow version: {}",
                version
            )));
        }

        Ok(Self {
            version,
            msg_type,
            length,
            xid,
        })
    }

    pub fn to_bytes(&self) -> Bytes {
        let mut buf = BytesMut::with_capacity(Self::HEADER_SIZE);
        buf.extend_from_slice(&[self.version, self.msg_type as u8]);
        buf.extend_from_slice(&self.length.to_be_bytes());
        buf.extend_from_slice(&self.xid.to_be_bytes());
        buf.freeze()
    }
}

#[derive(Debug, Clone)]
pub struct HelloMessage {
    pub header: OpenFlowHeader,
    pub elements: Vec<u8>,
}

impl HelloMessage {
    pub fn new(xid: u32) -> Self {
        let mut header = OpenFlowHeader::new(MessageType::Hello, xid);
        header.length = OpenFlowHeader::HEADER_SIZE as u16;

        Self {
            header,
            elements: Vec::new(),
        }
    }

    pub fn to_bytes(&self) -> Bytes {
        self.header.to_bytes()
    }
}

#[derive(Debug, Clone)]
pub struct FeaturesReplyMessage {
    pub header: OpenFlowHeader,
    pub datapath_id: u64,
    pub num_buffers: u32,
    pub num_tables: u8,
    pub auxiliary_id: u8,
    pub capabilities: u32,
}

impl FeaturesReplyMessage {
    pub fn new(xid: u32, datapath_id: u64) -> Self {
        let mut header = OpenFlowHeader::new(MessageType::FeaturesReply, xid);
        header.length = (OpenFlowHeader::HEADER_SIZE + 24) as u16;

        Self {
            header,
            datapath_id,
            num_buffers: 256,
            num_tables: 254,
            auxiliary_id: 0,
            capabilities: 0x0000_00FF, // All capabilities
        }
    }

    pub fn to_bytes(&self) -> Bytes {
        let mut buf = BytesMut::with_capacity(self.header.length as usize);
        buf.extend_from_slice(&self.header.to_bytes());
        buf.extend_from_slice(&self.datapath_id.to_be_bytes());
        buf.extend_from_slice(&self.num_buffers.to_be_bytes());
        buf.extend_from_slice(&[self.num_tables, self.auxiliary_id]);
        buf.extend_from_slice(&[0, 0]); // Padding
        buf.extend_from_slice(&self.capabilities.to_be_bytes());
        buf.extend_from_slice(&[0, 0, 0, 0]); // Reserved
        buf.freeze()
    }
}

#[derive(Debug, Clone)]
pub struct FlowModMessage {
    pub header: OpenFlowHeader,
    pub cookie: u64,
    pub cookie_mask: u64,
    pub table_id: u8,
    pub command: u8, // 0=ADD, 1=MODIFY, 2=MODIFY_STRICT, 3=DELETE, 4=DELETE_STRICT
    pub idle_timeout: u16,
    pub hard_timeout: u16,
    pub priority: u16,
    pub buffer_id: u32,
    pub out_port: u32,
    pub out_group: u32,
    pub flags: u16,
}

impl FlowModMessage {
    pub fn new(xid: u32, command: u8) -> Self {
        let mut header = OpenFlowHeader::new(MessageType::FlowMod, xid);
        header.length = (OpenFlowHeader::HEADER_SIZE + 56) as u16;

        Self {
            header,
            cookie: 0,
            cookie_mask: 0,
            table_id: 0,
            command,
            idle_timeout: 0,
            hard_timeout: 0,
            priority: 32768,
            buffer_id: 0xFFFF_FFFF,
            out_port: 0xFFFF_FFFF,
            out_group: 0xFFFF_FFFF,
            flags: 0,
        }
    }

    pub fn to_bytes(&self) -> Bytes {
        let mut buf = BytesMut::with_capacity(self.header.length as usize);
        buf.extend_from_slice(&self.header.to_bytes());
        buf.extend_from_slice(&self.cookie.to_be_bytes());
        buf.extend_from_slice(&self.cookie_mask.to_be_bytes());
        buf.extend_from_slice(&[self.table_id, self.command]);
        buf.extend_from_slice(&self.idle_timeout.to_be_bytes());
        buf.extend_from_slice(&self.hard_timeout.to_be_bytes());
        buf.extend_from_slice(&self.priority.to_be_bytes());
        buf.extend_from_slice(&self.buffer_id.to_be_bytes());
        buf.extend_from_slice(&self.out_port.to_be_bytes());
        buf.extend_from_slice(&self.out_group.to_be_bytes());
        buf.extend_from_slice(&self.flags.to_be_bytes());
        buf.extend_from_slice(&[0, 0]); // Padding
        buf.freeze()
    }
}

#[derive(Debug, Clone)]
pub enum OpenFlowMessage {
    Hello(HelloMessage),
    FeaturesReply(FeaturesReplyMessage),
    FlowMod(FlowModMessage),
    EchoReply(OpenFlowHeader),
}

impl OpenFlowMessage {
    pub fn to_bytes(&self) -> Bytes {
        match self {
            OpenFlowMessage::Hello(msg) => msg.to_bytes(),
            OpenFlowMessage::FeaturesReply(msg) => msg.to_bytes(),
            OpenFlowMessage::FlowMod(msg) => msg.to_bytes(),
            OpenFlowMessage::EchoReply(header) => header.to_bytes(),
        }
    }

    pub fn parse(data: &[u8]) -> OpenFlowResult<(OpenFlowHeader, Vec<u8>)> {
        let header = OpenFlowHeader::parse(data)?;
        let payload = if data.len() > OpenFlowHeader::HEADER_SIZE {
            data[OpenFlowHeader::HEADER_SIZE..].to_vec()
        } else {
            Vec::new()
        };
        Ok((header, payload))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_serialization() {
        let header = OpenFlowHeader::new(MessageType::Hello, 1);
        let bytes = header.to_bytes();
        assert_eq!(bytes.len(), OpenFlowHeader::HEADER_SIZE);
        assert_eq!(bytes[0], OpenFlowHeader::VERSION);
        assert_eq!(bytes[1], MessageType::Hello as u8);
    }

    #[test]
    fn test_hello_message() {
        let msg = HelloMessage::new(1);
        let bytes = msg.to_bytes();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_features_reply() {
        let msg = FeaturesReplyMessage::new(1, 0x0000_0000_0000_0001);
        let bytes = msg.to_bytes();
        assert_eq!(bytes.len(), msg.header.length as usize);
    }

    #[test]
    fn test_flow_mod_message() {
        let msg = FlowModMessage::new(1, 0); // ADD command
        let bytes = msg.to_bytes();
        assert_eq!(bytes.len(), msg.header.length as usize);
    }
}

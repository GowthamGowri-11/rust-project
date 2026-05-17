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
    pub match_fields: Vec<OxmField>,
    pub instructions: Vec<Instruction>,
}

/// OpenFlow Extensible Match (OXM) field
#[derive(Debug, Clone)]
pub enum OxmField {
    InPort(u32),
    EthSrc([u8; 6]),
    EthDst([u8; 6]),
    EthType(u16),
    IpProto(u8),
    Ipv4Src(u32),
    Ipv4Dst(u32),
    TcpSrc(u16),
    TcpDst(u16),
    UdpSrc(u16),
    UdpDst(u16),
}

impl OxmField {
    /// Encode OXM field to bytes (OpenFlow 1.3 format)
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            OxmField::InPort(port) => {
                // OXM_OF_IN_PORT: class=0x8000, field=0, length=4
                let mut buf = vec![0x80, 0x00, 0x00, 0x04];
                buf.extend_from_slice(&port.to_be_bytes());
                buf
            }
            OxmField::EthSrc(mac) => {
                // OXM_OF_ETH_SRC: class=0x8000, field=3, length=6
                let mut buf = vec![0x80, 0x00, 0x06, 0x06];
                buf.extend_from_slice(mac);
                buf
            }
            OxmField::EthDst(mac) => {
                // OXM_OF_ETH_DST: class=0x8000, field=2, length=6
                let mut buf = vec![0x80, 0x00, 0x04, 0x06];
                buf.extend_from_slice(mac);
                buf
            }
            OxmField::EthType(eth_type) => {
                // OXM_OF_ETH_TYPE: class=0x8000, field=4, length=2
                let mut buf = vec![0x80, 0x00, 0x08, 0x02];
                buf.extend_from_slice(&eth_type.to_be_bytes());
                buf
            }
            OxmField::IpProto(proto) => {
                // OXM_OF_IP_PROTO: class=0x8000, field=10, length=1
                let mut buf = vec![0x80, 0x00, 0x14, 0x01];
                buf.push(*proto);
                buf
            }
            OxmField::Ipv4Src(ip) => {
                // OXM_OF_IPV4_SRC: class=0x8000, field=11, length=4
                let mut buf = vec![0x80, 0x00, 0x16, 0x04];
                buf.extend_from_slice(&ip.to_be_bytes());
                buf
            }
            OxmField::Ipv4Dst(ip) => {
                // OXM_OF_IPV4_DST: class=0x8000, field=12, length=4
                let mut buf = vec![0x80, 0x00, 0x18, 0x04];
                buf.extend_from_slice(&ip.to_be_bytes());
                buf
            }
            OxmField::TcpSrc(port) => {
                // OXM_OF_TCP_SRC: class=0x8000, field=13, length=2
                let mut buf = vec![0x80, 0x00, 0x1a, 0x02];
                buf.extend_from_slice(&port.to_be_bytes());
                buf
            }
            OxmField::TcpDst(port) => {
                // OXM_OF_TCP_DST: class=0x8000, field=14, length=2
                let mut buf = vec![0x80, 0x00, 0x1c, 0x02];
                buf.extend_from_slice(&port.to_be_bytes());
                buf
            }
            OxmField::UdpSrc(port) => {
                // OXM_OF_UDP_SRC: class=0x8000, field=15, length=2
                let mut buf = vec![0x80, 0x00, 0x1e, 0x02];
                buf.extend_from_slice(&port.to_be_bytes());
                buf
            }
            OxmField::UdpDst(port) => {
                // OXM_OF_UDP_DST: class=0x8000, field=16, length=2
                let mut buf = vec![0x80, 0x00, 0x20, 0x02];
                buf.extend_from_slice(&port.to_be_bytes());
                buf
            }
        }
    }
}

/// OpenFlow Instruction
#[derive(Debug, Clone)]
pub enum Instruction {
    ApplyActions(Vec<Action>),
    GotoTable(u8),
    WriteActions(Vec<Action>),
    ClearActions,
}

impl Instruction {
    /// Encode instruction to bytes (OpenFlow 1.3 format)
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Instruction::ApplyActions(actions) => {
                // OFPIT_APPLY_ACTIONS = 4
                let mut action_bytes = Vec::new();
                for action in actions {
                    action_bytes.extend_from_slice(&action.to_bytes());
                }
                
                let length = 8 + action_bytes.len();
                let mut buf = Vec::new();
                buf.extend_from_slice(&[0x00, 0x04]); // type
                buf.extend_from_slice(&(length as u16).to_be_bytes()); // length
                buf.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // padding
                buf.extend_from_slice(&action_bytes);
                buf
            }
            Instruction::GotoTable(table_id) => {
                // OFPIT_GOTO_TABLE = 1
                let mut buf = Vec::new();
                buf.extend_from_slice(&[0x00, 0x01]); // type
                buf.extend_from_slice(&[0x00, 0x08]); // length = 8
                buf.push(*table_id);
                buf.extend_from_slice(&[0x00, 0x00, 0x00]); // padding
                buf
            }
            Instruction::WriteActions(actions) => {
                // OFPIT_WRITE_ACTIONS = 3
                let mut action_bytes = Vec::new();
                for action in actions {
                    action_bytes.extend_from_slice(&action.to_bytes());
                }
                
                let length = 8 + action_bytes.len();
                let mut buf = Vec::new();
                buf.extend_from_slice(&[0x00, 0x03]); // type
                buf.extend_from_slice(&(length as u16).to_be_bytes()); // length
                buf.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // padding
                buf.extend_from_slice(&action_bytes);
                buf
            }
            Instruction::ClearActions => {
                // OFPIT_CLEAR_ACTIONS = 5
                let mut buf = Vec::new();
                buf.extend_from_slice(&[0x00, 0x05]); // type
                buf.extend_from_slice(&[0x00, 0x08]); // length = 8
                buf.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // padding
                buf
            }
        }
    }
}

/// OpenFlow Action
#[derive(Debug, Clone)]
pub enum Action {
    Output { port: u32, max_len: u16 },
    SetField(OxmField),
    PushVlan { ethertype: u16 },
    PopVlan,
    SetQueue { queue_id: u32 },
    Drop,
}

impl Action {
    /// Encode action to bytes (OpenFlow 1.3 format)
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Action::Output { port, max_len } => {
                // OFPAT_OUTPUT = 0
                let mut buf = Vec::new();
                buf.extend_from_slice(&[0x00, 0x00]); // type
                buf.extend_from_slice(&[0x00, 0x10]); // length = 16
                buf.extend_from_slice(&port.to_be_bytes());
                buf.extend_from_slice(&max_len.to_be_bytes());
                buf.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00]); // padding
                buf
            }
            Action::SetField(field) => {
                // OFPAT_SET_FIELD = 25
                let field_bytes = field.to_bytes();
                let length = 4 + field_bytes.len();
                // Pad to 8-byte boundary
                let padded_length = ((length + 7) / 8) * 8;
                let padding = padded_length - length;
                
                let mut buf = Vec::new();
                buf.extend_from_slice(&[0x00, 0x19]); // type
                buf.extend_from_slice(&(padded_length as u16).to_be_bytes()); // length
                buf.extend_from_slice(&field_bytes);
                buf.extend_from_slice(&vec![0x00; padding]); // padding
                buf
            }
            Action::PushVlan { ethertype } => {
                // OFPAT_PUSH_VLAN = 17
                let mut buf = Vec::new();
                buf.extend_from_slice(&[0x00, 0x11]); // type
                buf.extend_from_slice(&[0x00, 0x08]); // length = 8
                buf.extend_from_slice(&ethertype.to_be_bytes());
                buf.extend_from_slice(&[0x00, 0x00]); // padding
                buf
            }
            Action::PopVlan => {
                // OFPAT_POP_VLAN = 18
                let mut buf = Vec::new();
                buf.extend_from_slice(&[0x00, 0x12]); // type
                buf.extend_from_slice(&[0x00, 0x08]); // length = 8
                buf.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // padding
                buf
            }
            Action::SetQueue { queue_id } => {
                // OFPAT_SET_QUEUE = 21
                let mut buf = Vec::new();
                buf.extend_from_slice(&[0x00, 0x15]); // type
                buf.extend_from_slice(&[0x00, 0x08]); // length = 8
                buf.extend_from_slice(&queue_id.to_be_bytes());
                buf
            }
            Action::Drop => {
                // Drop is represented by empty action list in OpenFlow
                Vec::new()
            }
        }
    }
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
            match_fields: Vec::new(),
            instructions: Vec::new(),
        }
    }

    /// Encode match structure (OpenFlow 1.3 format)
    fn encode_match(&self) -> Vec<u8> {
        let mut match_bytes = Vec::new();
        
        // Encode all OXM fields
        for field in &self.match_fields {
            match_bytes.extend_from_slice(&field.to_bytes());
        }
        
        // Match header: type (1=OFPMT_OXM), length
        let match_length = 4 + match_bytes.len();
        // Pad to 8-byte boundary
        let padded_length = ((match_length + 7) / 8) * 8;
        let padding = padded_length - match_length;
        
        let mut buf = Vec::new();
        buf.extend_from_slice(&[0x00, 0x01]); // type = OFPMT_OXM
        buf.extend_from_slice(&(match_length as u16).to_be_bytes()); // length
        buf.extend_from_slice(&match_bytes);
        buf.extend_from_slice(&vec![0x00; padding]); // padding
        
        buf
    }

    pub fn to_bytes(&self) -> Bytes {
        // Encode match and instructions
        let match_bytes = self.encode_match();
        let mut instruction_bytes = Vec::new();
        for instruction in &self.instructions {
            instruction_bytes.extend_from_slice(&instruction.to_bytes());
        }
        
        // Calculate total length
        // Header: 8 bytes
        // Flow mod fields: 40 bytes (cookie(8) + cookie_mask(8) + table_id(1) + command(1) + 
        //                           idle_timeout(2) + hard_timeout(2) + priority(2) + buffer_id(4) +
        //                           out_port(4) + out_group(4) + flags(2) + padding(2))
        // Match: variable
        // Instructions: variable
        let total_length = OpenFlowHeader::HEADER_SIZE + 40 + match_bytes.len() + instruction_bytes.len();
        
        let mut buf = BytesMut::with_capacity(total_length);
        
        // Update header length
        let mut header = self.header.clone();
        header.length = total_length as u16;
        
        // Encode header
        buf.extend_from_slice(&header.to_bytes());
        
        // Encode flow_mod fields (40 bytes total)
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
        
        // Encode match
        buf.extend_from_slice(&match_bytes);
        
        // Encode instructions
        buf.extend_from_slice(&instruction_bytes);
        
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
        // New FlowMod includes match structure (minimum 8 bytes padded)
        // Actual size depends on match encoding
        println!("FlowMod bytes length: {}", bytes.len());
        assert!(bytes.len() >= 56); // At minimum: header(8) + flow_mod(48)
        // Verify header length field matches actual bytes
        let header_length = u16::from_be_bytes([bytes[2], bytes[3]]);
        assert_eq!(bytes.len(), header_length as usize);
    }
    
    #[test]
    fn test_flow_mod_with_match_and_actions() {
        let mut msg = FlowModMessage::new(1, 0);
        msg.match_fields = vec![
            OxmField::InPort(1),
            OxmField::EthType(0x0800), // IPv4
        ];
        msg.instructions = vec![
            Instruction::ApplyActions(vec![
                Action::Output { port: 2, max_len: 0xFFFF }
            ])
        ];
        
        let bytes = msg.to_bytes();
        println!("FlowMod with match/actions bytes length: {}", bytes.len());
        assert!(bytes.len() > 56); // Should be larger with match and actions
        // Verify header length field matches actual bytes
        let header_length = u16::from_be_bytes([bytes[2], bytes[3]]);
        assert_eq!(bytes.len(), header_length as usize);
    }
}

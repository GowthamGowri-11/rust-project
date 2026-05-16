use crate::types::*;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use serde::{Deserialize, Serialize};

// OpenFlow message types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MessageType {
    Hello = 0,
    Error = 1,
    EchoRequest = 2,
    EchoReply = 3,
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
    PortMod = 15,
    StatsRequest = 16,
    StatsReply = 17,
}

impl MessageType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Hello),
            1 => Some(Self::Error),
            2 => Some(Self::EchoRequest),
            3 => Some(Self::EchoReply),
            5 => Some(Self::FeaturesRequest),
            6 => Some(Self::FeaturesReply),
            7 => Some(Self::GetConfigRequest),
            8 => Some(Self::GetConfigReply),
            9 => Some(Self::SetConfig),
            10 => Some(Self::PacketIn),
            11 => Some(Self::FlowRemoved),
            12 => Some(Self::PortStatus),
            13 => Some(Self::PacketOut),
            14 => Some(Self::FlowMod),
            15 => Some(Self::PortMod),
            16 => Some(Self::StatsRequest),
            17 => Some(Self::StatsReply),
            _ => None,
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
    pub const SIZE: usize = 8;

    pub fn new(version: u8, msg_type: MessageType, length: u16, xid: u32) -> Self {
        Self {
            version,
            msg_type,
            length,
            xid,
        }
    }

    pub fn encode(&self, buf: &mut BytesMut) {
        buf.put_u8(self.version);
        buf.put_u8(self.msg_type as u8);
        buf.put_u16(self.length);
        buf.put_u32(self.xid);
    }

    pub fn decode(buf: &mut Bytes) -> Option<Self> {
        if buf.remaining() < Self::SIZE {
            return None;
        }

        let version = buf.get_u8();
        let msg_type = MessageType::from_u8(buf.get_u8())?;
        let length = buf.get_u16();
        let xid = buf.get_u32();

        Some(Self {
            version,
            msg_type,
            length,
            xid,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketInMessage {
    pub buffer_id: u32,
    pub total_len: u16,
    pub in_port: PortNumber,
    pub reason: PacketInReason,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PacketInReason {
    NoMatch = 0,
    Action = 1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowModMessage {
    pub match_fields: FlowMatch,
    pub cookie: u64,
    pub command: FlowModCommand,
    pub idle_timeout: u16,
    pub hard_timeout: u16,
    pub priority: u16,
    pub buffer_id: u32,
    pub out_port: PortNumber,
    pub flags: u16,
    pub actions: Vec<FlowAction>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FlowModCommand {
    Add = 0,
    Modify = 1,
    ModifyStrict = 2,
    Delete = 3,
    DeleteStrict = 4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturesReply {
    pub datapath_id: SwitchId,
    pub n_buffers: u32,
    pub n_tables: u8,
    pub capabilities: u32,
    pub ports: Vec<PortInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControllerMessage {
    Hello,
    FeaturesRequest,
    FlowMod(FlowModMessage),
    PacketOut {
        buffer_id: u32,
        in_port: PortNumber,
        actions: Vec<FlowAction>,
        data: Vec<u8>,
    },
    StatsRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SwitchMessage {
    Hello,
    FeaturesReply(FeaturesReply),
    PacketIn(PacketInMessage),
    PortStatus {
        reason: u8,
        port: PortInfo,
    },
    FlowRemoved {
        cookie: u64,
        priority: u16,
        reason: u8,
        duration_sec: u32,
        packet_count: u64,
        byte_count: u64,
    },
}

use crate::{errors::PacketError, registry::Payload};
use anyhow::bail;
use serde::Serialize;
use serde_json::*;
use std::fmt::Display;
#[derive(Debug, Clone)]
pub enum PacketType {
    Unknown,
    Event,
    Request,
    Response,
    Discover,
    Info,
    Disconnect,
    Heartbeat,
    Ping,
    Pongs,
    GossipReq,
    GossipRes,
    GossipHello,
}

impl Display for PacketType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl From<PacketType> for String {
    fn from(packet: PacketType) -> Self {
        match packet {
            PacketType::Unknown => "???".to_string(),
            PacketType::Event => "EVENT".to_string(),
            PacketType::Request => "REQ".to_string(),
            PacketType::Response => "RES".to_string(),
            PacketType::Discover => "DISCOVER".to_string(),
            PacketType::Info => "INFO".to_string(),
            PacketType::Disconnect => "DISCONNECT".to_string(),
            PacketType::Heartbeat => "HEARTBEAT".to_string(),
            PacketType::Ping => "PING".to_string(),
            PacketType::Pongs => "PONG".to_string(),
            PacketType::GossipReq => "GOSSIP_REQ".to_string(),
            PacketType::GossipRes => "GOSSIP_RES".to_string(),
            PacketType::GossipHello => "GOSSIP_HELLO".to_string(),
        }
    }
}

impl PartialEq for PacketType {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Packet {
    //type
    pub(crate) tipe: PacketType,
    pub(crate) target: Option<String>,
    pub(crate) payload: PacketPayload,
}

impl Packet {
    pub(crate) fn new(tipe: PacketType, target: Option<String>, payload: PacketPayload) -> Self {
        Self {
            tipe,
            target,
            payload,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum PacketPayload {
    FullPayload(FullPacketPayload),
    Custom(serde_json::Value),
}

impl PacketPayload {
    pub(crate) fn get_custom(self) -> anyhow::Result<serde_json::Value> {
        match self {
            PacketPayload::FullPayload(full) => {
                let data = serde_json::to_value(full);
                match data {
                    Ok(value) => Ok(value),
                    Err(err) => bail!(PacketError::CannotParse(err.to_string())),
                }
            }
            PacketPayload::Custom(custom) => Ok(custom),
        }
    
    }

    pub (crate) fn get_full(self)-> anyhow::Result<FullPacketPayload>{
        match self {
            PacketPayload::FullPayload(full) => Ok(full),
            PacketPayload::Custom(_) => bail!(PacketError::NoFullPacket),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FullPacketPayload {
    pub(crate) action: String,
    pub(crate) sender: String,
    pub(crate) id: String,
    pub(crate) parent_id: String,
    pub(crate) request_id: String,
    pub(crate) caller: String,
    pub(crate) level: usize,
    pub(crate) tracing: bool,
    pub(crate) meta: Payload,
    pub(crate) timeout: Option<i64>,
    pub(crate) params: Payload,
    pub(crate) groups: Option<Vec<String>>,
}

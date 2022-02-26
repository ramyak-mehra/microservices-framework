use crate::{errors::PacketError, registry::Payload};
use anyhow::bail;
use serde::Serialize;
use serde_json::*;
use std::{any, fmt::Display};
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
    Null,
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
            PacketType::Null => "NULL".to_string(),
        }
    }
}

impl PartialEq for PacketType {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

pub(crate) trait PacketPayload
where
    Self: Sized,
{
    fn tipe(&self) -> PacketType;
    fn event_payload(mut self) -> anyhow::Result<PayloadEvent> {
        bail!("Not an event payload")
    }
    fn request_paylaod(mut self) -> anyhow::Result<PayloadRequest> {
        bail!("Not a request payload")
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Packet<P: PacketPayload> {
    //type
    pub(crate) tipe: PacketType,
    pub(crate) target: Option<String>,
    pub(crate) payload: P,
}

impl<P: PacketPayload> Packet<P> {
    pub(crate) fn new(tipe: PacketType, target: Option<String>, payload: P) -> Self {
        Self {
            tipe,
            target,
            payload,
        }
    }
    pub(crate) fn from_payload<T:PacketPayload+Send>(self, payload: T) -> Packet<T> {
        Packet {
            payload,
            target: self.target,
            tipe: self.tipe,
        }
    }
}

impl PacketPayload for PayloadRequest {
    fn tipe(&self) -> PacketType {
        PacketType::Request
    }

    fn request_paylaod(self) -> anyhow::Result<PayloadRequest> {
        Ok(self)
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PayloadRequest {
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

impl PacketPayload for PayloadEvent {
    fn tipe(&self) -> PacketType {
        PacketType::Event
    }
    fn event_payload(mut self) -> anyhow::Result<PayloadEvent> {
        Ok(self)
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PayloadEvent {
    pub(crate) id: String,
    pub(crate) event: String,
    pub(crate) data: Payload,
    pub(crate) groups: Vec<String>,
    pub(crate) broadcast: bool,
    pub(crate) meta: Payload,
    pub(crate) level: usize,
    pub(crate) tracing: bool,
    pub(crate) parentID: Option<String>,
    pub(crate) requestID: String,
    pub(crate) caller: String,
    pub(crate) needAck: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PayloadHeartbeat {
    pub(crate) cpu: Option<u32>,
}
impl PacketPayload for PayloadHeartbeat {
    fn tipe(&self) -> PacketType {
        PacketType::Heartbeat
    }
}
#[derive(Debug, Clone, Serialize)]
pub(crate) struct PayloadPong {
    pub(crate) sender: String,
    pub(crate) time: String,
    pub(crate) arrived: String,
    pub(crate) id: String,
}

impl PacketPayload for PayloadPong {
    fn tipe(&self) -> PacketType {
        PacketType::Pongs
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PayloadPing {
    pub(crate) sender: String,
    pub(crate) time: String,
    pub(crate) id: String,
}
impl PacketPayload for PayloadPing {
    fn tipe(&self) -> PacketType {
        PacketType::Ping
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PayloadNull {}
impl PacketPayload for PayloadNull {
    fn tipe(&self) -> PacketType {
        PacketType::Null
    }
}

use crate::registry::{Client, NodeRawInfo, Payload, ServiceItemInfo};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Clone, Serialize)]
#[repr(usize)]
pub(crate) enum DataType {
    Undefined = 0,
    Null = 1,
    Json = 2,
    Buffer = 3,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) enum PacketType {
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
impl From<&str> for PacketType {
    fn from(val: &str) -> Self {
        match val {
            "EVENT" => PacketType::Event,
            "REQ" => PacketType::Request,
            "RES" => PacketType::Response,
            "DISCOVER" => PacketType::Discover,
            "INFO" => PacketType::Info,
            "DISCONNECT" => PacketType::Disconnect,
            "HEARTBEAT" => PacketType::Heartbeat,
            "PING" => PacketType::Ping,
            " PONG" => PacketType::Pongs,
            "GOSSIP_REQ" => PacketType::GossipReq,
            "GOSSIP_RES" => PacketType::GossipRes,
            "GOSSIP_HELLO" => PacketType::GossipHello,

            _ => PacketType::Unknown,
        }
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

pub(crate) trait PacketPayload
where
    Self: Sized,
{
    fn tipe(&self) -> PacketType;
    fn sender(&self) -> &str;
    fn instance_id(&self) -> &str {
        panic!("Only Info packets have instance_id")
    }
    fn request_paylaod(self) -> PayloadRequest {
        panic!("Not a request payload.")
    }
    fn response_payload(self) -> PayloadResponse {
        panic!("Not a response payload.")
    }
    fn event_payload(self) -> PayloadEvent {
        panic!("Not an event payload.")
    }
    fn heartbeat_payload(self) -> PayloadHeartbeat {
        panic!("Not a heartbeat payload.")
    }
    fn pong_payload(self) -> PayloadPong {
        panic!("Not a pong payload.")
    }
    fn ping_payload(self) -> PayloadPing {
        panic!("Not a ping payload.")
    }
    fn info_payload(self) -> PayloadInfo {
        panic!("Not an info payload.")
    }
    fn disconnect_payload(self) -> PayloadDisconnect {
        panic!("Not a disconnect payload")
    }
    fn discover_payload(self) -> PayloadDiscover {
        panic!("Not a discover payload")
    }
}

#[derive(Debug, Clone, Serialize)]
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
    pub(crate) fn from_payload<T: PacketPayload + Send>(self, payload: T) -> Packet<T> {
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
    fn request_paylaod(self) -> PayloadRequest {
        self
    }
    fn sender(&self) -> &str {
        &self.sender
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PayloadRequest {
    pub(crate) sender: String,
    pub(crate) id: String,
    pub(crate) action: String,
    pub(crate) params: Option<Payload>,
    pub(crate) meta: Payload,
    pub(crate) timeout: Option<i64>,
    pub(crate) level: usize,
    pub(crate) tracing: bool,
    pub(crate) parent_id: Option<String>,
    pub(crate) request_id: String,
    pub(crate) stream: bool,
    pub(crate) seq: i32,
    pub(crate) caller: String,
}

impl PacketPayload for PayloadResponse {
    fn tipe(&self) -> PacketType {
        PacketType::Response
    }

    fn sender(&self) -> &str {
        &self.sender
    }
    fn response_payload(self) -> Self {
        self
    }
}
#[derive(Debug, Serialize, Deserialize)]

pub(crate) struct PayloadResponse {
    
    pub(crate) sender: String,
    pub(crate) id: String,
    pub(crate) success: bool,
    #[serde(with = "serde_bytes")]
    pub(crate) data: Vec<u8>,
    pub(crate) error: String,
    pub(crate) meta: String,
    pub(crate) stream: bool,
    pub(crate) seq: i32,
}

impl PacketPayload for PayloadEvent {
    fn tipe(&self) -> PacketType {
        PacketType::Event
    }
    fn event_payload(self) -> Self {
        self
    }
    fn sender(&self) -> &str {
        &self.sender
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PayloadEvent {
    pub(crate) sender: String,
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
    pub(crate) cpu: u32,
    pub(crate) sender: String,
}
impl PacketPayload for PayloadHeartbeat {
    fn tipe(&self) -> PacketType {
        PacketType::Heartbeat
    }
    fn sender(&self) -> &str {
        &self.sender
    }
    fn heartbeat_payload(self) -> Self {
        self
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
    fn sender(&self) -> &str {
        &self.sender
    }
    fn pong_payload(self) -> Self {
        self
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
    fn sender(&self) -> &str {
        &self.sender
    }
    fn ping_payload(self) -> Self {
        self
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PayloadDisconnect {
    pub(crate) sender: String,
}
impl PacketPayload for PayloadDisconnect {
    fn tipe(&self) -> PacketType {
        PacketType::Disconnect
    }
    fn sender(&self) -> &str {
        &self.sender
    }
}
#[derive(Debug, Clone, Serialize)]
pub(crate) struct PayloadDiscover {
    pub(crate) sender: String,
}
impl PacketPayload for PayloadDiscover {
    fn tipe(&self) -> PacketType {
        PacketType::Discover
    }
    fn sender(&self) -> &str {
        &self.sender
    }
    fn discover_payload(self)->Self{
        self
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PayloadInfo {
    pub(crate) sender: String,
    pub(crate) services: Vec<ServiceItemInfo>,
    pub(crate) config: String,
    pub(crate) ip_list: Vec<String>,
    pub(crate) hostame: String,
    pub(crate) client: Client,
    pub(crate) seq: usize,
    pub(crate) instance_id: String,
    pub(crate) meta_data: String,
}

impl PacketPayload for PayloadInfo {
    fn tipe(&self) -> PacketType {
        PacketType::Info
    }
    fn sender(&self) -> &str {
        &self.sender
    }

    fn instance_id(&self) -> &str {
        &self.instance_id
    }
    fn info_payload(self) -> Self {
        self
    }
}
impl PayloadInfo {
    pub(crate) fn from_node_info(item: NodeRawInfo, sender: String) -> Self {
        Self {
            sender,
            services: item.services,
            config: item.config,
            ip_list: item.ip_list,
            hostame: item.hostame,
            client: item.client,
            seq: item.seq,
            instance_id: item.instance_id,
            meta_data: item.meta_data,
        }
    }
}

use std::net::IpAddr;

use chrono::Duration;
use serde::{Serialize, Serializer};
use serde_json::Value;

use super::{discoverers::PayloadInfo, Payload};

#[derive(PartialEq, Eq, Clone, Debug, Serialize)]
pub(crate)struct Node {
    pub(crate)id: String,
    instance_id: Option<String>,
    pub(crate)available: bool,
    pub(crate)local: bool,
    #[serde(serialize_with = "duration_serialzie")]
    last_heartbeat_time: Duration,
    metadata: Value,
    /* feields that need to be added later.
    config

    */
    client: Option<Client>,
    ip_list: Vec<String>,
    port: Option<u16>,
    hostname: Option<String>,
    udp_address: Option<IpAddr>,
    raw_info: Option<NodeRawInfo>,
    pub(crate)cpu: u32,
    /*
    cpuseq
    */
    pub(crate)services: Vec<String>,
    pub(crate)seq: usize,
    #[serde(serialize_with = "option_duration_serialzie")]
    offline_since: Option<Duration>,
}

impl Node {
    pub(crate)fn new(id: String) -> Self {
        Self {
            id,
            instance_id: None,
            available: true,
            local: false,
            client: None,
            raw_info: None,
            metadata: Value::Null,
            //TODO:
            /*
            change this later with actual process uptime.
            */
            last_heartbeat_time: Duration::seconds(1),
            ip_list: Vec::new(),
            port: None,
            hostname: None,
            udp_address: None,
            services: Vec::new(),
            seq: 0,
            cpu: 0,
            offline_since: None,
        }
    }
    pub(crate)fn update(&mut self) {
        todo!()
    }
    pub(crate)fn update_local_info(&mut self) {
        todo!()
    }
    pub(crate)fn hearbeat(&mut self) {
        if !self.available {
            self.available = true;
            self.offline_since = None;
        }
        todo!()
    }
    pub(crate)fn disconnected(&mut self) {
        if self.available {
            self.seq = self.seq.saturating_add(1);
            /* update this with process uptime
            self.offline_since =
             */
        }
        self.available = false;
    }

    pub(crate)fn services_len(&self) -> usize {
        self.services.len()
    }
    pub(crate)fn set_local(mut self, value: bool) -> Self {
        self.local = value;
        self
    }
    pub(crate)fn set_ip_list(mut self, ip_list: Vec<String>) -> Self {
        self.ip_list = ip_list;
        self
    }
    pub(crate)fn set_instance_id(mut self, instance_id: String) -> Self {
        self.instance_id = Some(instance_id);
        self
    }
    pub(crate)fn set_hostname(mut self, hostname: String) -> Self {
        self.hostname = Some(hostname);
        self
    }
    pub(crate)fn set_client(mut self, client: Client) -> Self {
        self.client = Some(client);
        self
    }
    pub(crate)fn set_seq(mut self, seq: usize) -> Self {
        self.seq = seq;
        self
    }
    pub(crate)fn set_metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }
    pub(crate)fn raw_info(&self)->&NodeRawInfo{
        &self.raw_info.as_ref().unwrap()
    }
}
#[derive(PartialEq, Eq, Clone, Debug, Serialize)]

pub(crate)struct Client {
    pub(crate) client_type: String,
    pub(crate) version: String,
    pub(crate) lang_version: String,
}

fn duration_serialzie<S>(x: &Duration, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_i64(x.num_milliseconds())
}
fn option_duration_serialzie<S>(x: &Option<Duration>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match x {
        Some(x) => s.serialize_i64(x.num_milliseconds()),
        None => s.serialize_none(),
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate)struct NodeRawInfo {
    pub(crate)services: Vec<String>,
    pub(crate)config: String,
    pub(crate)ip_list: Vec<String>,
    pub(crate)hostame: String,
    pub(crate)client: Client,
    pub(crate)seq: usize,
    pub(crate)instance_id: String,
    pub(crate)meta_data: String,
}
impl From<PayloadInfo> for NodeRawInfo {
    fn from(item: PayloadInfo) -> Self {
        Self {
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

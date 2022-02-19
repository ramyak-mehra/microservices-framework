use std::net::IpAddr;

use chrono::Duration;
use serde_json::Value;

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Node {
    pub id: String,
    instance_id: Option<String>,
    pub available: bool,
    pub local: bool,
    last_heartbeat_time: Duration,
    metadata : Value,
    /* feields that need to be added later.
    config

    */
    client: Option<Client>,
    ip_list: Vec<String>,
    port: Option<u16>,
    hostname: Option<String>,
    udp_address: Option<IpAddr>,
    pub raw_info: Option<Value>,
    /*
    cpu
    cpuseq
    */
    pub services: Vec<String>,
    pub seq: usize,
    offline_since: Option<Duration>,
}

impl Node {
    pub fn new(id: String) -> Self {
        Self {
            id,
            instance_id: None,
            available: true,
            local: false,
            client: None,
            raw_info: None,
            metadata : Value::Null,
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
            offline_since: None,
        }
    }
    pub fn update(&mut self) {
        todo!()
    }
    pub fn update_local_info(&mut self) {
        todo!()
    }
    pub fn hearbeat(&mut self) {
        if !self.available {
            self.available = true;
            self.offline_since = None;
        }
        todo!()
    }
    pub fn disconnect(&mut self) {
        if self.available {
            self.seq = self.seq.saturating_add(1);
            /* update this with process uptime
            self.offline_since =
             */
        }
        self.available = false;
    }

    pub fn services_len(&self) -> usize {
        self.services.len()
    }
    pub fn set_local(mut self, value: bool) -> Self {
        self.local = value;
        self
    }
    pub fn set_ip_list(mut self, ip_list: Vec<String>) -> Self {
        self.ip_list = ip_list;
        self
    }
    pub fn set_instance_id(mut self, instance_id: String) -> Self {
        self.instance_id = Some(instance_id);
        self
    }
    pub fn set_hostname(mut self, hostname: String) -> Self {
        self.hostname = Some(hostname);
        self
    }
    pub fn set_client(mut self, client: Client) -> Self {
        self.client = Some(client);
        self
    }
    pub fn set_seq(mut self, seq: usize) -> Self {
        self.seq = seq;
        self
    }
    pub fn set_metadata(mut self , metadata:Value)->Self{
        self.metadata = metadata;
        self
    }
}
#[derive(PartialEq, Eq, Clone, Debug)]

pub struct Client {
    pub(crate) client_type: String,
    pub(crate) version: String,
    pub(crate) lang_version: String,
}

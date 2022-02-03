use std::{net::IpAddr, sync::Arc};

use chrono::Duration;
use serde_json::Value;

use super::ServiceItem;

#[derive(PartialEq, Eq, Clone)]
pub(crate)struct Node {
    pub(crate)id: String,
    instance_id: Option<String>,
    pub(crate)available: bool,
    pub(crate)local: bool,
    last_heartbeat_time: Duration,
    /* feields that need to be added later.
    config

    metadata
    */
    client: Option<Client>,
    ip_list: Vec<IpAddr>,
    port: Option<u16>,
    hostname: Option<String>,
    udp_address: Option<IpAddr>,
    pub(crate)raw_info: Option<Value>,
    /*
    cpu
    cpuseq
    */
    pub(crate)services: Vec<Arc<ServiceItem>>,
    pub(crate)seq: usize,
    offline_since: Option<Duration>,
}

impl Node {
    pub(crate)fn new(id: String) -> Self {
        Self {
            id: id,
            instance_id: None,
            available: true,
            local: false,
            client: None,
            raw_info: None,
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
    pub(crate)fn disconnect(&mut self) {
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
    pub(crate)fn set_ip_list(mut self, ip_list: Vec<IpAddr>) -> Self {
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
}
#[derive(PartialEq, Eq, Clone)]

pub(crate)struct Client {
    pub(crate) client_type: String,
    pub(crate) version: String,
    pub(crate) lang_version: String,
}

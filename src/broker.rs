use std::{
    collections::HashMap,
    sync::{mpsc::Receiver, Arc},
};

use anyhow::{bail, Result};

use chrono::Utc;
use serde_json::Value;
use tokio::sync::oneshot;

use crate::{registry::Logger, service::ServiceSpec, Registry, Service};

#[derive(Default)]
pub struct BrokerOptions {
    pub max_call_level: usize,
}

pub struct ServiceBroker {
    reciever: Receiver<ServiceBrokerMessage>,
    started: bool,
    namespace: Option<String>,
    metdata: HashMap<String, String>,
    pub node_id: String,
    instance: String,
    services: Vec<Service>,
    pub transit: Option<Transit>,
    pub logger: Arc<Logger>,
    pub options: BrokerOptions,
    /*
    local bus

    logger
    metricss
    middlewere
    cacher
    serializer
    error generator
    validator
    tracer
    transporter
    */
    registry: Registry,
}

pub struct Transit {}

impl ServiceBroker {
    fn start(&mut self) {
        let time = Utc::now();
        self.started = true;
    }

    fn add_local_service(&mut self, service: Service) {
        self.services.push(service);
    }
    fn register_local_service(&mut self, service: ServiceSpec) {
        self.registry.register_local_service(service);
    }

    fn destroy_service(&mut self, name: &str, version: &str) -> Result<()> {
        let service_index = self.get_local_service_index(name, version);
        if let None = service_index {
            bail!(
                "no service with the name {} and version {} found",
                name,
                version
            );
        }
        let service_index = service_index.unwrap();
        let mut full_name = "".to_string();

        {
            let service = self.services.get_mut(service_index).unwrap();
            full_name = service.full_name.clone();
            service.stop();
        }
        {
            self.services.remove(service_index);
        }

        self.registry
            .unregister_service(&full_name, Some(&self.node_id));
        self.services_changed(true);
        Ok(())
    }

    fn services_changed(&self, local_service: bool) {
        if (self.started && local_service) {
            todo!("notifify remote nodes")
        }
    }
    fn get_local_service_index(&self, name: &str, version: &str) -> Option<usize> {
        self.services.iter().position(|s| {
            if s.name == name && s.version == version {
                return true;
            }
            return false;
        })
    }
}
#[derive(Debug)]
pub enum ServiceBrokerMessage {
    AddLocalService(Service),
    RegisterLocalService(ServiceSpec),
    WaitForServices {
        dependencies: Vec<String>,
        timeout: i64,
        interval: i64,
    },
    Broadcast {
        event_name: String,
        data: Value,
        opts: Value,
    },
    Emit {
        event_name: String,
        data: Value,
        opts: Value,
    },
    Call {
        action_name: String,
        params: Value,
        opts: Value,
        result_channel: oneshot::Sender<HandlerResult>,
    },
}
impl PartialEq for ServiceBrokerMessage {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::AddLocalService(l0), Self::AddLocalService(r0)) => l0 == r0,
            (Self::RegisterLocalService(l0), Self::RegisterLocalService(r0)) => l0 == r0,
            (Self::WaitForServices { dependencies: l_dependencies, timeout: l_timeout, interval: l_interval }, Self::WaitForServices { dependencies: r_dependencies, timeout: r_timeout, interval: r_interval }) => l_dependencies == r_dependencies && l_timeout == r_timeout && l_interval == r_interval,
            (Self::Broadcast { event_name: l_event_name, data: l_data, opts: l_opts }, Self::Broadcast { event_name: r_event_name, data: r_data, opts: r_opts }) => l_event_name == r_event_name && l_data == r_data && l_opts == r_opts,
            (Self::Emit { event_name: l_event_name, data: l_data, opts: l_opts }, Self::Emit { event_name: r_event_name, data: r_data, opts: r_opts }) => l_event_name == r_event_name && l_data == r_data && l_opts == r_opts,
            (Self::Call { action_name: l_action_name, params: l_params, opts: l_opts, result_channel: l_result_channel }, Self::Call { action_name: r_action_name, params: r_params, opts: r_opts, result_channel: r_result_channel }) => l_action_name == r_action_name && l_params == r_params && l_opts == r_opts,
            _ => false
        }
    }
}
#[derive(PartialEq, Debug)]
pub struct HandlerResult {}

fn remove_from_list<T: PartialEq + Eq>(list: &mut Vec<T>, value: &T) {
    list.retain(|t| {
        if t == value {
            return false;
        }
        return true;
    });
}

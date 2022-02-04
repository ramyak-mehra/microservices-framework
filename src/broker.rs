use std::{
    collections::HashMap,
    sync::{
        mpsc::{Receiver, },
        Arc,
    },
};

use anyhow::{bail, Result};

use chrono:: Utc;

use crate::{
    registry:: Logger,
    service::ServiceSpec,
    Registry, Service,
};

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
    /*
    local bus
    options
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
#[derive(PartialEq ,Debug)]
pub enum ServiceBrokerMessage {
    AddLocalService(Service),
    RegisterLocalService(ServiceSpec),
    WaitForServices {
        dependencies: Vec<String>,
        timeout: i64,
        interval: i64,
    },
}

fn remove_from_list<T: PartialEq + Eq>(list: &mut Vec<T>, value: &T) {
    list.retain(|t| {
        if t == value {
            return false;
        }
        return true;
    });
}

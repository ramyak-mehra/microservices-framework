use std::{
    collections::HashMap,
    sync::mpsc::{Receiver, Sender},
};

use chrono::{DateTime, Utc};

use crate::{Registry, Service};

struct ServiceBroker {
    reciever: Receiver<ServiceBrokerAction>,
    started: bool,
    namespace: Option<String>,
    metdata: HashMap<String, String>,
    node_id: String,
    instance: String,
    services: Vec<Service>,
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
    registry: Option<Registry>,
}

impl ServiceBroker {
    fn start(&mut self) {
        let time = Utc::now();
        self.started = true;
    
    }

    fn add_local_service(&mut self , service : Service){
        self.services.push(service);
    }
    
}

enum ServiceBrokerAction {}

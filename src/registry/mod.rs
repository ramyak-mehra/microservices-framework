pub mod action_catalog;
pub mod action_endpoint;
pub mod endpoint_list;
pub mod event_endpoint;
pub mod node;
pub mod node_catalog;
pub mod service_catalog;
pub mod service_item;
use std::collections::HashMap;
pub use std::sync::Arc;

pub use action_endpoint::ActionEndpoint;
pub use endpoint_list::EndpointList;
pub use node::{Client, Node};
use service_item::ServiceItem;
// pub use event_endpoint::EventEndpoint;

type ActionsMap = HashMap<String, EndpointList>;

#[derive(PartialEq, Eq)]
pub struct Logger {}
#[derive(PartialEq, Eq)]
pub struct Broker {
    node_id: String,
    instance_id: String,
    moleculer_version: String,
}

#[derive(PartialEq, Eq)]
pub struct Registry {
    logger: Arc<Logger>,
}
impl Registry {
    pub fn logger(&self) -> &Arc<Logger> {
        &self.logger
    }
}

trait FnType {}

#[derive(PartialEq, Eq, Clone)]
pub struct Action {
    name: String,
}
impl FnType for Action {}
struct Event {}
impl FnType for Event {}
pub struct Strategy {}

trait EndpointTrait {
    fn node(&self) -> Node;
    fn service(&self) -> ServiceItem;
    fn update<F>(&mut self, data: F)
    where
        F: FnType;
}

#[derive(PartialEq, Eq, Clone)]
struct Endpoint {
    registry: Arc<Registry>,
    broker: Arc<Broker>,
    node: Arc<Node>,
    service: Arc<ServiceItem>,
    state: bool,
    id: String,
    local: bool,
}

impl Endpoint {
    fn new(
        registry: Arc<Registry>,
        broker: Arc<Broker>,
        node: Arc<Node>,
        service: Arc<ServiceItem>,
    ) -> Self {
        let local = node.id == broker.node_id;
        let id = node.id.to_string();
        Self {
            registry,
            broker,
            node,
            service,
            state: true,
            id: id,
            local,
        }
    }
}

enum EndpointType {
    Action,
    Event,
}

pub struct Service {
    name: String,
    full_name: String,
    version: String,
    /*
    settings ,
    metadata
    */
}

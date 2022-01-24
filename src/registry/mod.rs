pub mod action_catalog;
pub mod action_endpoint;
pub mod endpoint_list;
pub mod event_endpoint;
pub use std::sync::Arc;

pub use action_endpoint::ActionEndpoint;
pub use endpoint_list::EndpointList;
// pub use event_endpoint::EventEndpoint;

pub struct Logger {}
pub struct Broker {
    node_id: String,
}
pub struct Registry {
    logger: Arc<Logger>,
}
impl Registry {
    pub fn logger(&self) -> &Arc<Logger> {
        &self.logger
    }
}
#[derive(PartialEq, Eq)]
pub struct Node {
    id: String,
}
#[derive(PartialEq, Eq)]
pub struct ServiceItem {
    name: String,
}

trait FnType {}

#[derive(Clone)]
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

#[derive(Clone)]
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
        let id = node.id.clone();
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

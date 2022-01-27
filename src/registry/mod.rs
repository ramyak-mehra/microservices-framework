pub mod action_catalog;
pub mod action_endpoint;
pub mod endpoint_list;
pub mod event_endpoint;
pub mod node;
pub mod node_catalog;
pub mod registry;
pub mod service_catalog;
pub mod service_item;

use std::collections::HashMap;
use std::sync::Arc;

use crate::strategies::Strategy;
use action_catalog::ActionCatalog;
pub use action_endpoint::ActionEndpoint;
pub use endpoint_list::EndpointList;
pub use event_endpoint::EventEndpoint;
pub use node::{Client, Node};
use node_catalog::NodeCatalog;
pub use registry::Registry;
use service_catalog::ServiceCatalog;
use service_item::ServiceItem;
// pub use event_endpoint::EventEndpoint;

type ActionsMap = HashMap<String, EndpointList<ActionEndpoint>>;

#[derive(PartialEq, Eq)]
pub struct Logger {}
#[derive(PartialEq, Eq)]
pub struct Broker {
    node_id: String,
    instance_id: String,
    moleculer_version: String,
    logger: Arc<Logger>,
}

trait FnType {}

#[derive(PartialEq, Eq, Clone)]
pub struct Action {
    name: String,
    visibility: Visibility,
}

#[derive(PartialEq, Eq, Clone)]
enum Visibility {
    Published,
    Public,
    Protected,
    Private,
}
#[derive(PartialEq, Eq, Clone)]
pub struct Event {}
impl FnType for Event {}

///Endpoint trait for endpoint list
pub trait EndpointTrait {
    ///Data is eiter an Action struct or Event structs
    type Data;
    fn new(
        registry: Arc<Registry>,
        broker: Arc<Broker>,
        node: Arc<Node>,
        service: Arc<ServiceItem>,
        data: Self::Data,
    ) -> Self;
    fn node(&self) -> &Node;
    fn service(&self) -> &ServiceItem;
    fn update(&mut self, data: Self::Data);
    fn is_local(&self) -> bool;
    fn is_available(&self) -> bool;
    fn id(&self) -> &str;
    fn service_name(&self) -> &str;
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
#[derive(PartialEq, Eq)]
pub struct Opts<T: Strategy> {
    strategy: T,
}

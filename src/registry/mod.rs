pub(crate)mod action_catalog;
pub(crate)mod action_endpoint;
pub(crate)mod endpoint_list;
pub(crate)mod event_endpoint;
pub(crate)mod node;
pub(crate)mod node_catalog;
pub(crate)mod registry;
pub(crate)mod service_catalog;
pub(crate)mod service_item;

use std::collections::HashMap;
use std::sync::Arc;

use super::service::Service;
use crate::strategies::Strategy;
use action_catalog::ActionCatalog;
pub(crate)use action_endpoint::ActionEndpoint;
pub(crate)use endpoint_list::EndpointList;
pub(crate)use event_endpoint::EventEndpoint;
pub(crate)use node::{Client, Node};
use node_catalog::NodeCatalog;
use regex::Regex;
pub(crate)use registry::Registry;
use service_catalog::ServiceCatalog;
use service_item::ServiceItem;
use lazy_static::lazy_static;
// pub(crate)use event_endpoint::EventEndpoint;


type ActionsMap = HashMap<String, EndpointList<ActionEndpoint>>;

fn get_internal_service_regex_match(text: &str) -> bool {
    lazy_static! {
        static ref RE: Regex =  Regex::new(r"^\$").unwrap();

    }
    RE.is_match(text)
}


#[derive(PartialEq, Eq)]
pub(crate)struct Logger {}

trait FnType {}

#[derive(PartialEq, Eq, Clone)]
pub(crate)struct Action {
    pub(crate)name: String,
    visibility: Visibility,
    handler: fn(),
    // service: Option<Service>,
}

impl Action {
    pub(crate)fn new(name: String, handler: fn()) -> Self {
        Self {
            name,
            visibility: Visibility::Protected,
            handler,
            // service: None,
        }
    }
    // pub(crate)fn set_service(mut self, service: Service) -> Action {
    //     self.service = Some(service);
    //     self
    // }
}

#[derive(PartialEq, Eq, Clone)]
enum Visibility {
    Published,
    Public,
    Protected,
    Private,
}
#[derive(PartialEq, Eq, Clone)]
pub(crate)struct Event {}
impl FnType for Event {}

///Endpoint trait for endpoint list
pub(crate)trait EndpointTrait {
    ///Data is eiter an Action struct or Event structs
    type Data;
    fn new(node: Arc<Node>, service: Arc<ServiceItem>, data: Self::Data) -> Self;
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
    node: Arc<Node>,
    service: Arc<ServiceItem>,
    state: bool,
    id: String,
    local: bool,
}

impl Endpoint {
    fn new(node: Arc<Node>, service: Arc<ServiceItem>) -> Self {
        let id = node.id.to_string();
        let local = service.local;
        Self {
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

#[derive(PartialEq, Eq)]
pub(crate)struct Opts<T: Strategy> {
    strategy: T,
}
pub(crate)struct ListOptions {
    only_local: bool,
    only_available: bool,
    skip_internal: bool,
    with_actions: bool,
    with_events: bool,
    grouping: bool,
}

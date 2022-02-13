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

use super::service::Service;
use crate::strategies::Strategy;
use action_catalog::ActionCatalog;
pub use action_endpoint::ActionEndpoint;
pub use endpoint_list::EndpointList;
pub use event_endpoint::EventEndpoint;
use lazy_static::lazy_static;
pub use node::{Client, Node};
use node_catalog::NodeCatalog;
use regex::Regex;
pub use registry::Registry;
use service_catalog::ServiceCatalog;
use service_item::ServiceItem;
// pub use event_endpoint::EventEndpoint;

type ActionsMap = HashMap<String, EndpointList<ActionEndpoint>>;

fn get_internal_service_regex_match(text: &str) -> bool {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^\$").unwrap();
    }
    RE.is_match(text)
}

#[derive(PartialEq, Eq)]
pub struct Logger {}

trait FnType {}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Action {
    pub name: String,
    visibility: Visibility,
    handler: fn(),
    // service: Option<Service>,
}

impl Action {
    pub fn new(name: String, handler: fn()) -> Self {
        Self {
            name,
            visibility: Visibility::Protected,
            handler,
            // service: None,
        }
    }
    // pub fn set_service(mut self, service: Service) -> Action {
    //     self.service = Some(service);
    //     self
    // }
}

#[derive(PartialEq, Eq, Clone, Debug)]
enum Visibility {
    Published,
    Public,
    Protected,
    Private,
}
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Event {}
impl FnType for Event {}

///Endpoint trait for endpoint list
pub trait EndpointTrait {
    ///Data is eiter an Action struct or Event structs
    type Data;
    fn new(node_id: &str, service: &ServiceItem, data: Self::Data) -> Self;
    fn node(&self) -> &str;
    fn update(&mut self, data: Self::Data);
    fn is_local(&self) -> bool;
    fn is_available(&self) -> bool;
    fn id(&self) -> &str;
    fn service_name(&self) -> &str;
}

#[derive(PartialEq, Eq, Clone)]
struct Endpoint {
    service: String,
    state: bool,
    node_id: String,
    local: bool,
}

impl Endpoint {
    fn new(node_id: &str, service: &ServiceItem) -> Self {
        let local = service.local;
        let service = service.unique_name();
        Self {
            service,
            state: true,
            node_id: node_id.to_string(),
            local,
        }
    }
}

enum EndpointType {
    Action,
    Event,
}

#[derive(PartialEq, Eq)]
pub struct Opts<T: Strategy> {
    strategy: T,
}
pub struct ListOptions {
    only_local: bool,
    only_available: bool,
    skip_internal: bool,
    with_actions: bool,
    with_events: bool,
    grouping: bool,
}
use thiserror::Error;
#[derive(Error, Debug)]
enum RegistryError {
    #[error("No local node found")]
    NoLocalNodeFound,
    #[error("No service found")]
    NoServiceItemFound,
}

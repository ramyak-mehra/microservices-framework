pub(crate)mod action_catalog;
pub(crate)mod action_endpoint;
pub(crate)mod endpoint_list;
pub(crate)mod event_endpoint;
pub(crate)mod node;
pub(crate)mod node_catalog;
pub(crate)mod registry;
pub(crate)mod service_catalog;
pub(crate)mod service_item;
pub(crate)mod discoverers;
pub(crate)mod event_catalog;

use std::collections::HashMap;
use std::sync::Arc;

use super::service::Service;
use crate::{context::Context, strategies::Strategy, HandlerResult};
use action_catalog::ActionCatalog;
pub(crate)use action_endpoint::ActionEndpoint;
use event_catalog::EventCatalog;
pub(crate)use endpoint_list::EndpointList;
pub(crate)use event_endpoint::EventEndpoint;
use lazy_static::lazy_static;
pub(crate)use node::{Client, Node , NodeRawInfo};
use node_catalog::NodeCatalog;

use regex::Regex;
pub(crate)use registry::Registry;
use serde::Serialize;
use service_catalog::ServiceCatalog;
use service_item::ServiceItem;
// pub(crate)use event_endpoint::EventEndpoint;

type ActionsMap = HashMap<String, EndpointList<ActionEndpoint>>;
type EventsMap = HashMap<String,EndpointList<EventEndpoint>>;

lazy_static! {
    static ref RE: Regex = Regex::new(r"^\$").unwrap();
}
pub(crate)fn get_internal_service_regex_match(text: &str) -> bool {
    RE.is_match(text)
}

#[derive(PartialEq, Eq, Debug)]
pub(crate)struct Logger {}

pub(crate)type ActionHandler = fn(Context, Option<Payload>) -> HandlerResult;
pub(crate)type EventHandler = fn(Context);
#[derive(Default, Debug, PartialEq, Eq, Clone , Serialize)]
pub(crate)struct Payload {}

#[derive(PartialEq, Eq, Clone, Debug)]

pub(crate)struct Action {
    pub(crate)name: String,
    pub(crate) visibility: Visibility,
    pub(crate)handler: ActionHandler,
    // service: Option<Service>,
}

impl Action {
    pub(crate)fn new(name: String, handler: ActionHandler) -> Self {
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

#[derive(PartialEq, Eq, Clone, Debug)]
pub(crate)enum Visibility {
    Published,
    Public,
    Protected,
    Private,
}
#[derive(PartialEq, Eq, Clone, Debug)]
pub(crate)struct Event {
    pub(crate)name: String,
    pub(crate)group : Option<String>,
    pub(crate)handler: EventHandler
}

///Endpoint trait for endpoint list
pub(crate)trait EndpointTrait {
    ///Data is eiter an Action struct or Event structs
    type Data;
    fn new(node_id: &str, service: &ServiceItem, data: Self::Data) -> Self;
    fn node(&self) -> &str;
    fn update(&mut self, data: Self::Data);
    fn is_local(&self) -> bool;
    fn is_available(&self) -> bool;
    fn id(&self) -> &str;
    fn service_name(&self) -> &str;
    fn ep_type(&self) -> EndpointType;
}

#[derive(PartialEq, Eq, Clone, Debug)]
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

pub(crate)enum EndpointType {
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

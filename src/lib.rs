pub(crate) mod broker;
pub(crate) mod broker_delegate;
pub(crate) mod constants;
pub(crate) mod context;
pub(crate) mod errors;
pub(crate) mod ffi;
pub(crate) mod logger;
pub(crate) mod packet;
pub(crate) mod registry;
pub(crate) mod serializers;
pub(crate) mod service;
pub(crate) mod strategies;
pub(crate) mod transporter;
pub(crate) mod utils;
pub(crate) use broker::HandlerResult;
pub(crate) use broker::ServiceBroker;
pub(crate) use broker::ServiceBrokerMessage;
pub(crate) use registry::{DiscovererMessage, Registry};
pub(crate) use service::Service;
pub(crate) use transporter::transit::TransitMessage;

const INTERNAL_PREFIX: char = '$';
type BrokerSender = tokio::sync::mpsc::UnboundedSender<ServiceBrokerMessage>;
type DiscovererSender = tokio::sync::mpsc::UnboundedSender<DiscovererMessage>;
type TransitSender = tokio::sync::mpsc::UnboundedSender<TransitMessage>;
type SharedRegistry = std::sync::Arc<tokio::sync::RwLock<Registry>>;

use lazy_static::lazy_static;
use std::io;
use tokio::runtime::Runtime;
lazy_static! {
    static ref RUNTIME: io::Result<Runtime> = Runtime::new();
}
pub(crate) mod runtimemacro {
    macro_rules! runtime {
        () => {
            match RUNTIME.as_ref() {
                Ok(rt) => rt,
                Err(_) => {
                    return;
                }
            }
        };
    }
    pub(crate) use runtime;
}

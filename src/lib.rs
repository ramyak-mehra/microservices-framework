pub(crate)mod broker;
pub(crate)mod context;
pub(crate)mod errors;
pub(crate)mod logger;
pub(crate)mod packet;
pub(crate)mod registry;
pub(crate)mod service;
pub(crate)mod serializers;
pub(crate)mod strategies;
pub(crate)mod utils;
pub(crate)mod transporter;
pub(crate)mod constants;
pub(crate)mod broker_delegate;
pub(crate)use broker::HandlerResult;
pub(crate)use broker::ServiceBroker;
pub(crate)use broker::ServiceBrokerMessage;
pub(crate)use service::Service;
pub(crate)use registry::{Registry , DiscovererMessage};
pub(crate) use transporter::transit::TransitMessage;


const INTERNAL_PREFIX: char = '$';
type BrokerSender = tokio::sync::mpsc::UnboundedSender<ServiceBrokerMessage>;
type DiscovererSender = tokio::sync::mpsc::UnboundedSender<DiscovererMessage>;
type TransitSender = tokio::sync::mpsc::UnboundedSender<TransitMessage>;
type SharedRegistry = std::sync::Arc<tokio::sync::RwLock<Registry>>;
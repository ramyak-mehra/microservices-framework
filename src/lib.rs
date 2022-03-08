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
pub(crate)use broker::HandlerResult;
pub(crate)use broker::ServiceBroker;
pub(crate)use broker::ServiceBrokerMessage;
pub(crate)use registry::Registry;
pub(crate)use service::Service;

const INTERNAL_PREFIX: char = '$';

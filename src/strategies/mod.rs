use std::sync::Arc;

use crate::registry::{ActionEndpoint, Registry , Opts};
use crate::ServiceBroker;
mod round_robin;


pub(crate)trait Strategy {
    fn new(registry: Arc<Registry>, broker: Arc<ServiceBroker>, opts: StrategyOpts) -> Self;
    fn select<'a>(
        &mut self,
        list: Vec<&'a ActionEndpoint>,
        ctx: Option<Context>,
    ) -> Option<&'a ActionEndpoint>;
}

pub(crate)struct Context {}

pub(crate)struct StrategyOpts{}
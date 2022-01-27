use std::sync::Arc;

use crate::registry::{ActionEndpoint, Broker, Registry , Opts};
mod round_robin;


pub trait Strategy {
    fn new(registry: Arc<Registry>, broker: Arc<Broker>, opts: StrategyOpts) -> Self;
    fn select<'a>(
        &mut self,
        list: Vec<&'a ActionEndpoint>,
        ctx: Option<Context>,
    ) -> Option<&'a ActionEndpoint>;
}

pub struct Context {}

pub struct StrategyOpts{}
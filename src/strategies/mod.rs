use std::sync::Arc;

use crate::registry::{ActionEndpoint, Broker, Registry};
mod round_robin;

trait Strategy {
    fn new(registry: Arc<Registry>, broker: Arc<Broker>, opts: Opts) -> Self;
    fn select<'a>(
        &mut self,
        list: Vec<&'a ActionEndpoint>,
        ctx: Option<Context>,
    ) -> Option<&'a ActionEndpoint>;
}

struct Context {}
struct Opts {}

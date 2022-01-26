use super::*;
use std::sync::Arc;

struct RoundRobinStrategy {
    registry: Arc<Registry>,
    broker: Arc<Broker>,
    opts: Opts,
    counter: usize,
}

impl RoundRobinStrategy {
    // fn new() -> Self {}
}
impl Strategy for RoundRobinStrategy {
    fn new(registry: Arc<Registry>, broker: Arc<Broker>, opts: Opts) -> Self {
        Self {
            broker,
            registry,
            opts,
            counter: 0,
        }
    }
    fn select<'a>(
        &mut self,
        list: Vec<&'a ActionEndpoint>,
        ctx: Option<Context>,
    ) -> Option<&'a ActionEndpoint> {
        if self.counter >= list.len() {
            self.counter = 0;
        }
        self.counter = self.counter.saturating_add(1);
        if let Some(ep) = list.get(self.counter) {
            return Some(*ep);
        }
        None
    }
}

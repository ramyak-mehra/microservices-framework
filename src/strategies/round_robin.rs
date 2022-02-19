use super::*;
use std::sync::Arc;

#[derive(Debug)]
pub struct RoundRobinStrategy {
    counter: usize,
}

impl RoundRobinStrategy {
    pub fn new() -> Self {
        RoundRobinStrategy { counter: 0 }
    }
}
impl Default for RoundRobinStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for RoundRobinStrategy {
    // fn new(registry: Arc<Registry>, broker: Arc<ServiceBroker>, opts: StrategyOpts) -> Self {
    //     Self {
    //         broker,
    //         registry,
    //         opts,
    //         counter: 0,
    //     }
    // }
    fn select<'a, E: EndpointTrait>(
        &mut self,
        list: Vec<&'a E>,
        ctx: Option<&Context>,
    ) -> Option<&'a E> {
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

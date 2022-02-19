mod round_robin;
use crate::context::Context;
use crate::registry::EndpointTrait;
pub use round_robin::RoundRobinStrategy;
pub trait Strategy {
    // fn new(registry: Arc<Registry>, broker: Arc<ServiceBroker>, opts: StrategyOpts) -> Self;
    fn select<'a, E: EndpointTrait>(
        &mut self,
        list: Vec<&'a E>,
        ctx: Option<&Context>,
    ) -> Option<&'a E>;
}

pub struct StrategyOpts {}

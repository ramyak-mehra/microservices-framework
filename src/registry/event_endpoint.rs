// use super::{Broker, Endpoint, Node, Registry, ServiceItem, Event};

// pub struct EventEndpoint {
//     endpoint: Endpoint,
//     event: Event,
// }
// impl EventEndpoint {
//     fn new(
//         registry: <Registry>,
//         broker: Broker,
//         node: Node,
//         service: ServiceItem,
//         event: Event,
//     ) -> Self {
//         let endpoint = Endpoint::new(registry, broker, node, service);

//         Self { event, endpoint }
//     }

//     fn is_available(&self) -> bool {
//         self.endpoint.state
//     }

//     fn update(&mut self, event:  Event) {
//         self.event = event
//     }
// }

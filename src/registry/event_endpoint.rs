use super::*;

#[derive(Clone)]
pub struct EventEndpoint {
    endpoint: Endpoint,
    event: Event,
}

impl EndpointTrait for EventEndpoint {
    type Data = Event;
    fn update(&mut self, data: Self::Data) {
        self.event = data;
    }
    fn new(node: Arc<Node>, service: Arc<ServiceItem>, data: Self::Data) -> Self {
        let endpoint = Endpoint::new(node, service);
        Self {
            endpoint,

            event: data,
        }
    }

    fn node(&self) -> &Node {
        &self.endpoint.node
    }

    fn service(&self) -> &ServiceItem {
        &self.endpoint.service
    }
    fn is_local(&self) -> bool {
        self.endpoint.local
    }
    fn is_available(&self) -> bool {
        self.endpoint.state
    }
    fn id(&self) -> &str {
        &self.endpoint.id
    }
    fn service_name(&self) -> &str {
        &self.endpoint.service.name
    }
}

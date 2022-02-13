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
    fn new(node_id: &str, service: &ServiceItem, data: Self::Data) -> Self {
        let endpoint = Endpoint::new(node_id, service);
        Self {
            endpoint,

            event: data,
        }
    }

    fn node(&self) -> &str {
        &self.endpoint.node_id
    }

    fn is_local(&self) -> bool {
        self.endpoint.local
    }
    fn is_available(&self) -> bool {
        self.endpoint.state
    }
    fn id(&self) -> &str {
        &self.endpoint.node_id
    }
    fn service_name(&self) -> &str {
        &self.endpoint.service
    }
}

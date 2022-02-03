use super::*;

#[derive(PartialEq, Eq, Clone)]
pub struct ActionEndpoint {
    endpoint: Endpoint,
    pub action: Action,
    pub name: String,
}

impl EndpointTrait for ActionEndpoint {
    type Data = Action;
    fn update(&mut self, data: Self::Data) {
        self.action = data;
    }
    fn new(node: Arc<Node>, service: Arc<ServiceItem>, data: Self::Data) -> Self {
        let endpoint = Endpoint::new(node, service);
        let name = format!("{}:{}", endpoint.id, data.name);
        Self {
            endpoint,
            name,
            action: data,
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

use super::*;

#[derive(Clone)]
pub struct ActionEndpoint {
    endpoint: Endpoint,
    action: Action,
    pub name: String,
}
impl ActionEndpoint {
    pub fn new(
        registry: Arc<Registry>,
        broker: Arc<Broker>,
        node: Arc<Node>,
        service: Arc<ServiceItem>,
        action: Action,
    ) -> Self {
        let endpoint = Endpoint::new(registry, broker, node, service);
        let name = format!("{}:{}", endpoint.id, action.name);
        ActionEndpoint {
            action,
            endpoint,
            name,
        }
    }

    pub fn is_available(&self) -> bool {
        self.endpoint.state
    }
    pub fn is_local(&self) -> bool {
        self.endpoint.local
    }
    pub fn update(&mut self, action: Action) {
        self.action = action
    }
    pub fn node(&self) -> &Node {
        &self.endpoint.node
    }

    pub fn service(&self) -> &ServiceItem {
        &self.endpoint.service
    }
    pub fn id(&self) -> &str {
        &self.endpoint.id
    }
}

use super::*;

#[derive(PartialEq, Eq, Clone, Debug)]
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
    fn new(node_id: &str, service: &ServiceItem, data: Self::Data) -> Self {
        let endpoint = Endpoint::new(node_id, service);
        let name = format!("{}:{}", endpoint.node_id, data.name);
        Self {
            endpoint,
            name,
            action: data,
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

    fn ep_type(&self) -> EndpointType {
        EndpointType::Action
    }
}

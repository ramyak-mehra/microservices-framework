use crate::service::ServiceSpec;

use super::*;

#[derive(PartialEq, Eq, Clone)]
pub struct ServiceItem {
    pub name: String,
    pub node: String,
    pub local: bool,
    pub full_name: String,
    version: String,
    actions: ActionsMap,
    /*
    eventsmap
    metadata
    settings
    */
}
impl ServiceItem {
    pub fn new(node: &Node, service: &ServiceSpec, local: bool) -> Self {
        Self {
            node: node.id.clone(),
            local,
            actions: HashMap::new(),
            full_name: service.full_name.to_string(),
            version: service.version.to_string(),
            name: service.name.to_string(),
        }
    }
    pub fn equals(&self, full_name: &str, node_id: Option<&str>) -> bool {
        match node_id {
            Some(id) => self.node == id && self.full_name == full_name,
            None => self.full_name == full_name,
        }
    }

    ///Update service properties
    pub fn update(&mut self, service: &Service) {
        self.full_name = service.full_name.to_string();
        self.version = service.version.to_string();
        /*
        settings
        metadata
        */
        todo!()
    }
    ///Add action to service
    pub fn add_action(&mut self, action: EndpointList<ActionEndpoint>) {
        let name = action.name.clone();
        self.actions.insert(name, action);
        todo!("Decide if we want an arc of action or make a copy of that actions")
    }
    pub fn add_event(&mut self, event: EndpointList<EventEndpoint>) {
        todo!("Implement the events map")
    }
    pub fn unique_name(&self) -> String {
        format!("{}{}{}", self.full_name, self.version, self.node)
    }
}

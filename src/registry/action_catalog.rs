use std::collections::HashMap;

use super::*;

#[derive(PartialEq, Eq)]
pub struct ActionCatalog {
    actions: ActionsMap,
}

impl ActionCatalog {
    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
        }
    }
    fn add(&mut self, node: Arc<Node>, service: Arc<ServiceItem>, action: Action) {
        let list = self.actions.get_mut(&action.name);
        match list {
            Some(list) => list.add(node, service, action),
            None => {
                let name = action.name.clone();
                let mut list = EndpointList::new(name, None);
                let name = action.name.clone();
                list.add(node, service, action);
                self.actions.insert(name, list);
            }
        }
    }
    pub fn get(&self, action_name: &str) -> Option<&EndpointList<ActionEndpoint>> {
        self.actions.get(action_name)
    }
    fn is_available(&self, action_name: &str) -> bool {
        match self.actions.get(action_name) {
            Some(el) => el.has_available(),
            None => false,
        }
    }
    fn remove_by_service(&mut self, service: &ServiceItem) {
        self.actions.iter_mut().for_each(|item| {
            let (key, el) = item;
            el.remove_by_service(service);
        });
    }
    pub fn remove(&mut self, action_name: &str, node_id: &str) {
        let list = self.actions.get_mut(action_name);
        if let Some(el) = list {
            el.remove_by_node_id(node_id);
        }
    }
    fn list(&self) {
        todo!()
    }
}

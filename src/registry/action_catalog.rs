use std::collections::HashMap;

use super::*;

struct ActionCatalog {
    registry: Arc<Registry>,
    broker: Arc<Broker>,
    strategy: Strategy,
    logger: Arc<Logger>,
    actions: HashMap<String, EndpointList>,
}

impl ActionCatalog {
    fn new(registry: Arc<Registry>, broker: Arc<Broker>, strategy: Strategy) -> Self {
        let logger = registry.logger();
        let logger = Arc::clone(logger);

        Self {
            registry,
            strategy,
            broker,
            logger,
            actions: HashMap::new(),
        }
    }
    fn add(&mut self, node: Arc<Node>, service: Arc<ServiceItem>, action: Action) {
        let list = self.actions.get_mut(&action.name);
        match list {
            Some(list) => list.add(node, service, action),
            None => {
                let name = action.name.clone();
                let mut list = EndpointList::new(
                    Arc::clone(&self.registry),
                    Arc::clone(&self.broker),
                    name,
                    None,
                );
                let name = action.name.clone();
                list.add(node, service, action);
                self.actions.insert(name, list);
            }
        }
    }
    fn get(&self, action_name: &str) -> Option<&EndpointList> {
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
    fn remove(&mut self, action_name: &str, node_id: &str) {
        let list = self.actions.get_mut(action_name);
        if let Some(el) = list {
            el.remove_by_node_id(node_id);
        }
    }
    fn list(&self) {
        todo!()
    }
}

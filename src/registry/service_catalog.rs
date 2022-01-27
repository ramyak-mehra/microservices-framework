use super::*;

#[derive(PartialEq, Eq)]
pub struct ServiceCatalog {
    registry: Arc<Registry>,
    broker: Arc<Broker>,
    logger: Arc<Logger>,
    services: Vec<ServiceItem>,
}

impl ServiceCatalog {
    pub fn new(registry: Arc<Registry>, broker: Arc<Broker>) -> Self {
        let logger = &registry.logger;
        let logger = Arc::clone(&logger);
        Self {
            broker,
            registry,
            logger,
            services: Vec::new(),
        }
    }
    ///Add a new service
    pub fn add(&mut self, node: Arc<Node>, service: Arc<Service>, local: bool) {
        let service_item = ServiceItem::new(node, service, local);
        self.services.push(service_item);
    }
    ///Check the service exsists
    pub fn has(&self, full_name: &str, node_id: Option<&str>) -> bool {
        let svc = self
            .services
            .iter()
            .find(|svc| svc.equals(full_name, node_id));
        match svc {
            Some(_) => true,
            None => false,
        }
    }
    pub fn get(&self, full_name: &str, node_id: Option<&str>) -> Option<&ServiceItem> {
        self.services
            .iter()
            .find(|svc| svc.equals(full_name, node_id))
    }
    pub fn get_mut(&mut self, full_name: &str, node_id: Option<&str>) -> Option<&mut ServiceItem> {
        self.services
            .iter_mut()
            .find(|svc| svc.equals(full_name, node_id))
    }
    pub fn list(&self) {
        todo!()
    }
    pub fn get_local_node_service(&self) {
        todo!()
    }
    //remove all endpoints by node_id.
    pub fn remove_all_by_node_id(&mut self, node_id: &str) {
        let services: Vec<&ServiceItem> = self
            .services
            .iter()
            .filter(|svc| {
                if svc.node.id == node_id {
                    todo!("remove actions and events in registry");
                    return false;
                }
                true
            })
            .collect();
        todo!("updat the service")
    }

    pub fn remove(&mut self, full_name: &str, node_id: &str) {
        self.services.retain(|svc| {
            if svc.equals(full_name, Some(node_id)) {
                todo!("remove actions and events in registry");

                return false;
            }
            return true;
        })
    }
}

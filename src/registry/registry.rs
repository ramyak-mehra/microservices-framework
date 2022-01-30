use super::*;
#[derive(PartialEq, Eq)]
pub struct Registry {
    pub logger: Arc<Logger>,
    broker: Arc<ServiceBroker>,
    nodes: NodeCatalog,
    services: ServiceCatalog,
    actions: ActionCatalog,
    /*
    metrics
    strategy factor
    discoverer
    opts
    events
    */
}

impl Registry {
    pub fn new(broker: Arc<ServiceBroker>) -> Self {
        let logger = &broker.logger;
        let logger = Arc::clone(&logger);
        todo!()
    }

    fn init() {
        todo!("initialze discoverer")
    }
    fn stop() {
        todo!("stop discoverre")
    }

    fn register_moleculer_metrics(&self) {
        todo!("register molecular metrics")
    }
    fn update_metrics(&self) {
        todo!("update metrics")
    }
    pub fn register_local_service(&mut self, svc: ServiceItem) {
        todo!("after service has been done")
    }
    pub fn register_services() {
        todo!("add remote serice support")
    }
    fn check_action_visibility(action: &Action, node: &Arc<Node>) -> bool {
        match action.visibility {
            Visibility::Published => true,
            Visibility::Public => true,
            Visibility::Protected => node.local,
            _ => false,
        }
    }
    fn register_actions() {
        todo!()
    }
    fn create_private_action_endpoint(action: Action) {
        todo!()
    }
    pub fn has_services(&self, full_name: &str, node_id: Option<&str>) -> bool {
        self.services.has(full_name, node_id)
    }
    pub fn get_action_endpoints(&self, action_name: &str) -> Option<&EndpointList<ActionEndpoint>> {
        self.actions.get(action_name)
    }
    pub fn get_action_endpoint_by_node_id(
        &self,
        action_name: &str,
        node_id: &str,
    ) -> Option<&EndpointList<ActionEndpoint>> {
        let list = self.actions.get(action_name);
        if let Some(list) = list {
            list.get_endpoint_by_node_id(node_id);
        }
        None
    }
    pub fn unregister_service(&mut self, full_name: &str, node_id: Option<&str>) {
        let id = match node_id {
            Some(node_id) => node_id.to_string(),
            None => self.broker.node_id.clone(),
        };
        self.services.remove(full_name, &id);
        match node_id {
            Some(id) => {
                if id == self.broker.node_id {
                    self.regenerate_local_raw_info(true);
                }
            }
            None => self.regenerate_local_raw_info(true),
        }
    }
    fn unregister_service_by_node_id(&mut self, node_id: &str) {
        self.services.remove_all_by_node_id(node_id);
    }
    fn unregiste_action(&mut self, node_id: &str, action_name: &str) {
        self.actions.remove(action_name, node_id);
    }

    fn register_events() {
        todo!()
    }
    fn unregister_event(&mut self, node_id: &str, event_name: &str) {
        todo!()
    }

    fn regenerate_local_raw_info(&self, incSeq: bool) {
        todo!()
    }

    fn get_local_node_info(&self, force: bool) {
        todo!()
    }
    fn get_node_info(&self, node_id: &str) -> Option<Node> {
        todo!()
    }
    fn process_node_info(&self) {
        todo!()
    }
    fn get_node_list(&self, only_available: bool, with_services: bool) -> Vec<&Node> {
        self.nodes.list(only_available, with_services)
    }
    fn get_services_list(&self) -> Vec<&ServiceItem> {
        todo!()
    }
    fn get_actions_list(&self) -> Vec<&ActionEndpoint> {
        todo!()
    }
    fn get_event_list(&self) -> Vec<&EventEndpoint> {
        todo!()
    }
    fn get_node_raw_list(&self) {
        todo!()
    }
}

use crate::{
    errors::RegistryError, packet::PayloadInfo, service::ServiceSpec, ServiceBroker,
    ServiceBrokerMessage,
};

use super::*;
use anyhow::bail;
use serde_json::Value;
use tokio::sync::mpsc::UnboundedSender;
#[derive(Debug)]

pub struct Registry {
    broker_sender: UnboundedSender<ServiceBrokerMessage>,
    broker: Arc<ServiceBroker>,
    pub(crate) nodes: NodeCatalog,
    pub(crate) services: ServiceCatalog,
    pub(crate) actions: ActionCatalog,
    pub(crate) events: EventCatalog,
    /*
    metrics
    strategy factor
    discoverer
    opts
    events
    */
}
impl Registry {
    pub fn new(
        broker: Arc<ServiceBroker>,
        broker_sender: UnboundedSender<ServiceBrokerMessage>,
    ) -> Self {
        //TODO:get the library version for the client

        let mut nodes = NodeCatalog::new();
        nodes.create_local_node(
            "1.0".to_string(),
            broker.node_id.clone(),
            broker.instance.clone(),
            broker.options.metadata.clone(),
        );
        let services = ServiceCatalog::default();
        let actions = ActionCatalog::default();
        let events = EventCatalog::new(broker.clone());
        Registry {
            broker_sender,
            broker,
            nodes,
            services,
            actions,
            events,
        }
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
    pub fn register_local_service(&mut self, svc: ServiceSpec) -> anyhow::Result<()> {
        if !self
            .services
            .has(&svc.full_name, Some(&self.broker.node_id))
        {
            let node = self.nodes.local_node()?.clone();

            let service_full_name = self.services.add(&node, &svc, true);
            if let Some(actions) = svc.actions {
                self.register_actions(&node, &service_full_name, actions)?;
            }
            if let Some(events) = svc.events {
                self.register_events(&node, &service_full_name, events)?;
            }

            {
                let local_node = self.nodes.local_node_mut()?;

                local_node.services.push(service_full_name);
            }
        }
        Ok(())
    }
    pub fn register_services() {
        todo!("add remote serice support")
    }
    fn check_action_visibility(action: &Action, node: &Node) -> bool {
        match action.visibility {
            Visibility::Published => true,
            Visibility::Public => true,
            Visibility::Protected => node.local,
            _ => false,
        }
    }
    fn register_actions(
        &mut self,
        node: &Node,
        service_full_name: &str,
        actions: Vec<Action>,
    ) -> anyhow::Result<()> {
        let service = self.services.get_mut(service_full_name, Some(&node.id))?;

        actions.iter().for_each(|action| {
            if !Registry::check_action_visibility(action, node) {
                return;
            }
            if node.local {
                //TODO:stuff with middleware and handlers.
            } else if self.broker.transit.is_some() {
                //TODO: for remote services
                return;
            }
            self.actions.add(node, service, action.to_owned());
            //TODO:
            // service.add_action(action)
            //add the action to the service.
        });
        Ok(())
    }
    fn create_private_action_endpoint(&self, action: Action) -> anyhow::Result<ActionEndpoint> {
        let node = self.nodes.local_node()?;

        todo!("add service to action")
        // let action_ep = ActionEndpoint::new(node, service, action);
        // Ok(action_ep)
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
    ) -> Option<&ActionEndpoint> {
        let list = self.actions.get(action_name);
        if let Some(list) = list {
            return list.get_endpoint_by_node_id(node_id);
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
                    self.regenerate_local_raw_info(Some(true));
                }
            }
            None => {
                self.regenerate_local_raw_info(Some(true));
            }
        }
    }
    pub fn unregister_service_by_node_id(&mut self, node_id: &str) {
        self.services.remove_all_by_node_id(node_id);
    }
    fn unregiste_action(&mut self, node_id: &str, action_name: &str) {
        self.actions.remove(action_name, node_id);
    }

    fn register_events(
        &mut self,
        node: &Node,
        service_full_name: &str,
        events: Vec<Event>,
    ) -> anyhow::Result<()> {
        let service = self.services.get_mut(service_full_name, Some(&node.id))?;
        events.iter().for_each(|event| {
            self.events.add(node, service, event.to_owned());
        });

        Ok(())
    }
    fn unregister_event(&mut self, node_id: &str, event_name: &str) {
        self.events.remove(event_name, node_id);
    }

    fn regenerate_local_raw_info(&self, inc_seq: Option<bool>) -> NodeRawInfo {
        todo!()
    }

    pub fn get_local_node_info(&self, force: bool) -> anyhow::Result<NodeRawInfo> {
        // if let None = self.nodes.local_node() {
        //     return Ok(self.regenerate_local_raw_info(None));
        // }
        if force {
            return Ok(self.regenerate_local_raw_info(None));
        }

        Ok(self.nodes.local_node()?.raw_info().to_owned())
    }
    fn get_node_info(&self, node_id: &str) -> Option<Node> {
        todo!()
    }
    pub(crate) fn process_node_info(&self, node_id: &str, payload: PayloadInfo) {
        todo!()
    }
    pub fn get_node_list(&self, only_available: bool, with_services: bool) -> Vec<&Node> {
        self.nodes.list(only_available, with_services)
    }
    pub fn get_services_list(&self, opts: ListOptions) -> Vec<&ServiceItem> {
        self.services.list(opts)
    }
    fn get_action_list(&self, opts: ListOptions) -> Vec<&ActionEndpoint> {
        //self.actions.list(opts)
        todo!()
    }
    fn get_event_list(&self) -> Vec<&EventEndpoint> {
        todo!()
    }
    fn get_node_raw_list(&self) {
        todo!()
    }
}
#[cfg(test)]

mod tests {}

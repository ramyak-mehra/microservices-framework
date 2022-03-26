use std::time::Duration;

use crate::{
    broker::BrokerOptions, broker_delegate::BrokerDelegate, errors::RegistryError,
    packet::PayloadInfo, registry::discoverers::NodeEvents, service::ServiceSpec, utils,
    BrokerSender, DiscovererSender, ServiceBroker, ServiceBrokerMessage, SharedRegistry,
    TransitSender,
};

use super::*;
use log::{debug, info, warn};
use serde_json::{json, Value};
use tokio::sync::{mpsc::UnboundedSender, RwLock};
#[derive(Debug)]

pub(crate) struct Registry {
    broker_sender: UnboundedSender<ServiceBrokerMessage>,

    pub(crate) nodes: NodeCatalog,
    pub(crate) services: ServiceCatalog,
    pub(crate) actions: ActionCatalog,
    pub(crate) events: EventCatalog,
    node_id: String,
    discoverer_sender: Option<DiscovererSender>, /*
                                                 metrics
                                                 opts
                                                 */
}
impl Registry {
    pub(crate) fn new(
        broker_sender: UnboundedSender<ServiceBrokerMessage>,
        node_id: String,
        instance_id: String,
        broker_options: Arc<BrokerOptions>,
    ) -> Self {
        //TODO:get the library version for the client

        let mut nodes = NodeCatalog::new();
        nodes.create_local_node(
            "1.0".to_string(),
            node_id.clone(),
            instance_id,
            broker_options.metadata.clone(),
        );
        let services = ServiceCatalog::default();
        let actions = ActionCatalog::default();
        let events = EventCatalog::new(broker_options);

        Self {
            broker_sender,
            nodes,
            services,
            actions,
            events,
            node_id,
            discoverer_sender: None,
        }
    }

    async fn init(
        &mut self,
        transit: TransitSender,
        registry: SharedRegistry,
        opts: DiscovererOpts,
        broker_sender: BrokerSender,
        broker: Arc<BrokerDelegate>,
    ) {
        let mut discoverer = LocalDiscoverer::new(
            transit,
            registry,
            opts,
            broker_sender,
            broker,
            self.node_id.clone(),
        );
        let discoverer_sender = discoverer.discoverer_sender();
        self.discoverer_sender = Some(discoverer_sender);
        discoverer.init().await;
        tokio::spawn(async move {
            discoverer.message_handler().await;
        });
    }
    async fn stop(&mut self) {
        self.discoverer_sender
            .as_ref()
            .unwrap()
            .send(DiscovererMessage::StopHeartbeatTimer);
    }

    fn register_moleculer_metrics(&self) {
        todo!("register molecular metrics")
    }
    fn update_metrics(&self) {
        todo!("update metrics")
    }
    pub(crate) fn register_local_service(&mut self, svc: ServiceSpec) -> anyhow::Result<()> {
        if !self.services.has(&svc.full_name, Some(&self.node_id)) {
            let node = self.nodes.local_node()?.clone();
            // Due to rust ownership rules we can't have mutable or immutable reference here.
            // Find a better way to handle refetching the services
            let service_pos = self.services.add(&node, &svc, true);

            if let Some(actions) = svc.actions {
                self.register_actions(&node, &service_pos, actions)?;
            }
            if let Some(events) = svc.events {
                self.register_events(&node, &service_pos, events)?;
            }

            {
                let local_node = self.nodes.local_node_mut()?;
                let service = self.services.get_at(service_pos)?;
                local_node.services.push(service.into());
            }
            //TODO: this.broker.started instead of true
            self.regenerate_local_raw_info(Some(true));
            info!("{} service is registered." , svc.name);
        }
        Ok(())
    }
    pub(crate) fn register_services(&mut self) {
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
        service_pos: &usize,
        actions: Vec<Action>,
    ) -> anyhow::Result<()> {
        let service = self.services.get_at_mut(*service_pos)?;

        actions.iter().for_each(|action| {
            if !Registry::check_action_visibility(action, node) {
                return;
            }
            if node.local {
                //TODO:stuff with middleware and handlers.
                //Access is broker_sender is present through channels
                // } else if self.broker_sender.transit.is_some() {
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
    pub(crate) fn has_services(&self, full_name: &str, node_id: Option<&str>) -> bool {
        self.services.has(full_name, node_id)
    }
    pub(crate) fn get_action_endpoints(
        &self,
        action_name: &str,
    ) -> Option<&EndpointList<ActionEndpoint>> {
        self.actions.get(action_name)
    }
    pub(crate) fn get_action_endpoint_by_node_id(
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
    pub(crate) fn unregister_service(&mut self, full_name: &str, node_id: Option<&str>) {
        let id = match node_id {
            Some(node_id) => node_id.to_string(),
            None => self.node_id.clone(),
        };
        self.services.remove(full_name, &id);
        match node_id {
            Some(id) => {
                if id == self.node_id {
                    self.regenerate_local_raw_info(Some(true));
                }
            }
            None => {
                self.regenerate_local_raw_info(Some(true));
            }
        }
    }
    pub(crate) fn unregister_service_by_node_id(&mut self, node_id: &str) {
        self.services.remove_all_by_node_id(node_id);
    }
    fn unregiste_action(&mut self, node_id: &str, action_name: &str) {
        self.actions.remove(action_name, node_id);
    }

    fn register_events(
        &mut self,
        node: &Node,
        service_pos: &usize,
        events: Vec<Event>,
    ) -> anyhow::Result<()> {
        let service = self.services.get_at(*service_pos)?;
        events.iter().for_each(|event| {
            self.events.add(node, service, event.to_owned());
        });

        Ok(())
    }
    fn unregister_event(&mut self, node_id: &str, event_name: &str) {
        self.events.remove(event_name, node_id);
    }

   pub(crate) fn regenerate_local_raw_info(&mut self, inc_seq: Option<bool>) -> anyhow::Result<NodeRawInfo> {
        let mut node = self.nodes.local_node_mut()?;
        if let Some(inc_seq) = inc_seq {
            if inc_seq {
                node.seq += 1;
            }
        }
        let raw_info = NodeRawInfo::from_node(node);
        node.set_raw_info(raw_info.clone());
        Ok(raw_info)
    }

    pub(crate) fn get_local_node_info(&mut self, force: bool) -> anyhow::Result<NodeRawInfo> {
        // if let None = self.nodes.local_node() {
        //     return Ok(self.regenerate_local_raw_info(None));
        // }
        if force {
            return self.regenerate_local_raw_info(None);
        }

        Ok(self.nodes.local_node()?.raw_info().to_owned())
    }
    fn get_node_info(&mut self, node_id: &str) -> Option<NodeRawInfo> {
        match self.nodes.get_node(node_id) {
            Some(node) => {
                if node.local {
                    return Some(self.get_local_node_info(false).unwrap());
                }
                return Some(node.raw_info().to_owned());
            }
            None => None,
        }
    }
    ///Process an incoming node INFO packet.
    pub(crate) async fn process_node_info(&mut self, node_id: &str, payload: PayloadInfo) {
        let (is_new, register_services, is_reconnected) =
            self.nodes.process_node_info(payload).await;
        let node = self.nodes.get_node(node_id).unwrap();
        if register_services {
            //TODO: register services
            //    { self.register_services();}
        }
        if is_new {
            let data = serde_json::json!({"node": node , "reconnected":false});
            let _ = self
                .broker_sender
                .send(ServiceBrokerMessage::BroadcastLocal {
                    event_name: NodeEvents::Connected.to_string(),
                    data: data,
                    opts: Value::Null,
                });
            info!("Node {} connected", node_id)
        } else if is_reconnected {
            let data = serde_json::json!({"node": node , "reconnected":true});
            let _ = self
                .broker_sender
                .send(ServiceBrokerMessage::BroadcastLocal {
                    event_name: NodeEvents::Connected.to_string(),
                    data: data,
                    opts: Value::Null,
                });
            info!("Node {} reconnected", node_id)
        } else {
            let data = serde_json::json!({ "node": node });
            let _ = self
                .broker_sender
                .send(ServiceBrokerMessage::BroadcastLocal {
                    event_name: NodeEvents::Updated.to_string(),
                    data: data,
                    opts: Value::Null,
                });
            debug!("Node {} updated", node_id)
        }
    }
    pub(crate) fn get_node_list(&self, only_available: bool, with_services: bool) -> Vec<&Node> {
        self.nodes.list(only_available, with_services)
    }
    pub(crate) fn get_services_list(&self, opts: ListOptions) -> Vec<&ServiceItem> {
        self.services.list(opts)
    }
    pub(crate) fn set_last_heartbeat_time(&mut self, node_id: &str, time: Duration) {
        let node = self.nodes.get_node_mut(node_id).unwrap();
        node.last_heartbeat_time = Some(time);
    }
    fn get_action_list(&self, opts: ListOptions) -> Vec<&ActionEndpoint> {
        //self.actions.list(opts)
        todo!()
    }
    pub(crate) async fn check_remote_nodes(&mut self, heartbeat_timeout: Duration) {
        let now = utils::process_uptime();
        let mut to_be_disconected = Vec::new();
        for node in self.nodes.nodes.values_mut() {
            if node.local || !node.available {
                continue;
            }
            match node.last_heartbeat_time {
                Some(last) => {
                    if (now - last) > heartbeat_timeout {
                        to_be_disconected.push(node.id.clone());
                    }
                }
                None => {
                    node.last_heartbeat_time = Some(now.clone());
                    continue;
                }
            }
        }
        to_be_disconected.iter().for_each(|id| {
            warn!("Hearbeat is not received from {} node.", id);
            // Node cannot be null as we have sorted them just before disconnecting them.
            let node = self.nodes.disconnected(id, true);

            let data = json!({
                "node":node.unwrap(),
                "unexpected":true
            });
            let _ = self
                .broker_sender
                .send(ServiceBrokerMessage::BroadcastLocal {
                    event_name: NodeEvents::Disconnected.to_string(),
                    data,
                    opts: Value::Null,
                });
        });
    }

    pub(crate) async fn check_offline_nodes(&mut self, clean_offline_nodes_timeout: Duration) {
        let now = utils::process_uptime();
        let mut to_be_removed = Vec::new();
        for node in self.nodes.nodes.values_mut() {
            if node.local || node.available {
                continue;
            }
            match node.last_heartbeat_time {
                Some(last) => {
                    if (now - last) > clean_offline_nodes_timeout {
                        to_be_removed.push(node.id.clone());
                    }
                }
                None => {
                    // Not recieved the first
                    node.last_heartbeat_time = Some(now.clone());
                    continue;
                }
            }
        }
        to_be_removed.iter().for_each(|id| {
            warn!("Removing offline {} node from registry because it hasn't submitted heartbeat signal for 10 minutes.", id);
            self.nodes.delete(id);

        });
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

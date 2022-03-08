use std::future::Future;

use crate::{context::EventType, utils, ServiceBroker};

use super::*;

#[derive(Debug, Clone)]
pub struct EventCatalog {
    events: Vec<EndpointList<EventEndpoint>>,
    broker: Arc<ServiceBroker>,
}
impl EventCatalog {
    pub fn new(broker: Arc<ServiceBroker>) -> Self {
        Self {
            events: Vec::new(),
            broker,
        }
    }

    pub fn add(&mut self, node: &Node, service: &ServiceItem, event: Event) {
        let event_name = event.name.clone();
        let group_name = match &event.group {
            Some(group_name) => group_name.clone(),
            None => service.name.to_owned(),
        };
        let list = self.get_mut(&event_name, &group_name);
        match list {
            Some(list) => {
                list.add(node, service, event);
            }
            None => {
                let mut list = EndpointList::new(event_name, Some(group_name));

                list.add(node, service, event);
                self.events.push(list);
            }
        }
    }

    fn get(&self, event_name: &str, group_name: &str) -> Option<&EndpointList<EventEndpoint>> {
        self.events
            .iter()
            .find(|list| list.name == event_name && list.group.as_ref().unwrap() == group_name)
    }

    fn get_mut(
        &mut self,
        event_name: &str,
        group_name: &str,
    ) -> Option<&mut EndpointList<EventEndpoint>> {
        self.events
            .iter_mut()
            .find(|list| list.name == event_name && list.group.as_ref().unwrap() == group_name)
    }
    fn get_balanced_endpoints(
        &self,
        event_name: &str,
        group_names: &Option<Vec<String>>,
    ) -> Vec<(&EventEndpoint, &Option<String>)> {
        let mut res = Vec::new();
        self.events.iter().for_each(|list| {
            if !utils::match_str(event_name, &list.name) {
                return;
            }
            let mut execute = || {
                let ep = list.next_local(None, &self.broker.options.strategy_factory);
                if let Some(ep) = ep {
                    if ep.is_available() {
                        res.push((ep, &list.group));
                    }
                }
                drop(ep);
            };
            match &group_names {
                Some(group_names) => {
                    if group_names.is_empty() || find_group(&group_names, &list.group) {
                        execute();
                    }
                }
                None => execute(),
            }
        });
        res
    }
 pub   fn get_groups(&self, event_name: &str) -> Vec<String> {
        let groups: Option<Vec<String>> = self
            .events
            .iter()
            .filter(|list| utils::match_str(event_name, &list.name))
            .map(|item| item.group.to_owned())
            .filter(|item| item.is_some())
            .collect();
        match groups {
            Some(mut groups) => {
                // TODO: remove duplicates

                groups
            }
            None => Vec::with_capacity(0),
        }
    }

  pub  fn get_all_endpoints(
        &self,
        event_name: &str,
        group_names: Option<&Vec<String>>,
    ) -> Vec<&EventEndpoint> {
        let mut res = Vec::new();
        self.events.iter().for_each(|list| {
            if !utils::match_str(event_name, &list.name) {
                return;
            }
            let mut execute = || {
                let mut ep: Vec<&EventEndpoint> = list
                    .endpoints
                    .iter()
                    .filter(|ep| ep.is_available())
                    .collect();
                res.append(&mut ep);
                drop(ep);
            };
            match &group_names {
                Some(group_names) => {
                    if group_names.is_empty() || find_group(&group_names, &list.group) {
                        execute();
                    }
                }
                None => execute(),
            }
        });
        //TODO:remove duplicates
        res
    }
    pub async fn emit_local_services(&self, ctx: Context) {
        let sender = ctx.node_id();
        let event_name = ctx.event_name.as_ref().unwrap();
        let mut futures = Vec::new();
        self.events.iter().for_each(|list| {
            if !utils::match_str(&event_name, &list.name) {
                return;
            }
            let mut execute = || match ctx.event_type {
                EventType::Broadcast | EventType::BroadcastLocal => {
                    list.endpoints.iter().for_each(|ep| {
                        if ep.is_local() {
                            let mut new_ctx = ctx.clone();
                            new_ctx.node_id = Some(sender.to_string());
                            futures
                                .push(EventCatalog::call_event_handler(new_ctx, ep.event.clone()));
                        }
                    });
                }
                EventType::Emit => {
                    let ep = list.next_local(Some(&ctx), &self.broker.options.strategy_factory);
                    if let Some(ep) = ep {
                        let mut new_ctx = ctx.clone();
                        new_ctx.node_id = Some(sender.to_string());
                        futures.push(EventCatalog::call_event_handler(new_ctx, ep.event.clone()))
                    }
                }
            };
            match &ctx.event_groups {
                Some(event_groups) => {
                    if event_groups.is_empty() || find_group(event_groups, &list.group) {
                        execute();
                    }
                }
                None => execute(),
            }
        });
        //TODO: use better way to execute them all.
        for future in futures {
            future.await;
        }
    }

    async fn call_event_handler(ctx: Context, event: Event) {
        tokio::spawn(async move { (event.handler)(ctx) });
    }
    fn remove_by_service(&mut self, service: &ServiceItem) {
        self.events
            .iter_mut()
            .for_each(|list| list.remove_by_service(service));
    }
    pub fn remove(&mut self, event_name: &str, node_id: &str) {
        self.events.iter_mut().for_each(|list| {
            if list.name == event_name {
                list.remove_by_node_id(node_id);
            }
        });
    }
}
fn find_group(event_groups: &Vec<String>, group: &Option<String>) -> bool {
    match group {
        Some(group) => {
            let found = event_groups.iter().find(|event_group| {
                return *event_group == group;
            });
            found.is_some()
        }
        None => false,
    }
}

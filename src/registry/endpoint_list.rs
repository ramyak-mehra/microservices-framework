use tokio::sync::RwLock;

use crate::INTERNAL_PREFIX;

use super::*;
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct EndpointList<E: EndpointTrait + Clone> {
    pub name: String,
    group: Option<String>,
    internal: bool,
    pub endpoints: Vec<E>,
    local_endpoints: Vec<E>,
}

impl<E: EndpointTrait + Clone> EndpointList<E> {
    pub fn new(name: String, group: Option<String>) -> Self {
        let internal = name.starts_with(INTERNAL_PREFIX);
        let endpoints = Vec::new();
        let local_endpoints = Vec::new();

        Self {
            name,
            group,
            endpoints,
            local_endpoints,
            internal,
        }
    }

    pub fn add(&mut self, node: &Node, service: &ServiceItem, data: E::Data) {
        let entry = self
            .endpoints
            .iter_mut()
            .find(|x| x.service_name() == service.unique_name());
        if let Some(entry) = entry {
            entry.update(data);
            return;
        }

        let ep = E::new(&node.id, service, data);

        self.endpoints.push(ep.clone());
        if ep.is_local() {
            self.local_endpoints.push(ep)
        }
    }
    fn get_first(&self) -> Option<&E> {
        self.endpoints.get(0)
    }

    fn select<'a, S: Strategy>(
        &self,
        list: Vec<&'a E>,
        ctx: &Context,
        strategy: &RwLock<S>,
    ) -> Option<&'a E> {
        let mut strategy = strategy.blocking_write();
        let ret = strategy.select(list, Some(ctx));
        ret
    }

    pub fn next<S: Strategy>(&self, ctx: &Context, strategy: &RwLock<S>) -> Option<&E> {
        if self.endpoints.is_empty() {
            return None;
        }

        if self.internal && self.has_local() {
            return self.next_local(ctx, strategy);
        }

        if self.endpoints.len() == 1 {
            let ep = self.endpoints.get(0);
            if let Some(ep) = ep {
                ep.is_available();
                return Some(ep);
            }
            return None;
        }

        //TODO: Search local item based on registr opts
        let ep_list: Vec<&E> = self
            .endpoints
            .iter()
            .filter(|ep| ep.is_available())
            .collect();
        if ep_list.is_empty() {
            return None;
        }

        self.select(ep_list, ctx, strategy)
    }
    pub fn next_local<S: Strategy>(&self, ctx: &Context, strategy: &RwLock<S>) -> Option<&E> {
        if self.local_endpoints.is_empty() {
            return None;
        }

        if self.local_endpoints.len() == 1 {
            let ep = self.local_endpoints.get(0);
            if let Some(ep) = ep {
                ep.is_available();
                return Some(ep);
            }
            return None;
        }

        //TODO: Search local item based on registr opts
        let ep_list: Vec<&E> = self
            .endpoints
            .iter()
            .filter(|ep| ep.is_available())
            .collect();
        if ep_list.is_empty() {
            return None;
        }

        self.select(ep_list, ctx, strategy)
    }

    pub fn has_available(&self) -> bool {
        for ep in self.endpoints.iter() {
            if ep.is_available() {
                return true;
            }
        }
        false
    }
    pub fn has_local(&self) -> bool {
        !self.local_endpoints.is_empty()
    }

    fn update_local_endpoints(&mut self) {
        let mut local: Vec<E> = Vec::new();
        for ep in &self.endpoints {
            if ep.is_local() {
                let e = ep.clone();
                local.push(e);
            }
        }
        std::mem::swap(&mut local, &mut self.local_endpoints);
        drop(local);
    }

    pub fn count(&self) -> usize {
        self.endpoints.len()
    }
    pub fn get_endpoint_by_node_id(&self, node_id: &str) -> Option<&E> {
        self.endpoints
            .iter()
            .find(|e| e.id() == node_id && e.is_available())
    }
    fn has_node_id(&self, node_id: &str) -> bool {
        self.endpoints.iter().any(|e| e.id() == node_id)
    }
    pub fn remove_by_service(&mut self, service: &ServiceItem) {
        self.endpoints.retain(|ep| {
            let delete = ep.service_name() == service.unique_name();
            !delete
        });
        self.update_local_endpoints();
    }

    pub fn remove_by_node_id(&mut self, node_id: &str) {
        self.endpoints.retain(|ep| {
            let delete = ep.id() == node_id;
            !delete
        });
        self.update_local_endpoints();
    }
}

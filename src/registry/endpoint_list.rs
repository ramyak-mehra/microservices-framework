use std::sync::Arc;

use super::*;
#[derive(PartialEq, Eq, Clone)]
pub struct EndpointList<T: EndpointTrait + Clone> {
    pub name: String,
    group: Option<String>,
    internal: bool,
    pub endpoints: Vec<T>,
    local_endpoints: Vec<T>,
}

impl<T: EndpointTrait + Clone> EndpointList<T> {
    pub fn new(name: String, group: Option<String>) -> Self {
        let internal = name.starts_with("$");
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

    pub fn add(&mut self, node: &Node, service:&ServiceItem, data: T::Data) {
        let entry = self
            .endpoints
            .iter_mut()
            .find(|x| x.service_name() == service.unique_name());

        match entry {
            Some(found) => {
                found.update(data);
                return;
            }
            None => {}
        }

        let ep = T::new(&node.id, service, data);

        self.endpoints.push(ep.clone());
        if ep.is_local() {
            self.local_endpoints.push(ep)
        }
    }
    fn get_first(&self) -> Option<&T> {
        self.endpoints.get(0)
    }

    fn select(&self) -> &T {
        todo!()
    }

    fn next(&self) -> &T {
        todo!()
    }
    fn next_local(&self) -> &T {
        todo!()
    }

    pub fn has_available(&self) -> bool {
        for ep in self.endpoints.iter() {
            if ep.is_available() {
                return true;
            }
        }
        return false;
    }
    pub fn has_local(&self) -> bool {
        self.local_endpoints.len() > 0
    }

    fn update_local_endpoints(&mut self) {
        let mut local: Vec<T> = Vec::new();
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
    pub fn get_endpoint_by_node_id(&self, node_id: &str) -> Option<&T> {
        self.endpoints
            .iter()
            .find(|e| e.id() == node_id && e.is_available())
    }
    fn has_node_id(&self, node_id: &str) -> bool {
        match self.endpoints.iter().find(|e| e.id() == node_id) {
            Some(_) => true,
            None => false,
        }
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

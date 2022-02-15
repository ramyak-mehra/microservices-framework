use anyhow::bail;

use crate::service::ServiceSpec;

use super::*;
#[derive(PartialEq, Eq)]
pub struct ServiceCatalog {
    services: Vec<ServiceItem>,
}

impl ServiceCatalog {
    pub fn new() -> Self {
        Self {
            services: Vec::new(),
        }
    }
    ///Add a new service
    pub fn add(&mut self, node: &Node, service: &ServiceSpec, local: bool) -> String {
        let service_item = ServiceItem::new(node, service, local);
        let full_name = service_item.full_name.clone();
        self.services.push(service_item);
        full_name
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
    pub fn get_mut(
        &mut self,
        full_name: &str,
        node_id: Option<&str>,
    ) -> anyhow::Result<&mut ServiceItem> {
        let result = self
            .services
            .iter_mut()
            .find(|svc| svc.equals(full_name, node_id));
        match result {
            Some(service_item) => Ok(service_item),
            None => bail!(RegistryError::NoServiceItemFound),
        }
    }
    pub fn list(&self, opts: ListOptions) -> Vec<&ServiceItem> {
        self.services
            .iter()
            .filter(|svc| {
                if opts.skip_internal && get_internal_service_regex_match(&svc.name) {
                    return false;
                }
                if opts.only_local && !svc.local {
                    return false;
                }
                //TODO: find a way to get node available
                // if opts.only_available && !svc.node.available {
                //     return false;
                // }

                return true;
            })
            .collect()
        // TODO:("implement grouping and all that stuff")
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
                if svc.node == node_id {
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

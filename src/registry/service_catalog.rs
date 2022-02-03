use crate::service::ServiceSpec;

use super::*;
use regex::Regex;
#[derive(PartialEq, Eq)]
pub(crate)struct ServiceCatalog {
    services: Vec<Arc<ServiceItem>>,
}

impl ServiceCatalog {
    pub(crate)fn new() -> Self {
        Self {
            services: Vec::new(),
        }
    }
    ///Add a new service
    pub(crate)fn add(&mut self, node: Arc<Node>, service: &ServiceSpec, local: bool) -> Arc<ServiceItem> {
        let service_item = ServiceItem::new(node, service, local);
        let service_item = Arc::new(service_item);

        let item = Arc::clone(&service_item);
        self.services.push(service_item);
        item
    }
    ///Check the service exsists
    pub(crate)fn has(&self, full_name: &str, node_id: Option<&str>) -> bool {
        let svc = self
            .services
            .iter()
            .find(|svc| svc.equals(full_name, node_id));
        match svc {
            Some(_) => true,
            None => false,
        }
    }
    pub(crate)fn get(&self, full_name: &str, node_id: Option<&str>) -> Option<&Arc<ServiceItem>> {
        self.services
            .iter()
            .find(|svc| svc.equals(full_name, node_id))
    }
    pub(crate)fn get_mut(
        &mut self,
        full_name: &str,
        node_id: Option<&str>,
    ) -> Option<&mut Arc<ServiceItem>> {
        self.services
            .iter_mut()
            .find(|svc| svc.equals(full_name, node_id))
    }
    pub(crate)fn list(
        &self,
       opts : ListOptions 
    ) -> Vec<&Arc<ServiceItem>> {
       
        self.services.iter().filter(|svc| {
            if opts.skip_internal && get_internal_service_regex_match(&svc.name) {
                return false;
            }
            if opts.only_local && !svc.local {
                return false;
            }
            if opts.only_available && !svc.node.available {
                return false;
            }

            return true;
        }).collect()
        // TODO:("implement grouping and all that stuff")

    }
    pub(crate)fn get_local_node_service(&self) {
        todo!()
    }
    //remove all endpoints by node_id.
    pub(crate)fn remove_all_by_node_id(&mut self, node_id: &str) {
        let services: Vec<&Arc<ServiceItem>> = self
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

    pub(crate)fn remove(&mut self, full_name: &str, node_id: &str) {
        self.services.retain(|svc| {
            if svc.equals(full_name, Some(node_id)) {
                todo!("remove actions and events in registry");

                return false;
            }
            return true;
        })
    }
}

use anyhow::bail;

use crate::{errors::RegistryError, service::ServiceSpec};

use super::*;
#[derive(PartialEq, Eq, Default, Debug)]
pub(crate) struct ServiceCatalog {
    services: Vec<ServiceItem>,
}

impl ServiceCatalog {
    ///Add a new service and return the position in the list.
    pub(crate) fn add(&mut self, node: &Node, service: &ServiceSpec, local: bool) -> usize {
        let service_item = ServiceItem::new(node, service, local);
        let full_name = service_item.full_name.clone();
        self.services.push(service_item);
        self.get_pos(&full_name, Some(&node.id)).unwrap()
    }
    ///Check the service exsists
    pub(crate) fn has(&self, full_name: &str, node_id: Option<&str>) -> bool {
        let svc = self
            .services
            .iter()
            .find(|svc| svc.equals(full_name, node_id));
        svc.is_some()
    }
    pub(crate) fn get(
        &self,
        full_name: &str,
        node_id: Option<&str>,
    ) -> anyhow::Result<&ServiceItem> {
        let result = self
            .services
            .iter()
            .find(|svc| svc.equals(full_name, node_id));
        match result {
            Some(service_item) => Ok(service_item),
            None => bail!(RegistryError::NoServiceItemFound),
        }
    }
    pub(crate) fn get_pos(&self, full_name: &str, node_id: Option<&str>) -> anyhow::Result<usize> {
        let result = self
            .services
            .iter()
            .position(|svc| svc.equals(full_name, node_id));
        match result {
            Some(service_item) => Ok(service_item),
            None => bail!(RegistryError::NoServiceItemFound),
        }
    }
    pub(crate) fn get_at(&self, pos: usize) -> anyhow::Result<&ServiceItem> {
        let result = self.services.get(pos);
        match result {
            Some(service_item) => Ok(service_item),
            None => bail!(RegistryError::NoServiceItemFound),
        }
    }
    pub(crate) fn get_at_mut(&mut self, pos: usize) -> anyhow::Result<&mut ServiceItem> {
        let result = self.services.get_mut(pos);
        match result {
            Some(service_item) => Ok(service_item),
            None => bail!(RegistryError::NoServiceItemFound),
        }
    }
    pub(crate) fn get_mut(
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
    pub(crate) fn list(&self, opts: ListOptions) -> Vec<&ServiceItem> {
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

                true
            })
            .collect()
        // TODO:("implement grouping and all that stuff")
    }
    pub(crate) fn get_local_node_services(&self) -> Vec<ServiceItemInfo> {
        self.services
            .iter()
            .filter(|svc| svc.local)
            .map(|svc| svc.into())
            .collect()
    }
    //remove all endpoints by node_id.
    pub(crate) fn remove_all_by_node_id(&mut self, node_id: &str) {
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

    pub(crate) fn remove(&mut self, full_name: &str, node_id: &str) {
        self.services.retain(|svc| {
            if svc.equals(full_name, Some(node_id)) {
                todo!("remove actions and events in registry");

                return false;
            }
            true
        })
    }
}

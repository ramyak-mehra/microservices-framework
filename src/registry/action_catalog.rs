use std::collections::HashMap;

use super::*;

#[derive(PartialEq, Eq, Default, Debug, Clone)]
pub(crate)struct ActionCatalog {
    actions: ActionsMap,
}

impl ActionCatalog {
    pub(crate)fn add(&mut self, node: &Node, service: &ServiceItem, action: Action) {
        let list = self.actions.get_mut(&action.name);
        match list {
            Some(list) => list.add(node, service, action),
            None => {
                let name = action.name.clone();
                let mut list = EndpointList::new(name.clone(), None);
                
                list.add(node, service, action);
                self.actions.insert(name, list);
            }
        }
    }
    pub(crate)fn get(&self, action_name: &str) -> Option<&EndpointList<ActionEndpoint>> {
        self.actions.get(action_name)
    }
    fn is_available(&self, action_name: &str) -> bool {
        match self.actions.get(action_name) {
            Some(el) => el.has_available(),
            None => false,
        }
    }
    fn remove_by_service(&mut self, service: &ServiceItem) {
        self.actions.iter_mut().for_each(|item| {
            let (_, el) = item;
            el.remove_by_service(service);
        });
    }
    pub(crate)fn remove(&mut self, action_name: &str, node_id: &str) {
        let list = self.actions.get_mut(action_name);
        if let Some(el) = list {
            el.remove_by_node_id(node_id);
        }
    }
    pub(crate)fn list(&self, opts: ListOptions) -> Vec<&EndpointList<ActionEndpoint>> {
        let res: HashMap<&String, &EndpointList<ActionEndpoint>> = self
            .actions
            .iter()
            .filter(|item| {
                let (name, ep_list) = item;
                if opts.skip_internal && get_internal_service_regex_match(name) {
                    return false;
                }
                if opts.only_local && !ep_list.has_local() {
                    return false;
                }
                if opts.only_available && !ep_list.has_available() {
                    return false;
                }
                if ep_list.count() > 0 {
                    let ep = ep_list.endpoints.get(0);
                    if let Some(ep) = ep {
                        if ep.action.visibility == Visibility::Protected {
                            return false;
                        }
                    }
                }
                true
            })
            .collect();
        let res = res.values().map(|ep| ep.to_owned()).collect();
        res
    }
}

use std::{collections::HashMap, net::IpAddr, sync::Arc};

use std::sync::RwLock;

use anyhow::bail;

use crate::registry::RegistryError;

use super::{node, Client, Logger, Node, Registry};

pub struct NodeCatalog {
    nodes: HashMap<String, Node>,
    local_node_id: String,
}
impl NodeCatalog {
    pub fn new() -> Self {
        //TODO: add the create local node logic here
        Self {
            nodes: HashMap::new(),
            local_node_id: "".to_string(),
        }
    }
    ///Create a local node
    fn create_local_node(&mut self, version: String, node_id: String, instance_id: String) {
        let client = Client {
            client_type: "rust".to_string(),
            lang_version: "1.56.1".to_string(),
            version: version,
        };
        let node = Node::new(node_id)
            .set_local(true)
            .set_ip_list(get_ip_list())
            .set_instance_id(instance_id)
            .set_hostname(get_hostname())
            .set_seq(1)
            .set_client(client);
        //TODO:data constency between local node and node catalog
        self.nodes.insert(node.id.to_string(), node.clone());
        self.local_node_id = node.id;
        todo!()
        /*
        node.metadata = self.broker.metadata.clone()
        */
    }
    pub fn add(&mut self, id: &str, node: Node) {
        self.nodes.insert(id.to_string(), node);
    }
    pub fn had_node(&self, id: &str) -> bool {
        match self.nodes.get(id) {
            Some(_) => true,
            None => false,
        }
    }
    pub fn get_node(&self, id: &str) -> Option<&Node> {
        self.nodes.get(id)
    }
    pub fn local_node(&self) -> anyhow::Result<&Node> {
        match self.nodes.get(&self.local_node_id) {
            Some(node) => Ok(node),
            None => bail!(RegistryError::NoLocalNodeFound),
        }
    }
    pub fn local_node_mut(&mut self) -> anyhow::Result<&mut Node> {
        match self.nodes.get_mut(&self.local_node_id) {
            Some(node) => Ok(node),
            None => bail!(RegistryError::NoLocalNodeFound),
        }
    }

    pub fn get_node_mut(&mut self, id: &str) -> Option<&mut Node> {
        self.nodes.get_mut(id)
    }
    pub fn delete(&mut self, id: &str) -> Option<Node> {
        self.nodes.remove(id)
    }
    pub fn count(&self) -> usize {
        self.nodes.len()
    }
    pub fn online_count(&self) -> usize {
        let mut count: usize = 0;
        self.nodes.iter().for_each(|node_item| {
            let (_, node) = node_item;
            if node.available {
                count = count.saturating_add(1);
            }
        });
        count
    }
    pub fn process_node_info(&self) {
        todo!()
    }
    pub fn disconnect(&mut self) {
        todo!()
    }

    pub fn list(&self, only_available: bool, with_services: bool) -> Vec<&Node> {
        self.nodes
            .values()
            .filter(|node| {
                if only_available && !node.available {
                    return false;
                }
                if with_services && node.services_len() <= 0 {
                    return false;
                }
                return true;
            })
            .collect()
    }
    pub fn list_mut(&mut self, only_available: bool, with_services: bool) -> Vec<&mut Node> {
        self.nodes
            .values_mut()
            .filter(|node| {
                if only_available && !node.available {
                    return false;
                }
                if with_services && node.services_len() <= 0 {
                    return false;
                }
                return true;
            })
            .collect()
    }
    pub fn nodes_vec(&self) -> Vec<&Node> {
        self.nodes.values().collect()
    }
    pub fn nodes_vec_mut(&mut self) -> Vec<&mut Node> {
        self.nodes.values_mut().collect()
    }
}
fn get_ip_list() -> Vec<IpAddr> {
    todo!()
}
fn get_hostname() -> String {
    todo!()
}

use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr},
    sync::Arc,
};

use anyhow::bail;
use serde_json::Value;

use crate::{errors::RegistryError, utils};

use super::{Client, Node};

#[derive(Debug)]

pub(crate)struct NodeCatalog {
    nodes: HashMap<String, Node>,
    local_node_id: String,
}
impl NodeCatalog {
    pub(crate)fn new() -> Self {
        //TODO: add the create local node logic here
        Self {
            nodes: HashMap::new(),
            local_node_id: "".to_string(),
        }
    }
    ///Create a local node
    pub(crate)fn create_local_node(
        &mut self,
        client_version: String,
        node_id: String,
        instance_id: String,
        metadata: Value,
    ) {
        let client = Client {
            client_type: "rust".to_string(),
            lang_version: "1.56.1".to_string(),
            version: client_version,
        };
        let node = Node::new(node_id)
            .set_local(true)
            .set_ip_list(utils::ip_list())
            .set_instance_id(instance_id)
            .set_hostname(utils::hostname().into_owned())
            .set_seq(1)
            .set_client(client)
            .set_metadata(metadata);
        self.nodes.insert(node.id.to_string(), node.clone());
        self.local_node_id = node.id;
    }
    pub(crate)fn add(&mut self, id: &str, node: Node) {
        self.nodes.insert(id.to_string(), node);
    }
    pub(crate)fn had_node(&self, id: &str) -> bool {
        self.nodes.get(id).is_some()
    }
    pub(crate)fn get_node(&self, id: &str) -> Option<&Node> {
        self.nodes.get(id)
    }
    pub(crate)fn get_node_mut(&mut self, id: &str) -> Option<&mut Node> {
        self.nodes.get_mut(id)
    }
    pub(crate)fn local_node(&self) -> anyhow::Result<&Node> {
        match self.get_node(&self.local_node_id) {
            Some(node) => Ok(node),
            None => bail!(RegistryError::NoLocalNodeFound),
        }
    }
    pub(crate)fn local_node_mut(&mut self) -> anyhow::Result<&mut Node> {
        let local_node_id = self.local_node_id.clone();
        match self.get_node_mut(&local_node_id) {
            Some(node) => Ok(node),
            None => bail!(RegistryError::NoLocalNodeFound),
        }
    }

    pub(crate)fn delete(&mut self, id: &str) -> Option<Node> {
        self.nodes.remove(id)
    }
    pub(crate)fn count(&self) -> usize {
        self.nodes.len()
    }
    pub(crate)fn online_count(&self) -> usize {
        let mut count: usize = 0;
        self.nodes.iter().for_each(|node_item| {
            let (_, node) = node_item;
            if node.available {
                count = count.saturating_add(1);
            }
        });
        count
    }
    pub(crate)fn process_node_info(&self) {
        todo!()
    }
    //Returns a bool if there was a node availabel that is removed
    pub(crate)fn disconnected(&mut self, node_id: &str)->Option<Node> {
        let node = self.get_node_mut(node_id);
        if let Some(node) = node {
            if node.available {
                node.disconnected();
                return Some(node.clone());
            }
        }
        None
        }

    pub(crate)fn list(&self, only_available: bool, with_services: bool) -> Vec<&Node> {
        self.nodes
            .values()
            .filter(|node| {
                if only_available && !node.available {
                    return false;
                }
                if with_services && node.services_len() == 0 {
                    return false;
                }
                true
            })
            .collect()
    }
    pub(crate)fn list_mut(&mut self, only_available: bool, with_services: bool) -> Vec<&mut Node> {
        self.nodes
            .values_mut()
            .filter(|node| {
                if only_available && !node.available {
                    return false;
                }
                if with_services && node.services_len() == 0 {
                    return false;
                }
                true
            })
            .collect()
    }
    pub(crate)fn nodes_vec(&self) -> Vec<&Node> {
        self.nodes.values().collect()
    }
    pub(crate)fn nodes_vec_mut(&mut self) -> Vec<&mut Node> {
        self.nodes.values_mut().collect()
    }
}

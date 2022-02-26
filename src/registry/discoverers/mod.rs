use std::sync::Arc;

use async_trait::async_trait;
use derive_more::Display;
use log::{info, warn};
use serde_json::{json, Value};
use tokio::sync::{mpsc, RwLock};

use crate::{
    transporter::{Transit, Transporter},
    Registry, ServiceBroker, ServiceBrokerMessage,
};

#[async_trait]
trait Discoverer {
    fn registry(&self) -> Arc<RwLock<Registry>>;

    fn broker_sender(&self) -> mpsc::UnboundedSender<ServiceBrokerMessage>;

    fn broker(&self) -> Arc<ServiceBroker>;

    fn opts(&self)->&DiscovererOpts;

    fn transit<T: Transporter + Send +Sync>(&self) -> &Transit<T>;
    async fn stop(&self) {
        self.stop_heartbeat_timers().await;
    }


    async fn start_heatbeat_timers(&self){
        self.stop_heartbeat_timers().await;
        // if 
    }



    async fn stop_heartbeat_timers(&self){
        todo!()
    }

    ///Discover a new or old node by node_id.
    fn discover_node(&self);

    /// Discover all nodes (after connected)
    fn disocer_all_nodes(&self);

    ///Called whent the local node is ready(transporter connected)
    fn local_node_ready(&self) {
        // Local node has started all local services. We send a new INFO packet
        // which contains the local services because we are ready to accept incoming requests.
        self.send_local_node_info();
    }

    fn send_local_node_info(&self);

    fn local_node_disconnected(&self) {
        todo!("check if it is present");
        // self.transit().send_disconnect_packet();
    }

    fn remote_node_disconnected(&self, node_id: &str, is_unexpected: bool) {
        let node = self.registry().blocking_write().nodes.disconnected(node_id);
        if let Some(node) = node {
            self.registry()
                .blocking_write()
                .unregister_service_by_node_id(node_id);
            let data = json!({"node":node , "unexpected":is_unexpected});
            let _ = self
                .broker_sender()
                .send(ServiceBrokerMessage::BroadcastLocal {
                    event_name: NodeEvents::Disconnected.to_string(),
                    data,
                    opts: Value::Null,
                });
            if is_unexpected {
                warn!("Node {} disconnected unexpectedly.", node_id);
            } else {
                info!("Node {} disconnected", node_id);
            }
            //TODO: remove pending requests from transit as well.
            if self.broker().transit.is_some() {}
        }
    }
}

#[derive(Debug, Display)]
enum NodeEvents {
    #[display(fmt = "$node.disconnected")]
    Disconnected,
}
struct DiscovererOpts{
    
}
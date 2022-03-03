use std::sync::Arc;

use async_trait::async_trait;
use chrono::Duration;
use derive_more::Display;
use log::{info, warn};
use serde_json::{json, Value};
use tokio::sync::{mpsc, RwLock, RwLockReadGuard};

use crate::{
    errors::TransitError,
    packet::{PayloadHeartbeat, PayloadInfo},
    transporter::{transit::TransitMessage, Transit, Transporter},
    Registry, ServiceBroker, ServiceBrokerMessage,
};

use super::Node;

#[async_trait]
trait Discoverer {
    fn transit_sender(&self) -> mpsc::UnboundedSender<TransitMessage>;

    fn registry(&self) -> Arc<RwLock<Registry>>;

    fn broker_sender(&self) -> mpsc::UnboundedSender<ServiceBrokerMessage>;

    fn broker(&self) -> Arc<ServiceBroker>;

    fn opts(&self) -> &DiscovererOpts;

    fn local_node(&self) -> &Node;
    async fn stop(&self) {
        self.stop_heartbeat_timers().await;
    }
    fn init(&mut self, registry: Arc<RwLock<Registry>>);
    async fn start_heatbeat_timers(&self) {
        self.stop_heartbeat_timers().await;
        // if
    }

    async fn stop_heartbeat_timers(&self) {
        todo!()
    }

    async fn check_offline_nodes(&self) {
        if self.opts().disable_offline_node_removing {
            return;
        }
        todo!("check offlien nodes")
    }

    async fn heartbeat_received(&self, node_id: &str, payload: PayloadHeartbeat) {
        let registry = self.registry();
        let registry = registry.read().await;
        let node = registry.nodes.get_node(node_id);
        todo!("hearbeat_recieved see below")
        //Payload while sending is heartbeat but it does not contain the necessary info to parse the payload.
    }

    async fn process_remote_node_info(&self, node_id: &str, payload: PayloadInfo) {
        let registry = &self.registry();
        let registry = registry.write().await;
        registry.process_node_info(node_id, payload);
    }

    async fn send_heartbeat(&self) -> anyhow::Result<()> {
        let cpu = self.local_node().cpu.clone();
        let _ = self
            .transit_sender()
            .send(TransitMessage::SendHeartBeat(cpu))?;

        Ok(())
    }

    ///Discover a new or old node by node_id.
    fn discover_node(&self, node_id: &str);

    /// Discover all nodes (after connected)
    fn disocer_all_nodes(&self);

    ///Called whent the local node is ready(transporter connected)
    fn local_node_ready(&self) {
        // Local node has started all local services. We send a new INFO packet
        // which contains the local services because we are ready to accept incoming requests.
        self.send_local_node_info();
    }

    fn send_local_node_info(&self);

    fn local_node_disconnected(&self) -> anyhow::Result<()> {
        let _ = self
            .transit_sender()
            .send(TransitMessage::SendDisconnectPacket)
            .map_err(|err| {
                return TransitError::SendError(
                    TransitMessage::SendDisconnectPacket,
                    err.to_string(),
                );
            })?;
        Ok(())
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
struct DiscovererOpts {
    disable_offline_node_removing: bool,
    clean_offline_nodes_timeout: Duration,
}

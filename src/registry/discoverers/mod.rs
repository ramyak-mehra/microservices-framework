mod local;

pub(crate) use std::sync::Arc;

use async_trait::async_trait;
use derive_more::Display;
use log::{info, warn};
use serde_json::{json, Value};
use std::time::Duration;
use tokio::sync::oneshot;
pub(crate) use tokio::sync::{mpsc, RwLock};
use tokio_js_set_interval::{_set_interval_spawn, clear_interval};

use crate::{
    broker_delegate::BrokerDelegate, transporter::transit::TransitMessage, utils, BrokerSender,
};
pub(crate) use crate::{
    packet::{PayloadHeartbeat, PayloadInfo},
    transporter::{Transit, Transporter},
    Registry, ServiceBroker, ServiceBrokerMessage,
};
type SharedRegistry = Arc<RwLock<Registry>>;
type TransitSender = mpsc::UnboundedSender<TransitMessage>;
#[async_trait]
trait Discoverer<T: Transporter + Sync + Send>
where
    Self: Sync + Send,
{
    fn registry(&self) -> &SharedRegistry;

    fn broker_sender(&self) -> BrokerSender;

    fn broker(&self) -> &Arc<BrokerDelegate>;

    fn opts(&self) -> &Arc<DiscovererOpts>;

    fn transit_sender(&self) -> TransitSender;

    fn node_id(&self) -> &str;
    fn set_heartbeat_interval(&mut self, interval: usize);

    fn discoverer_sender(&self) -> mpsc::UnboundedSender<DiscovererMessage>;
    fn discoverer_reciever(&mut self) -> &mut mpsc::UnboundedReceiver<DiscovererMessage>;

    async fn update_timer_id(&mut self, id: u64, timers_id: TimersId);

    async fn set_timer_id_null(&mut self, timers_id: TimersId);

    fn get_timer_id(&self, timers_id: TimersId) -> &Option<u64>;

    async fn local_node_cpu(&self) -> u32 {
        let registry = self.registry();
        let registry = registry.read().await;
        registry.nodes.local_node().unwrap().cpu.clone()
    }
    async fn stop(&mut self) {
        self.stop_heartbeat_timers().await;
    }
    fn init(&mut self, registry: Arc<RwLock<Registry>>);

    /// Start heartbeat timers
    async fn start_heatbeat_timers(&mut self) {
        self.stop_heartbeat_timers().await;
        if self.opts().heartbeat_interval > Duration::from_secs(0) {
            let interval_time = self.opts().heartbeat_interval;
            // Add random delay
            let heartbeat_time = interval_time;
            let heartbeat_sender = self.discoverer_sender();
            let check_nodes_sender = self.discoverer_sender();
            let offline_timer_sender = self.discoverer_sender();
            let send_heartbeat = move || {
                let _ = heartbeat_sender.send(DiscovererMessage::HeartBeat);
            };
            let send_check_nodes = move || {
                let _ = check_nodes_sender.send(DiscovererMessage::RemoteNode);
            };
            let send_offline_timer = move || {
                let _ = offline_timer_sender.send(DiscovererMessage::OfflineTimer);
            };
            let heartbeat_time = heartbeat_time.as_millis() as u64;
            let heartbeat_id = _set_interval_spawn(send_heartbeat, heartbeat_time);
            self.update_timer_id(heartbeat_id, TimersId::Heartbeat)
                .await;
            let check_nodes_id =
                _set_interval_spawn(send_check_nodes, interval_time.as_millis() as u64);
            self.update_timer_id(check_nodes_id, TimersId::CheckNodes)
                .await;
            let offline_timer_id = _set_interval_spawn(send_offline_timer, 60 * 1000);
            self.update_timer_id(offline_timer_id, TimersId::Offline)
                .await;
        }
    }
    /// Stop heatbeat timers
    async fn stop_heartbeat_timers(&mut self) {
        let heartbeat_timer_id = self.get_timer_id(TimersId::Heartbeat);
        if let Some(id) = heartbeat_timer_id {
            clear_interval(*id);
            self.set_timer_id_null(TimersId::Heartbeat).await;
        }

        let check_nodes_timer_id = self.get_timer_id(TimersId::CheckNodes);
        if let Some(id) = check_nodes_timer_id {
            clear_interval(*id);
            self.set_timer_id_null(TimersId::CheckNodes).await;
        }
        let offline_timer_id = self.get_timer_id(TimersId::Offline);
        if let Some(id) = offline_timer_id {
            clear_interval(*id);
            self.set_timer_id_null(TimersId::Offline).await;
        }
    }
    /// Disable built-in Hearbeat logic. Used by TCP transported.
    async fn disable_heartbeat(&mut self) {
        self.set_heartbeat_interval(0);
        self.start_heatbeat_timers().await;
    }
    /// Heartbeat method.
    async fn beat(registry: SharedRegistry) {
        let mut registry = registry.write().await;
        let cpu_usage = utils::get_cpu_usage().await;
        registry
            .nodes
            .local_node_mut()
            .unwrap()
            .update_local_info(cpu_usage);

        // self.send_heartbeat().await;
    }

    /// Check all registered remote nodes are available.
    async fn check_remote_nodes(registry: SharedRegistry, opts: Arc<DiscovererOpts>) {
        if opts.disable_heartbeat_checks {
            return;
        }

        let mut registry = registry.write().await;
        registry.check_remote_nodes(opts.heartbeat_timout).await;
    }
    /// Check offline nodes. Remove which are older than 10 minutes.
    async fn check_offline_nodes(registry: SharedRegistry, opts: Arc<DiscovererOpts>) {
        if opts.disable_offline_node_removing {
            return;
        }
        let mut registry = registry.write().await;
        registry
            .check_offline_nodes(opts.clean_offline_nodes_timeout)
            .await;
    }

    async fn heartbeat_received(registry:SharedRegistry , node_id: &str, payload: PayloadHeartbeat) {
        let registry = registry.read().await;
        let node = registry.nodes.get_node(node_id);
        todo!("hearbeat_recieved see below")
        //Payload while sending is heartbeat but it does not contain the necessary info to parse the payload.
    }

    async fn process_remote_node_info(
        registry: SharedRegistry,
        node_id: &str,
        payload: PayloadInfo,
    ) {
        let mut registry = registry.write().await;
        registry.process_node_info(node_id, payload).await;
    }

    async fn send_heartbeat(&self) {
        if !self.broker().transit_present() {
            return;
        }
        let cpu = self.local_node_cpu().await;
        //TODO: figure out how to get T of transit.(probably use a queue)
        // send_heartbeat(cpu).await;
    }

    ///Discover a new or old node by node_id.
    async fn discover_node(&self, node_id: &str);

    /// Discover all nodes (after connected)
    async fn discover_all_nodes(transit_sender: TransitSender);

    ///Called whent the local node is ready(transporter connected)
    async fn local_node_ready(
        registry: SharedRegistry,
        transit_sender: TransitSender,

        disable_balancer: bool,
    ) {
        // Local node has started all local services. We send a new INFO packet
        // which contains the local services because we are ready to accept incoming requests.
        Self::send_local_node_info(registry, transit_sender, None, disable_balancer).await;
    }

    async fn send_local_node_info(
        registry: SharedRegistry,
        transit_sender: TransitSender,
        node_id: Option<String>,
        disable_balancer: bool,
    );

    async fn local_node_disconnected(broker: Arc<BrokerDelegate>, node_id: String) {
        if !broker.transit_present() {
            return;
        }
        Transit::<T>::send_disconnect_packet(node_id).await;
    }

    async fn remote_node_disconnected(
        registry: SharedRegistry,
        broker_sender: BrokerSender,
        node_id: &str,
        is_unexpected: bool,
        transit_present: bool,
    ) {
        let node = registry
            .write()
            .await
            .nodes
            .disconnected(node_id, is_unexpected);
        if let Some(node) = node {
            registry
                .write()
                .await
                .unregister_service_by_node_id(node_id);
            let data = json!({"node":node , "unexpected":is_unexpected});
            let _ = broker_sender.send(ServiceBrokerMessage::BroadcastLocal {
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
            if transit_present {}
        }
    }
    async fn message_handler(&mut self) {
        while let Some(msg) = self.discoverer_reciever().recv().await {
            let registry = Arc::clone(&self.registry());
            let opts = Arc::clone(&self.opts());
            let broker = Arc::clone(&self.broker());
            let broker_sender = self.broker_sender();
            let transit_sender = self.transit_sender();
            let node_id = self.node_id().to_string();

            tokio::spawn(async move {
                match msg {
                    DiscovererMessage::HeartBeat => Self::beat(registry).await,
                    DiscovererMessage::RemoteNode => Self::check_remote_nodes(registry, opts).await,
                    DiscovererMessage::OfflineTimer => {
                        Self::check_offline_nodes(registry, opts).await
                    }
                    DiscovererMessage::LocalNodeDisconnect(sender) => {
                        Self::local_node_disconnected(broker, node_id).await;
                        let _ = sender.send(());
                    }
                    DiscovererMessage::LocalNodeready => {
                        Self::local_node_ready(
                            registry,
                            transit_sender,
                            broker.options().disable_balancer,
                        )
                        .await;
                    }
                    DiscovererMessage::DiscoverAllNodes => {
                        Self::discover_all_nodes(transit_sender).await;
                    }
                    DiscovererMessage::SendLocalNodeInfo { sender } => {
                        Self::send_local_node_info(
                            registry,
                            transit_sender,
                            Some(sender),
                            broker.options().disable_balancer,
                        )
                        .await;
                    }
                    DiscovererMessage::ProcessRemoteNodeInfo {
                        sender,
                        info_payload,
                    } => {
                        Self::process_remote_node_info(registry, &sender, info_payload).await;
                    }
                    DiscovererMessage::RemoteNodeDisconnected {
                        sender,
                        is_unexpected,
                    } => {
                        Self::remote_node_disconnected(
                            registry,
                            broker_sender,
                            &sender,
                            is_unexpected,
                            broker.transit_present(),
                        )
                        .await;
                    }
                    DiscovererMessage::HeartbeatRecieved {
                        sender,
                        heartbeat_payload,
                    } => {
                        Self::heartbeat_received(registry , &sender, heartbeat_payload).await;
                    },
                }
            });
        }
    }
}

#[derive(Debug, Display)]
pub(crate) enum NodeEvents {
    #[display(fmt = "$node.disconnected")]
    Disconnected,
    #[display(fmt = "$node.connected")]
    Connected,
    #[display(fmt = "$node.updated")]
    Updated,
}
struct DiscovererOpts {
    disable_heartbeat_checks: bool,
    disable_offline_node_removing: bool,
    clean_offline_nodes_timeout: Duration,
    heartbeat_interval: Duration,
    heartbeat_timout: Duration,
}
pub(crate) enum DiscovererMessage {
    HeartBeat,
    RemoteNode,
    OfflineTimer,
    LocalNodeDisconnect(oneshot::Sender<()>),
    LocalNodeready,
    DiscoverAllNodes,
    SendLocalNodeInfo {
        sender: String,
    },
    ProcessRemoteNodeInfo {
        sender: String,
        info_payload: PayloadInfo,
    },
    RemoteNodeDisconnected {
        sender: String,
        is_unexpected: bool,
    },
    HeartbeatRecieved {
        sender: String,
        heartbeat_payload: PayloadHeartbeat,
    },
}
enum TimersId {
    Heartbeat,
    CheckNodes,
    Offline,
}

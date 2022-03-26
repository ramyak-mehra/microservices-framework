mod local;
pub(crate) use local::LocalDiscoverer;
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
    broker::BrokerOptions,
    broker_delegate::BrokerDelegate,
    transporter::transit::{TransitMessage, TransporterEvents},
    utils, BrokerSender, DiscovererSender, SharedRegistry, TransitSender,
};
pub(crate) use crate::{
    packet::{PayloadHeartbeat, PayloadInfo},
    transporter::{Transit, Transporter},
    Registry, ServiceBroker, ServiceBrokerMessage,
};

#[async_trait]
pub(crate) trait Discoverer<T: Transporter + Sync + Send>
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

    fn discoverer_sender(&self) -> DiscovererSender;
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
    async fn init(&self) {
        let transporter_connected_sub = self.broker().create_subscriber().await;
        transporter_connected_sub
            .subscribe_to(TransporterEvents::Connected.to_string())
            .await;
        let transporter_disconnected_sub = self.broker().create_subscriber().await;
        transporter_disconnected_sub
            .subscribe_to(TransporterEvents::Disconnected.to_string())
            .await;
        let discoverer_sender = self.discoverer_sender();
        tokio::spawn(async move {
            let _connected_message = transporter_connected_sub
                .receiver()
                .recv_async()
                .await
                .unwrap();
            let _ = discoverer_sender.send(DiscovererMessage::StartHeartbeatTimer);
        });
        let discoverer_sender = self.discoverer_sender();
        tokio::spawn(async move {
            let _disconnected_message = transporter_disconnected_sub
                .receiver()
                .recv_async()
                .await
                .unwrap();
            let _ = discoverer_sender.send(DiscovererMessage::StopHeartbeatTimer);
        });
    }

    /// Start heartbeat timers
    async fn start_heartbeat_timers(&mut self) {
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
        self.start_heartbeat_timers().await;
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

    async fn heartbeat_received(
        registry: SharedRegistry,
        node_id: &str,
        payload: PayloadHeartbeat,
    ) {
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
            if matches!(msg, DiscovererMessage::StartHeartbeatTimer) {
                self.start_heartbeat_timers().await;
            } else if matches!(msg, DiscovererMessage::StopHeartbeatTimer) {
                self.stop_heartbeat_timers().await;
            }
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
                    DiscovererMessage::LocalNodeReady => {
                        Self::local_node_ready(
                            registry,
                            transit_sender,
                            broker.options().disable_balancer,
                        )
                        .await;
                    }
                    DiscovererMessage::DiscoverAllNodes(sender_ch) => {
                        Self::discover_all_nodes(transit_sender).await;
                        let _ = sender_ch.send(());
                    }
                    DiscovererMessage::SendLocalNodeInfo(sender, sender_ch) => {
                        Self::send_local_node_info(
                            registry,
                            transit_sender,
                            sender,
                            broker.options().disable_balancer,
                        )
                        .await;
                        let _ = sender_ch.send(());
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
                        Self::heartbeat_received(registry, &sender, heartbeat_payload).await;
                    }
                    DiscovererMessage::StartHeartbeatTimer => todo!(),
                    DiscovererMessage::StopHeartbeatTimer => todo!(),
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
#[derive(Debug)]
pub(crate) struct DiscovererOpts {
    disable_heartbeat_checks: bool,
    disable_offline_node_removing: bool,
    clean_offline_nodes_timeout: Duration,
    heartbeat_interval: Duration,
    heartbeat_timout: Duration,
}
impl Default for DiscovererOpts {
    fn default() -> Self {
        Self {
            disable_heartbeat_checks: false,
            disable_offline_node_removing: false,
            clean_offline_nodes_timeout: Duration::from_secs(10 * 60),
            heartbeat_interval: Duration::from_secs(5),
            heartbeat_timout: Duration::from_secs(15),
        }
    }
}

pub(crate) enum DiscovererMessage {
    StartHeartbeatTimer,
    StopHeartbeatTimer,
    HeartBeat,
    RemoteNode,
    OfflineTimer,
    LocalNodeDisconnect(oneshot::Sender<()>),
    LocalNodeReady,
    DiscoverAllNodes(oneshot::Sender<()>),
    SendLocalNodeInfo(Option<String>, oneshot::Sender<()>),
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
pub(crate) enum TimersId {
    Heartbeat,
    CheckNodes,
    Offline,
}

#[cfg(test)]
mod tests {
    use circulate::Relay;

    use crate::{
        broker::BrokerOptions,
        registry::{Logger, Payload},
    };

    use super::*;

    fn get_test_broker(
        recv: mpsc::UnboundedReceiver<ServiceBrokerMessage>,
        sender: BrokerSender,
        registry: Arc<RwLock<Registry>>,
        broker_options: Arc<BrokerOptions>,
        instance: String,
        node_id: String,
    ) -> ServiceBroker {
        ServiceBroker {
            reciever: recv,
            started: false,
            namespace: None,
            metdata: Payload {},
            sender,
            instance,
            node_id,
            services: Vec::new(),
            transit: None,
            logger: Arc::new(Logger {}),
            options: broker_options,
            registry,
            local_bus: Relay::default(),
        }
    }
    async fn get_test_registry(
        node_id: &str,
        instance: &str,
        broker_options: Arc<BrokerOptions>,
        broker_sender: BrokerSender,
    ) -> Arc<RwLock<Registry>> {
        let registry = Arc::new(RwLock::new(Registry::new(
            broker_sender.clone(),
            node_id.to_owned(),
            instance.to_owned(),
            broker_options
        )));
        registry.write().await.regenerate_local_raw_info(None);
        registry
    }

    fn get_discoverer(
        broker_sender: BrokerSender,
        transit_sender: TransitSender,
        broker: Arc<BrokerDelegate>,
        registry: Arc<RwLock<Registry>>,
        node_id: &str,
    ) -> LocalDiscoverer {
        let opts = DiscovererOpts::default();
        LocalDiscoverer::new(
            transit_sender,
            registry,
            opts,
            broker_sender,
            broker,
            node_id.to_owned(),
        )
    }

    async fn get_test_discoverer() -> (LocalDiscoverer, mpsc::UnboundedReceiver<TransitMessage>) {
        let node_id = "test_node".to_string();
        let instance = "test_instance".to_string();
        let (broker_sender, recv) = mpsc::unbounded_channel::<ServiceBrokerMessage>();
        let (transit_sender, transit_recv) = mpsc::unbounded_channel::<TransitMessage>();
        let broker_options = Arc::new(BrokerOptions::default());
        let registry = get_test_registry(
            &node_id,
            &instance,
            Arc::clone(&broker_options),
            broker_sender.clone(),
        ).await;
        let broker = get_test_broker(
            recv,
            broker_sender.clone(),
            registry.clone(),
            broker_options.clone(),
            instance.clone(),
            node_id.clone(),
        );
        let mut broker_delegate = BrokerDelegate::new();
        broker_delegate.set_broker(Arc::new(broker));
        let broker_delegate = Arc::new(broker_delegate);
        (
            get_discoverer(
                broker_sender,
                transit_sender,
                broker_delegate,
                registry.clone(),
                &node_id,
            ),
            transit_recv,
        )
    }

    #[tokio::test]
    async fn test_disover_nodes() {
        let (mut discoverer, mut transit_recv) = get_test_discoverer().await;
        let discoverer_sender = discoverer.discoverer_sender();
        discoverer.init().await;
        tokio::spawn(async move {
            discoverer.message_handler().await;
        });
        let transit_jh = tokio::spawn(async move {
            while let Some(msg) = transit_recv.recv().await {
                assert!(matches!(msg, TransitMessage::DiscoverNodes));
                break;
            }
        });
        let (sender, recv) = oneshot::channel::<()>();
        let _ = discoverer_sender.send(DiscovererMessage::DiscoverAllNodes(sender));
        transit_jh.await;
    }
    #[tokio::test]
    async fn test_send_local_node_info() {
        let (mut discoverer, mut transit_recv) = get_test_discoverer().await;
        let discoverer_sender = discoverer.discoverer_sender();
        discoverer.init().await;
        tokio::spawn(async move {
            discoverer.message_handler().await;
        });
        let transit_jh = tokio::spawn(async move {
            while let Some(msg) = transit_recv.recv().await {
                assert!(matches!(msg, TransitMessage::SendNodeInfo { .. }));
                break;
            }
        });
        let (sender, recv) = oneshot::channel::<()>();
        let _ = discoverer_sender.send(DiscovererMessage::SendLocalNodeInfo(
            Some("test_node".to_string()),
            sender,
        ));
        transit_jh.await;
    }
}

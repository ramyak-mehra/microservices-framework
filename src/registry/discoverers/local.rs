use super::*;
use crate::{broker_delegate::BrokerDelegate, transporter::nats::NatsTransporter, BrokerSender};
use tokio_js_set_interval::{_set_interval_spawn, set_interval};

struct LocalDiscoverer {
    transit: Transit<NatsTransporter>,
    registry: Arc<RwLock<Registry>>,
    opts: DiscovererOpts,
    broker_sender: BrokerSender,
    broker: Arc<BrokerDelegate>,
    discoverer_reciever: mpsc::UnboundedReceiver<DiscovererMessage>,
    discoverer_sender: mpsc::UnboundedSender<DiscovererMessage>,
    heartbeat_timer_id: Option<u64>,
    check_nodes_timer_id: Option<u64>,
    offline_timer_id: Option<u64>,
}

impl LocalDiscoverer {
    fn new(registry: Arc<RwLock<Registry>>) {}
    async fn start_heatbeat_timers(&mut self) {
        self.stop_heartbeat_timers().await;
        if self.opts().heartbeat_interval > Duration::from_secs(0) {
            // Add random delay.
            let time = self.opts.heartbeat_interval;
        }
    }
}
#[async_trait]
impl Discoverer<NatsTransporter> for LocalDiscoverer {
    fn registry(&self) -> &Arc<tokio::sync::RwLock<Registry>> {
        &self.registry
    }

    fn broker_sender(&self) -> &BrokerSender {
        &self.broker_sender
    }

    fn broker(&self) -> &Arc<BrokerDelegate> {
        &self.broker
    }

    fn opts(&self) -> &DiscovererOpts {
        &self.opts
    }

    fn transit(&self) -> &Transit<NatsTransporter> {
        &self.transit
    }

    fn set_heartbeat_interval(&mut self, interval: usize) {
        todo!()
    }

    fn discoverer_sender(&self) -> mpsc::UnboundedSender<DiscovererMessage> {
        self.discoverer_sender.clone()
    }

    fn init(&mut self, registry: Arc<RwLock<Registry>>) {
        todo!("stuff related to local bus")
    }

    async fn discover_node(&self, node_id: &str) {
        let broker_sender = self.broker_sender.clone();
        let node_id = node_id.to_string();
        tokio::spawn(async move {
            Transit::<NatsTransporter>::discover_node(broker_sender, node_id).await;
        });
    }

    async fn discover_all_nodes(&self) {
        let broker_sender = self.broker_sender.clone();
        tokio::spawn(async move {
            Transit::<NatsTransporter>::discover_nodes(broker_sender).await;
        });
    }

    async fn send_local_node_info(&self, node_id: Option<String>) {
        let info = self.registry.read().await.get_local_node_info(false);
        match info {
            Ok(info) => {
                if let None = node_id {
                    if self.broker.options().disable_balancer {
                        let _ = self.transit.tx.make_balanced_subscriptions().await;
                    }
                };
                self.transit.send_node_info(info, node_id);
            }
            Err(err) => warn!("No local info present. {}", err),
        }
    }

    async fn update_timer_id(&mut self, id: u64, timers_id: TimersId) {
        match timers_id {
            TimersId::Heartbeat => self.heartbeat_timer_id = Some(id),
            TimersId::CheckNodes => self.check_nodes_timer_id = Some(id),
            TimersId::Offline => self.offline_timer_id = Some(id),
        }
    }

    async fn set_timer_id_null(&mut self, timers_id: TimersId) {
        match timers_id {
            TimersId::Heartbeat => self.heartbeat_timer_id = None,
            TimersId::CheckNodes => self.check_nodes_timer_id = None,
            TimersId::Offline => self.offline_timer_id = None,
        }
    }

    fn get_timer_id(&self, timers_id: TimersId) -> &Option<u64> {
        match timers_id {
            TimersId::Heartbeat => &self.heartbeat_timer_id,
            TimersId::CheckNodes => &self.check_nodes_timer_id,
            TimersId::Offline => &self.offline_timer_id,
        }
    }

    fn discoverer_reciever(&mut self) -> &mut mpsc::UnboundedReceiver<DiscovererMessage> {
        &mut self.discoverer_reciever
    }
}

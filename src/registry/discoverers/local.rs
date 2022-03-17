use super::*;
use crate::{
    broker_delegate::BrokerDelegate,
    transporter::{nats::NatsTransporter, transit::TransitMessage},
    BrokerSender,
};
use tokio_js_set_interval::{_set_interval_spawn, set_interval};

struct LocalDiscoverer {
    transit: mpsc::UnboundedSender<TransitMessage>,
    registry: Arc<RwLock<Registry>>,
    opts: DiscovererOpts,
    broker_sender: BrokerSender,
    broker: Arc<BrokerDelegate>,
    discoverer_reciever: mpsc::UnboundedReceiver<DiscovererMessage>,
    discoverer_sender: mpsc::UnboundedSender<DiscovererMessage>,
    heartbeat_timer_id: Option<u64>,
    check_nodes_timer_id: Option<u64>,
    offline_timer_id: Option<u64>,
    node_id: String,
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
    fn transit_sender(&self) -> mpsc::UnboundedSender<TransitMessage> {
        self.transit.clone()
    }
}
#[async_trait]
impl Discoverer<NatsTransporter> for LocalDiscoverer {
    fn registry(&self) -> &Arc<tokio::sync::RwLock<Registry>> {
        &self.registry
    }
    fn node_id(&self) -> &str {
        &self.node_id
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
        let node_id = node_id.to_string();
        let transit_sender = self.transit_sender();

        transit_sender.send(TransitMessage::DiscoverNode { node_id });
    }

    async fn discover_all_nodes(&self) {
        let transit_sender = self.transit_sender();

        transit_sender.send(TransitMessage::DiscoverNodes);
    }

    async fn send_local_node_info(&self, node_id: Option<String>) {
        let info = self.registry.read().await.get_local_node_info(false);
        let transit_sender = self.transit_sender();
        match info {
            Ok(info) => {
                if let None = node_id {
                    if self.broker.options().disable_balancer {
                        let (sender, mut recv) = oneshot::channel::<()>();
                        transit_sender.send(TransitMessage::MakeBalancedSubscription(sender));
                        recv.await;
                    }
                };

                transit_sender.send(TransitMessage::SendNodeInfo { info, node_id });
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

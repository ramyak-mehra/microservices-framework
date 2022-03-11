use crate::{transporter::nats::NatsTransporter, broker_delegate::BrokerDelegate};

use super::*;

struct LocalDiscoverer {
    transit: Transit<NatsTransporter>,
    registry: Arc<RwLock<Registry>>,
    opts: DiscovererOpts,
    broker_sender: mpsc::UnboundedSender<ServiceBrokerMessage>,
    broker: Arc<BrokerDelegate>,
}

impl LocalDiscoverer {
    fn new(registry: Arc<RwLock<Registry>> , ) {}
}
#[async_trait]
impl Discoverer<NatsTransporter> for LocalDiscoverer {
    fn registry(&self) -> &Arc<tokio::sync::RwLock<Registry>> {
        &self.registry
    }

    fn broker_sender(&self) -> &mpsc::UnboundedSender<ServiceBrokerMessage> {
        &self.broker_sender
    }

    fn broker(&self) -> &Arc<BrokerDelegate> {
        &self.broker
    }

    fn opts(&self) -> &DiscovererOpts {
        &self.opts
    }

    fn init(&mut self, registry: Arc<RwLock<Registry>>) {
        todo!("stuff related to local bus")
    }

    async fn discover_node(&self, node_id: &str) {
        self.transit.discover_node(node_id.to_string()).await;
    }

    async fn discover_all_nodes(&self) {
        self.transit.discover_nodes().await;
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

    fn transit(&self) -> &Transit<NatsTransporter> {
        &self.transit
    }
}

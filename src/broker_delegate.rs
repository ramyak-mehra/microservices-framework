use std::sync::Arc;

use anyhow::bail;
use circulate::Subscriber;

use crate::{
    broker::BrokerOptions,
    context::Context,
    registry::{discoverers::ServiceBroker, ActionEndpoint},
    serializers::json::JSONSerializer,
};
#[derive(Debug)]
pub(crate) struct BrokerDelegate {
    broker: Option<Arc<ServiceBroker>>,
    node_id: String,
    instance_id: String,
    namespace: Option<String>,
}

impl BrokerDelegate {
    fn init() -> Self {
        Self {
            broker: None,
            node_id: "".to_string(),
            instance_id: "".to_string(),
            namespace: None,
        }
    }
    fn check_broker(&self) {
        if let None = self.broker {
            panic!("Broker not initialsed inside broker delegate")
        }
    }
    fn broker(&self) -> &Arc<ServiceBroker> {
        match &self.broker {
            Some(broker) => broker,
            None => panic!("Broker not initialised inside broker delegegate"),
        }
    }
    pub(crate) fn set_broker(&mut self, broker: Arc<ServiceBroker>) {
        self.node_id = broker.node_id.clone();
        self.instance_id = broker.instance.clone();
        self.broker = Some(broker);
    }
    pub(crate) fn node_id(&self) -> &str {
        self.check_broker();
        &self.node_id
    }
    pub(crate) fn namespace(&self) -> &Option<String> {
        self.check_broker();
        &self.namespace
    }
    pub(crate) fn instance_id(&self) -> &str {
        self.check_broker();
        &self.instance_id
    }
    pub(crate) fn options(&self) -> Arc<BrokerOptions> {
        self.check_broker();
        self.broker().options.clone()
    }
    pub(crate) fn transit_present(&self) -> bool {
        self.broker().transit.is_some()
    }
    pub(crate) async fn get_local_action_endpoint(
        &self,
        action_name: &str,
        ctx: &Context,
    ) -> anyhow::Result<ActionEndpoint> {
        self.broker()
            .get_local_action_endpoint(action_name, ctx)
            .await
    }
    pub(crate) fn get_broker(&self) -> &ServiceBroker {
        self.broker().as_ref()
    }
    pub(crate) fn serializer(&self) -> &JSONSerializer {
        self.broker().serializer()
    }
    pub(crate) async fn create_subscriber(&self) -> Subscriber {
        self.broker().create_subscriber().await
    }
}

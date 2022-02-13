use std::sync::Arc;

use anyhow::{bail, Result};
use chrono::NaiveDateTime;
use serde_json::{json, Value};
use tokio::sync::{mpsc::Sender, oneshot};

use crate::{
    registry::{
        service_item::ServiceItem, Action, ActionEndpoint, EndpointTrait, EndpointType, Event,
    },
    HandlerResult, ServiceBroker, ServiceBrokerMessage,
};

struct ContextOptions {
    timeout: i64,
    retries: usize,
}

pub struct Context<E: EndpointTrait> {
    id: String,
    broker_sender: Sender<ServiceBrokerMessage>,
    action: Option<Action>,
    event: Option<Event>,
    node_id: Option<String>,
    tracing: bool,
    level: usize,
    broker: Arc<ServiceBroker>,
    endpoint: Option<E>,
    service: ServiceItem,
    start_hr_time: NaiveDateTime,
    options: ContextOptions,
    event_type: EventType,
    event_groups: Vec<String>,
    parent_id: Option<String>,
    caller: Option<String>,
    meta: Value,
    locals: Value,
    request_id: String,
    parent_ctx: Option<Box<Context<E>>>,

    /*
    tracing
    span
    span_stack
    need_ack
    ack_id
    startTime = null;
    startHrTime = null;
    stopTime = null;
    duration = null;
    error = null;
    */
    cached_result: bool,
}
enum EventType {
    Emit,
    Broadcast,
}

impl<E: EndpointTrait> Context<E> {
    fn new(broker: Arc<ServiceBroker>, endpoint: E) -> Self {
        // let mut node_id = None;
        // if let Some(broker) = broker {
        //     node_id = Some(broker.node_id.clone());
        //     //broker generate uid;
        // }
        todo!()
    }

    fn create(broker: Arc<ServiceBroker>, endpoint: E, params: Value, opts: Value) {
        let mut ctx = Context::new(broker, endpoint);
    }

    fn set_endpoint(&mut self, endpoint: Option<E>) {
        //self.endpoint = Some(ep);
        if let Some(ep) = endpoint {
            self.node_id = Some(ep.id().to_owned());
            let ep_type = ep.ep_type();
            match ep_type {
                EndpointType::Action => {}
                EndpointType::Event => {}
            }
        }
    }

    async fn call(&self, action_name: &str, params: Value, mut opts: Value) -> Result<()> {
        let mut opts = opts.as_object();
        if let Some(opts) = opts {
            //TODO:set the parent context
            //opts.insert("parentCtx".to_string(), self.clone());
        }
        let opts = Value::Object(opts.unwrap().to_owned());
        if self.options.timeout > 0 {
            //TODO: callculate time difference for distributed distance
        }

        if self.broker.options.max_call_level > 0
            && self.level >= self.broker.options.max_call_level
        {
            bail!("Max call level error")
        }

        let (sender, mut recv) = oneshot::channel::<HandlerResult>();

        let _result = self
            .broker_sender
            .send(ServiceBrokerMessage::Call {
                action_name: action_name.to_string(),
                params,
                opts,
                result_channel: sender,
            });
        let result = recv.await?;

        //TODO:merge meta of the context object

        Ok(())
    }

    async fn emit(&self, event_name: &str, data: Value, opts: Option<Value>) {
        let mut opts: Value = match opts {
            Some(opts) => {
                let value = json!({ "groups": opts });
                value
            }
            _ => json!({}),
        };
        if let Some(groups) = opts.get("groups") {
            if !groups.is_array() {
                opts = json!({ "groups": vec![groups] });
            };
        }
        //TODO:Set the parent context
        let _result = self.broker_sender.send(ServiceBrokerMessage::Emit {
            event_name: event_name.to_string(),
            data,
            opts,
        });
    }

    async fn broadcast(&self, event_name: &str, data: Value, opts: Option<Value>) {
        let mut opts: Value = match opts {
            Some(opts) => {
                let value = json!({ "groups": opts });
                value
            }
            _ => json!({}),
        };
        if let Some(groups) = opts.get("groups") {
            if !groups.is_array() {
                opts = json!({ "groups": vec![groups] });
            };
        }
        //TODO:Set the parent context
        let _result = self.broker_sender.send(ServiceBrokerMessage::Broadcast {
            event_name: event_name.to_string(),
            data,
            opts,
        });
    }
    fn start_span() {
        todo!("while implementing tracing")
    }
    fn finish_span() {
        todo!("while implementing tracing")
    }
}

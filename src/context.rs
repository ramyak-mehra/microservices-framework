use std::sync::Arc;

use anyhow::{bail, Result};
use chrono::NaiveDateTime;
use serde_json::{json, Value};
use tokio::sync::{
    mpsc::{Sender, UnboundedSender},
    oneshot,
};

use crate::{
    broker::{BrokerOptions, CallOptions},
    registry::{service_item::ServiceItem, Action, EndpointTrait, EndpointType, Event, Payload},
    utils, HandlerResult, ServiceBroker, ServiceBrokerMessage,
};
#[derive(Debug, Clone)]

struct ContextOptions {
    timeout: Option<i64>,
    retries: Option<usize>,
}

#[derive(Debug)]
pub struct Context {
    id: String,
    pub request_id: Option<String>,
    broker_sender: UnboundedSender<ServiceBrokerMessage>,
    action: Option<String>,
    event: Option<String>,
    parent_id: Option<String>,
    event_groups: Option<Vec<String>>,
    event_type: EventType,
    pub params: Option<Payload>,
    meta: Payload,
    caller: Option<String>,
    locals: Option<Payload>,
    node_id: Option<String>,

    tracing: bool,
    level: usize,

    service: String,

    options: ContextOptions,
    parent_ctx: Option<Box<Context>>,
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
#[derive(Debug)]
enum EventType {
    Emit,
    Broadcast,
}

impl Context {
    pub fn new(broker: &ServiceBroker, service: String) -> Self {
        let id = utils::generate_uuid();
        let request_id = id.clone();
        let meta = Payload {};

        Self {
            id,
            request_id: Some(request_id),
            broker_sender: broker.sender.clone(),
            action: None,
            event: None,
            parent_id: None,
            event_groups: None,
            event_type: EventType::Emit,
            meta,
            caller: None,
            locals: None,
            node_id: Some(broker.node_id.clone()),
            tracing: false,
            level: 1,
            options: ContextOptions {
                timeout: None,
                retries: None,
            },
            parent_ctx: None,
            cached_result: false,
            service: service,
            params: None,
        }
    }

    pub fn child_action_context(
        &self,
        broker: &ServiceBroker,
        params: Payload,
        opts: Option<CallOptions>,
        action: &str,
    ) -> Self {
        //    let parent_ctx = self.clone();
        let meta = self.meta.to_owned();
        if broker.options.metrics {
            //TODO:
            //meta = meta.add("tracing" , true)
        }
        if opts.is_some() {
            //TODO:
            //copy the options meta
        }
        let id = utils::generate_uuid();
        let mut request_id = id.clone();
        if let Some(id) = self.request_id.clone() {
            request_id = id;
        }

        let mut caller = action.to_string();

        let service: Vec<&str> = caller.split('.').collect();
        let service = service.get(0).unwrap().to_string();
        Self {
            id,
            request_id: Some(request_id),
            broker_sender: self.broker_sender.clone(),
            action: Some(action.to_string()),
            event: None,
            parent_id: Some(self.id.clone()),
            event_groups: None,
            event_type: EventType::Emit,
            meta,
            caller: Some(caller),
            locals: self.locals.to_owned(),
            node_id: self.node_id.clone(),
            tracing: self.tracing,
            level: self.level + 1,
            options: self.options.to_owned(),
            parent_ctx: None,
            cached_result: false,
            service: service,
            params: Some(params),
        }
    }

    pub fn action(&self) -> &str {
        self.action.as_ref().unwrap()
    }
    pub fn node_id(&self) -> &String {
        self.node_id.as_ref().unwrap()
    }

    async fn call(
        &self,
        broker_options: &BrokerOptions,
        action_name: &str,
        params: Payload,
        mut opts: CallOptions,
    ) -> Result<()> {
        // if let Some(opts) = opts {
        //TODO:set the parent context
        //opts.insert("parentCtx".to_string(), self.clone());
        // }
        // let opts = Value::Object(opts.unwrap().to_owned());
        if self.options.timeout > Some(0) {
            //TODO: callculate time difference for distributed distance
        }

        if broker_options.max_call_level > 0 && self.level >= broker_options.max_call_level {
            bail!("Max call level error")
        }

        let (sender, recv) = oneshot::channel::<anyhow::Result<HandlerResult>>();

        let _result = self.broker_sender.send(ServiceBrokerMessage::Call {
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

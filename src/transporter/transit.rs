use std::{collections::HashMap, sync::Arc};

use chrono::{DateTime, Local};
use derive_more::Display;
use log::{debug, error, info, warn};

use serde_json::{json, Value};

use super::*;
use crate::{
    constants::*,
    context::{Context, EventType},
    errors::{PacketError, ServiceBrokerError},
    registry::{Action, EndpointTrait, EventEndpoint, Node, Payload, node::NodeRawInfo},
    utils, HandlerResult,
};
use crate::{ServiceBroker, ServiceBrokerMessage};
use anyhow::{self, bail};
use tokio::{
    sync::{mpsc, oneshot},
    task,
};

type P = PacketType;
struct TransitOptions {
    max_queue_size: Option<usize>,
}

struct Request {
    node_id: Option<String>,
    action: String,
    ctx: Context,
    resolve: fn(HandlerResult),
}

pub(crate) struct Transit<T: Transporter + Send + Sync> {
    broker: Arc<ServiceBroker>,
    broker_sender: mpsc::UnboundedSender<ServiceBrokerMessage>,
    pub(crate)tx: T,
    opts: TransitOptions,
    node_id: String,
    instance_id: String,
    /*
    discoverer
    */
    conntected: bool,
    disconnecting: bool,
    is_ready: bool,
    pending_requests: HashMap<String, Request>,
}

impl<T: Transporter + Send + Sync> Transit<T> {
    fn new(
        broker: Arc<ServiceBroker>,
        opts: TransitOptions,
        transporter: T,
        broker_sender: mpsc::UnboundedSender<ServiceBrokerMessage>,
    ) -> Self {
        let broker = broker.clone();
        let node_id = broker.node_id.clone();
        let instance_id = broker.instance.clone();

        Self {
            broker,
            tx: transporter,
            opts,
            node_id,
            instance_id,
            broker_sender,
            conntected: false,
            disconnecting: false,
            is_ready: false,
            pending_requests: HashMap::new(),
        }
    }

    fn after_connect() {
        todo!()
    }

    async fn connect(&mut self) {
        info!("Connecting to the transported...");
        todo!()
    }
    async fn disconnect(&mut self) {
        self.conntected = false;
        self.is_ready = false;
        self.disconnecting = true;
        let data = json!({"graceFul" : true});
        let _ = self
            .broker_sender
            .send(ServiceBrokerMessage::BroadcastLocal {
                event_name: TransporterEvents::Disconnected.to_string(),
                data,
                opts: Value::Null,
            });
        if self.tx.connected() {
            //TODO: call discoverer local node disconnected then
            self.tx.disconnect().await;
            self.disconnecting = false;
        }
    }

    fn read(&mut self) {
        if self.conntected {
            self.is_ready = true;
            //TODO:
            //return self.discoverer.localnodeready()
            todo!()
        }
    }
    ///Send DISCONNECT to remote nodes
    pub(crate)async fn send_disconnect_packet(&self) {
        todo!("after publish")
    }
    async fn make_subsciptions(&self) {
        let topics = vec![
            self.packet_topic(P::Event),
            self.packet_topic(P::Request),
            self.packet_topic(P::Response),
            //Discover handler
            self.packet_topic(P::Discover),
            self.packet_topic_witout_node_id(P::Discover),
            //NodeInfo handler
            self.packet_topic(P::Info), //Broadcast INFO. If a new node connected.
            self.packet_topic_witout_node_id(P::Info), //Resposne INFO to DISCOVER packet.
            // Disconnect handler
            self.packet_topic_witout_node_id(P::Disconnect),
            //Hearbeat handler
            self.packet_topic_witout_node_id(P::Heartbeat),
            //Ping handler
            self.packet_topic_witout_node_id(P::Ping), //Broadcasted
            self.packet_topic(P::Ping),                //Targeted
            //Pong handler
            self.packet_topic(P::Pongs),
        ];
        self.tx.make_subsciptions(topics).await;
        todo!("return from the make subscriptions");
    }

    async fn message_handler<P: PacketPayload>(&self, cmd: PacketType, packet: Packet<P>) {
        let payload = packet.payload;
        //TODO: check for empty payload i.e PayloadNull
        // TODO: Check protocol version

        if payload.sender() == self.node_id {
            if cmd == PacketType::Info && payload.instance_id() != self.instance_id {
                panic!("Service Broker has detected a nodeID conflict, use unique nodeIDS. ServiceBroker has stopped.")
            }
            if cmd != PacketType::Event && cmd != PacketType::Request && cmd != PacketType::Response
            {
                return;
            }
        }
        todo!()
    }

    async fn event_handler(&self, payload: PayloadEvent) -> anyhow::Result<bool> {
        debug!(
            "Event {} received from {} node {:?}",
            payload.event, payload.sender, payload.groups
        );
        if !self.broker.started {
            warn!(
                "Incoming {} event from {} node is dropped, because broker is stopped",
                payload.event, payload.sender
            );
            return Ok(false);
        }
        //TODO: handle service name not present
        let mut ctx = Context::new(self.broker.as_ref(), "".to_string());
        ctx.id = payload.id;
        ctx.event_name = Some(payload.event);
        ctx.params = Some(payload.data);
        ctx.event_groups = Some(payload.groups);
        ctx.event_type = if payload.broadcast {
            EventType::Broadcast
        } else {
            EventType::Emit
        };
        ctx.meta = payload.meta;
        ctx.level = payload.level;
        ctx.tracing = payload.tracing;
        ctx.parent_id = payload.parentID;
        ctx.request_id = Some(payload.requestID);
        ctx.caller = Some(payload.caller);
        ctx.node_id = Some(payload.sender);
        let (send, recv) = oneshot::channel::<bool>();
        let send_res = self
            .broker_sender
            .send(ServiceBrokerMessage::EmitLocalServices {
                ctx,
                result_channel: send,
            });
        if send_res.is_err() {
            return Ok(false);
        }
        match recv.await {
            Ok(value) => Ok(value),
            Err(_) => Ok(false),
        }
    }

    async fn request_handler(&self, payload: PayloadRequest) -> anyhow::Result<()> {
        debug!(
            "<= Request '{}' received from '{}' node.",
            payload.action, payload.sender
        );

        if !self.broker.started {
            warn!(
                "Incoming '{}' request from '{}' node is dropped because broker is stopped.",
                payload.action, payload.sender
            );
            bail!(ServiceBrokerError::ServiceNotAvailable {
                action_name: payload.action,
                node_id: self.node_id.clone()
            })
        }
        //TODO: check for stream dont't whats that for now
        let action_name = payload.action.clone();
        let service = utils::service_from_action(&action_name);
        let mut ctx = Context::new(self.broker.as_ref(), service);
        ctx.id = payload.id;
        //TODO: ctx.setParams
        ctx.parent_id = payload.parent_id;
        ctx.request_id = Some(payload.request_id);
        ctx.caller = Some(payload.caller);
        ctx.meta = payload.meta;
        ctx.level = payload.level;
        ctx.tracing = payload.tracing;
        ctx.node_id = Some(payload.sender);
        if payload.timeout.is_some() {
            ctx.options.timeout = payload.timeout
        }

        let endpoint = self
            .broker
            .get_local_action_endpoint(&payload.action, &ctx).await?;
        let endpoint = endpoint.clone();
        ctx.set_endpoint(&endpoint, Some(action_name), None);
        let params = payload.params;
        let result = task::spawn_blocking(move || (endpoint.action.handler)(ctx, params)).await?;
        todo!("handler sending response");
        Ok(())
    }

    pub(crate)async fn response_handler(&mut self, packet: PayloadResponse) {
        let id = packet.id;
        let req = self.pending_requests.get_mut(&id);
        match req {
            Some(req) => {
                debug!(
                    "<= Response {} is received from {}.",
                    req.action, packet.sender
                );
                req.ctx.node_id = Some(packet.sender);
                //TODO: merge meta
                self.remove_pending_request(&id);
                if !packet.success {
                    //TODO:Call error
                } else {
                    //TODO: (req.resolve)(packet.data)
                }
            }
            None => {
                debug!("Orphan response is received. Maybe the request is timed out earlier. ID:{} , Sender:{}" , id , packet.sender);
                //TODO: update the metrics
                return;
            }
        }
    }

    async fn request(&mut self, ctx: Context, resolve: fn(HandlerResult)) -> anyhow::Result<()> {
        if self.opts.max_queue_size.is_some() {
            if self.pending_requests.len() >= self.opts.max_queue_size.unwrap() {
                //TODO: Proper error handling
                bail!("Max queue size reached")
            }
        }
        self._send_request(ctx, resolve).await;
        Ok(())
    }

    async fn _send_request(&mut self, ctx: Context, resolve: fn(HandlerResult)) {
        //TODO: handle streaming response
        let node_id = ctx.node_id.to_owned();
        let action = ctx.action().to_string();

        let request = Request {
            node_id: node_id.clone(),
            action: action.clone(),
            ctx: ctx.clone(),
            resolve,
        };
        let id = ctx.id;
        let payload = PayloadRequest {
            action: action.clone(),
            sender: self.node_id.clone(),
            id: id.clone(),
            parent_id: ctx.parent_id,
            request_id: ctx.request_id.unwrap(),
            caller: ctx.caller.unwrap(),

            level: ctx.level,
            tracing: ctx.tracing,
            meta: ctx.meta,
            timeout: ctx.options.timeout,
            params: ctx.params,
            stream: false,
            seq: 0,
        };
        let packet = Packet::new(P::Request, node_id.clone(), payload);
        let node_name = match node_id {
            Some(node_id) => node_id,
            None => "someone".to_string(),
        };
        debug!("=> Send {} request to {} node", action, node_name);

        self.pending_requests.insert(id, request);
        let result = self.publish(packet).await;
        if result.is_err() {
            let err = result.err().unwrap();
            let message = format!(
                "Unable to send {} request to {} node. {} ",
                action, node_name, err
            );
            self.publish_error(err, FAILED_SEND_REQUEST_PACKET, message);
        }
    }

    ///Send an event to a remote node.
    /// The event is balanced by transporter
    pub(crate)async fn send_event(
        &self,
        ctx: Context,
        endpoint: Option<EventEndpoint>,
        params: Payload,
    ) -> anyhow::Result<()> {
        let groups = match ctx.event_groups {
            Some(groups) => groups,
            None => bail!("No event groups present"),
        };
        let event_name = match ctx.event_name {
            Some(name) => name,
            None => bail!("No event name present"),
        };

        match endpoint {
            Some(ep) => {
                debug!(
                    "=> Send '{}' event to '{}' node {:?}.",
                    event_name,
                    ep.node(),
                    groups
                )
            }
            None => debug!("=> Send '{}' event to '{:?}'.", event_name, groups),
        }
        let is_braodcast = match ctx.event_type {
            EventType::Broadcast => true,
            EventType::Emit => false,
            EventType::BroadcastLocal => false,
        };
        let payload_event = PayloadEvent {
            id: ctx.id,
            event: event_name.clone(),
            data: params,
            groups: groups,
            broadcast: is_braodcast,
            meta: ctx.meta,
            level: ctx.level,
            tracing: ctx.tracing,
            parentID: ctx.parent_id,
            requestID: ctx
                .request_id
                .expect("No request id present in the context"),
            caller: ctx.caller.expect("No caller present in the context"),
            needAck: ctx.need_ack,
            sender: self.node_id.clone(),
        };
        let packet = Packet::new(P::Event, ctx.node_id, payload_event);
        let result = self.publish(packet).await;
        if result.is_err() {
            let err = result.unwrap_err();
            let message = format!("Unable to send {} event to groups. {}", event_name, err);
            self.publish_error(err, FAILED_SEND_EVENT_PACKET, message);
        }
        Ok(())
    }

    fn remove_pending_request(&mut self, id: &str) {
        self.pending_requests.remove(id);
    }

    fn remove_pending_request_by_node(&mut self, node_id: &str) {
        debug!("Remove pending requests of {} node.", node_id);
        self.pending_requests.retain(|key, value| {
            if value.node_id.is_some() && value.node_id.as_ref().unwrap() == node_id {
                //TODO: add the req.reject error
                return false;
            }
            return true;
        });
        todo!("remove from res stream");
        todo!("remove from req streams")
    }

    fn send_response(
        &self,
        node_id: String,
        id: String,
        meta: Payload,
        data: Option<HandlerResult>,
        err: Option<String>,
    ) {
        let payload = {};
        if err.is_some() {
            //TODO:
            todo!("add error to the payload");
        }
    }

   pub(crate)async fn discover_nodes(&self) {
        let packet = Packet::new(P::Discover, None, PayloadNull {});
        let result = self.publish(packet).await;
        if result.is_err() {
            let err = result.unwrap_err();
            let message = format!("Unable to send DISCOVER packet. {}", err);
            self.publish_error(err, FAILED_NODES_DISCOVERY, message);
        }
    }
   pub(crate)async fn discover_node(&self, node_id: String) {
        let packet = Packet::new(P::Discover, Some(node_id.clone()), PayloadNull {});
        let result = self.publish(packet).await;
        if result.is_err() {
            let err = result.unwrap_err();
            let message = format!(
                "Unable to send DISCOVER packet to {} node. {}",
                node_id, err
            );
            self.publish_error(err, FAILED_NODES_DISCOVERY, message);
        }
    }
    pub(crate)async fn send_node_info(&self, info: NodeRawInfo, node_id: Option<String>) {
        if !self.conntected || !self.is_ready {
            return;
        }
        let payload = PayloadInfo {
            sender: todo!(),
            services: todo!(),
            config: todo!(),
            ip_list: todo!(),
            hostame: todo!(),
            client: todo!(),
            seq: todo!(),
            instance_id: todo!(),
            meta_data: todo!(),
        };
        let packet = Packet::new(P::Info, node_id, payload);
        todo!("")
    }
    async fn send_ping(&self, node_id: Option<String>, id: Option<String>) {
        let id = match id {
            Some(id) => id,
            None => utils::generate_uuid(),
        };
        let data = json!({"time" :Local::now().to_rfc3339() , "id" : id  });
        let payload = PayloadPing {
            sender: self.node_id.clone(),
            time: Local::now().to_rfc3339(),
            id,
        };

        let packet = Packet::new(P::Ping, node_id.clone(), payload);
        let result = self.publish(packet).await;
        if result.is_err() {
            let err = result.unwrap_err();
            let node_id = match node_id {
                Some(id) => id,
                None => "".to_string(),
            };
            let message = format!("Unable to send PING packet to {} node. {}", node_id, err);
            self.publish_error(err, FAILED_SEND_PING_PACKET, message);
        }
    }

    async fn send_pong(&self, payload: PayloadPing) -> anyhow::Result<()> {
        let pong_payload = PayloadPong {
            sender: self.node_id.clone(),
            time: payload.time,
            arrived: Local::now().to_rfc3339(),
            id: payload.id,
        };
        let packet = Packet::new(P::Pongs, Some(payload.sender.clone()), pong_payload);
        let result = self.publish(packet).await;
        if result.is_err() {
            let err = result.unwrap_err();
            let message = format!(
                "Unable to send PONG packet to {} node. {}",
                payload.sender, err
            );
            self.publish_error(err, FAILED_SEND_PONG_PACKET, message);
        }

        Ok(())
    }

    async fn process_pong(&self, payload: PayloadPong) -> anyhow::Result<()> {
        let now = Local::now();

        let time = self.extract_time_from_payload(&payload.time)?;
        let elapsed_time = now - time;
        let arrived_time = self.extract_time_from_payload(&payload.arrived)?;
        let time_diff = (now - arrived_time - elapsed_time / 2).num_milliseconds();
        let elapsed_time = elapsed_time.num_milliseconds();

        let data = json!({"nodeID" : payload.sender ,"elapsedTime" :elapsed_time , "timeDiff":time_diff , "id": payload.id });

        let _ = self
            .broker_sender
            .send(ServiceBrokerMessage::BroadcastLocal {
                event_name: TransporterEvents::Pong.to_string(),
                data,
                opts: Value::Null,
            });
        Ok(())
    }

    pub(crate)async fn send_heartbeat(&self, local_node_cpu: u32) {
        let payload = PayloadHeartbeat {
            cpu: local_node_cpu,
            sender: self.node_id.clone(),
        };
        let packet = Packet::new(P::Heartbeat, None, payload);
        let result = self.publish(packet).await;
        if result.is_err() {
            let error = result.unwrap_err();
            let message = format!("Unable to send HEARTBEAT packet. {}", error);
            self.publish_error(error, FAILED_TO_SEND_HEARTBEAT, message);
        }
    }

    async fn subscribe(&self, topic: String, node_id: String) -> anyhow::Result<()> {
        self.tx
            .subscibe(Topic {
                node_id: Some(node_id),
                cmd: topic,
            })
            .await
    }

    async fn publish<P: PacketPayload>(&self, packet: Packet<P>) -> anyhow::Result<()> {
        todo!()
    }

    fn packet_topic(&self, packet_type: PacketType) -> Topic {
        Topic {
            cmd: packet_type.into(),
            node_id: Some(self.node_id.clone()),
        }
    }
    fn packet_topic_witout_node_id(&self, packet_type: PacketType) -> Topic {
        Topic {
            cmd: packet_type.into(),
            node_id: None,
        }
    }

    fn publish_error(&self, error: anyhow::Error, tipe: &str, message: String) {
        error!("{}", message);
        let err = error.to_string();
        let data = json!({
        "error": err,
        "module":"transit",
        "type":tipe
        }
        );
        let _ = self
            .broker_sender
            .send(ServiceBrokerMessage::BroadcastLocal {
                event_name: TransporterEvents::Error.to_string(),
                data,
                opts: Value::Null,
            });
    }
    fn extract_time_from_payload(&self, value: &str) -> anyhow::Result<DateTime<Local>> {
        match DateTime::parse_from_rfc3339(value) {
            Ok(time) => Ok(DateTime::from(time)),
            Err(_) => bail!(PacketError::CannotParse(
                "Cannot extract time from the packet.".to_string()
            )),
        }
    }
}

#[derive(Debug, Display)]
enum TransporterEvents {
    #[display(fmt = "$transporter.disconnected")]
    Disconnected,
    #[display(fmt = "$transit.error")]
    Error,
    #[display(fmt = "$node.pong")]
    Pong,
}

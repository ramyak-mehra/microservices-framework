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
    registry::{EndpointTrait, EventEndpoint, Node, Payload},
    utils, HandlerResult,
};
use crate::{ServiceBroker, ServiceBrokerMessage};
use anyhow::{self, bail};
use tokio::{sync::mpsc, task};

type P = PacketType;
struct TransitOptions {}

struct Request {
    node_id: String,
}

pub(crate) struct Transit<T: Transporter + Send + Sync> {
    reciever: mpsc::UnboundedReceiver<TransitMessage>,
    broker: Arc<ServiceBroker>,
    broker_sender: mpsc::UnboundedSender<ServiceBrokerMessage>,
    tx: T,
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
        reciever: mpsc::UnboundedReceiver<TransitMessage>,
    ) -> Self {
        let broker = broker.clone();
        let node_id = broker.node_id.clone();
        let instance_id = broker.instance.clone();

        Self {
            reciever,
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
    fn send_disconnect_packet() {
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

    fn message_handler<P: PacketPayload>(&self, cmd: String, packet: Packet<P>) {
        todo!("implement payload parsing first")
    }

    fn event_handler(&self) {
        todo!("implement the payload parsing first")
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
        ctx.parent_id = Some(payload.parent_id);
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
            .get_local_action_endpoint(&payload.action, &ctx)?;
        let endpoint = endpoint.clone();
        ctx.set_endpoint(&endpoint, Some(action_name), None);
        let params = payload.params;
        let result =
            task::spawn_blocking(move || (endpoint.action.handler)(ctx, Some(params))).await?;
        todo!("handler sending response");
        Ok(())
    }

    ///Send an event to a remote node.
    /// The event is balanced by transporter
    pub async fn send_event(
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
        };
        let packet = Packet::new(P::Event, ctx.node_id, payload_event);
        let result = self.publish(packet).await;
        if result.is_err() {
            let err = result.unwrap_err();
            let message = format!("Unable to send {} event to groups. {}", event_name, err);
            self.send_error(err, FAILED_SEND_EVENT_PACKET, message);
        }
        Ok(())
    }

    fn remove_pending_request(&mut self, id: &str) {
        self.pending_requests.remove(id);
    }

    fn remove_pending_request_by_node(&mut self, node_id: &str) {
        debug!("Remove pending requests of {} node.", node_id);
        self.pending_requests.retain(|key, value| {
            if value.node_id == node_id {
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

    async fn discover_nodes(&self) {
        let packet = Packet::new(P::Discover, None, PayloadNull {});
        let result = self.publish(packet).await;
        if result.is_err() {
            let err = result.unwrap_err();
            let message = format!("Unable to send DISCOVER packet. {}", err);
            self.send_error(err, FAILED_NODES_DISCOVERY, message);
        }
    }
    async fn discover_node(&self, node_id: String) {
        let packet = Packet::new(P::Discover, Some(node_id.clone()), PayloadNull {});
        let result = self.publish(packet).await;
        if result.is_err() {
            let err = result.unwrap_err();
            let message = format!(
                "Unable to send DISCOVER packet to {} node. {}",
                node_id, err
            );
            self.send_error(err, FAILED_NODES_DISCOVERY, message);
        }
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
            self.send_error(err, FAILED_SEND_PING_PACKET, message);
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
            self.send_error(err, FAILED_SEND_PONG_PACKET, message);
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

    async fn send_heartbeat(&self, local_node: &Node) {
        let payload = PayloadHeartbeat {
            cpu: local_node.cpu,
        };
        let packet = Packet::new(P::Heartbeat, None, payload);
        let result = self.publish(packet).await;
        if result.is_err() {
            let error = result.unwrap_err();
            let message = format!("Unable to send HEARTBEAT packet. {}", error);
            self.send_error(error, FAILED_TO_SEND_HEARTBEAT, message);
        }
    }

    async fn subscibe(&self, topic: String, node_id: String) -> anyhow::Result<()> {
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

    fn send_error(&self, error: anyhow::Error, tipe: &str, message: String) {
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

enum TransitMessage {}

#[derive(Debug, Display)]
enum TransporterEvents {
    #[display(fmt = "$transporter.disconnected")]
    Disconnected,
    #[display(fmt = "$transit.error")]
    Error,
    #[display(fmt = "$node.pong")]
    Pong,
}

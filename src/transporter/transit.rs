use std::{collections::HashMap, sync::Arc, time::Duration};

use chrono::{DateTime, Local};
use derive_more::Display;
use log::{debug, error, info, warn};
use serde_json::{json, Value};
use tokio_js_set_interval::{_set_timeout, set_timeout};

use super::*;
use crate::{
    broker_delegate::BrokerDelegate,
    constants::*,
    context::{Context, EventType},
    errors::{PacketError, ServiceBrokerError},
    registry::{
        node::NodeRawInfo, Action, DiscovererMessage, EndpointTrait, EventEndpoint, Node, Payload,
    },
    utils, BrokerSender, DiscovererSender, HandlerResult,
};
use crate::{ServiceBroker, ServiceBrokerMessage};
use anyhow::{self, bail};
use tokio::{
    sync::{mpsc, oneshot, RwLock},
    task,
};

type P = PacketType;
struct TransitOptions {
    max_queue_size: Option<usize>,
    disable_reconnect: bool,
}

struct Request {
    node_id: Option<String>,
    action: String,
    ctx: Context,
    resolve: fn(HandlerResult),
}
type PendingRequests = Arc<RwLock<HashMap<String, Request>>>;
pub(crate) struct Transit<T: Transporter + Send + Sync> {
    broker: Arc<BrokerDelegate>,
    broker_sender: BrokerSender,
    pub(crate) tx: T,
    opts: TransitOptions,
    node_id: String,
    instance_id: String,
    discoverer_sender: DiscovererSender,
    conntected: bool,
    disconnecting: bool,
    is_ready: bool,
    broker_started: bool,
    pending_requests: PendingRequests,
}

impl<T: Transporter + Send + Sync> Transit<T> {
    fn new(
        broker: Arc<BrokerDelegate>,
        opts: TransitOptions,
        transporter: T,
        broker_sender: BrokerSender,
        discoverer_sender: DiscovererSender,
    ) -> Self {
        let node_id = broker.node_id().to_owned();
        let instance_id = broker.instance_id().to_owned();

        Self {
            broker,
            broker_started: false,
            tx: transporter,
            opts,
            node_id,
            instance_id,
            broker_sender,
            conntected: false,
            disconnecting: false,
            is_ready: false,
            discoverer_sender,
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    /// It will be called after transporter connected or reconnected.
    async fn after_connect(&mut self, was_reconnect: bool) {
        if was_reconnect {
            // After reconnecting, we should send a broadcast INFO packet because there may be some new nodes.
            // In case of disabled balancer, it triggers the `make_balanced_subscriptions method`.
            let (sender_ch, recv) = oneshot::channel();
            let _ = self
                .discoverer_sender
                .send(DiscovererMessage::SendLocalNodeInfo(None, sender_ch));
            let _ = recv.await;
        } else {
            self.make_subsciptions().await;
        }
        let (sender_ch, recv) = oneshot::channel();
        let _ = self
            .discoverer_sender
            .send(DiscovererMessage::DiscoverAllNodes(sender_ch));
        let _ = recv.await;
        tokio::time::sleep(Duration::from_millis(500)).await;
        self.conntected = true;
        let data = json!({
            "wasReconnect": !!was_reconnect,
        });
        let _ = self
            .broker_sender
            .send(ServiceBrokerMessage::BroadcastLocal {
                event_name: TransporterEvents::Connected.to_string(),
                data,
                opts: Value::Null,
            });
    }

    async fn connect(&mut self) {
        info!("Connecting to the transporter...");
        let mut reconnected_started = false;
        let result = self.tx.connect().await;
        if let Err(err) = result {
            if self.disconnecting {
                return;
            }
            if reconnected_started {
                return;
            }
            warn!("Connection is failed. {}", err);
            debug!("{}", err);
            if self.opts.disable_reconnect {
                return;
            }
            reconnected_started = true;

            _set_timeout(
                || {
                    info!("Reconnecting...");
                    let _ = self.tx.connect();
                },
                5 * 1000,
            )
            .await;
        };
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
            let (sender, mut recv) = oneshot::channel();
            let result = self
                .discoverer_sender
                .send(DiscovererMessage::LocalNodeDisconnect(sender));
            let _ = recv.await;
            self.tx.disconnect().await;
            self.disconnecting = false;
        }
    }
    /// Local broker is ready (all services loaded).
    /// Send INFO packet to all other nodes
    fn ready(&mut self) {
        if self.conntected {
            self.is_ready = true;

            let _ = self
                .discoverer_sender
                .send(DiscovererMessage::LocalNodeReady);
        }
    }
    ///Send DISCONNECT to remote nodes
    pub(crate) async fn send_disconnect_packet(node_id: String) {
        let packet = Packet::new(P::Disconnect, None, PayloadDisconnect { sender: node_id });
        let result = Transit::<T>::publish(packet).await;
        if let Err(err) = result {
            debug!("Unable to send DISCONNECT packet. {}", err);
        }
    }
    async fn make_subsciptions(&self) {
        let topics = vec![
            packet_topic(&self.node_id, P::Event),
            packet_topic(&self.node_id, P::Request),
            packet_topic(&self.node_id, P::Response),
            //Discover handler
            packet_topic(&self.node_id, P::Discover),
            packet_topic_witout_node_id(P::Discover),
            //NodeInfo handler
            packet_topic(&self.node_id, P::Info), //Broadcast INFO. If a new node connected.
            packet_topic_witout_node_id(P::Info), //Resposne INFO to DISCOVER packet.
            // Disconnect handler
            packet_topic_witout_node_id(P::Disconnect),
            //Hearbeat handler
            packet_topic_witout_node_id(P::Heartbeat),
            //Ping handler
            packet_topic_witout_node_id(P::Ping), //Broadcasted
            packet_topic(&self.node_id, P::Ping), //Targeted
            //Pong handler
            packet_topic(&self.node_id, P::Pongs),
        ];
        self.tx.make_subsciptions(topics).await;
        todo!("return from the make subscriptions");
    }

    async fn message_handler<P: 'static + PacketPayload + Send>(
        &self,
        cmd: PacketType,
        packet: Packet<P>,
    ) {
        let payload = packet.payload;
        let broker_sender = self.broker_sender.clone();
        let node_id = self.node_id.to_owned();
        let broker = Arc::clone(&self.broker);
        let broker_started = self.broker_started;
        let pending_requests = Arc::clone(&self.pending_requests);
        let instance_id = self.instance_id.to_owned();
        let discoverer_sender = self.discoverer_sender.clone();
        tokio::spawn(async move {
            //TODO: check for empty payload i.e PayloadNull
            // TODO: Check protocol version

            if payload.sender() == node_id {
                if cmd == PacketType::Info && payload.instance_id() != instance_id {
                    panic!("Service Broker has detected a nodeID conflict, use unique nodeIDS. ServiceBroker has stopped.")
                }
                if cmd != PacketType::Event
                    && cmd != PacketType::Request
                    && cmd != PacketType::Response
                {
                    return;
                }
            }

            match cmd {
                PacketType::Unknown => todo!(),
                PacketType::Event => {
                    let result = Transit::<T>::event_handler(
                        broker_sender,
                        payload.event_payload(),
                        broker,
                        broker_started,
                    )
                    .await;
                }
                PacketType::Request => {
                    let result = Transit::<T>::request_handler(
                        node_id,
                        payload.request_paylaod(),
                        broker,
                        broker_started,
                    )
                    .await;
                }
                PacketType::Response => {
                    let result = Transit::<T>::response_handler(
                        payload.response_payload(),
                        pending_requests,
                    )
                    .await;
                }
                PacketType::Discover => todo!(),
                PacketType::Info => todo!(),
                PacketType::Disconnect => {
                    let result =
                        discoverer_sender.send(DiscovererMessage::RemoteNodeDisconnected {
                            sender: payload.sender().to_string(),
                            is_unexpected: false,
                        });
                }
                PacketType::Heartbeat => {
                    let result = discoverer_sender.send(DiscovererMessage::HeartbeatRecieved {
                        sender: payload.sender().to_string(),
                        heartbeat_payload: payload.heartbeat_payload(),
                    });
                }
                PacketType::Ping => {
                    let result =
                        Transit::<T>::send_pong(broker_sender, node_id, payload.ping_payload())
                            .await;
                }
                PacketType::Pongs => {
                    let result =
                        Transit::<T>::process_pong(broker_sender, payload.pong_payload()).await;
                }
                PacketType::GossipReq => todo!(),
                PacketType::GossipRes => todo!(),
                PacketType::GossipHello => todo!(),
            }
        });
    }

    async fn event_handler(
        broker_sender: BrokerSender,
        payload: PayloadEvent,
        broker: Arc<BrokerDelegate>,
        broker_started: bool,
    ) -> anyhow::Result<bool> {
        debug!(
            "Event {} received from {} node {:?}",
            payload.event, payload.sender, payload.groups
        );
        if !broker_started {
            warn!(
                "Incoming {} event from {} node is dropped, because broker is stopped",
                payload.event, payload.sender
            );
            return Ok(false);
        }
        //TODO: handle service name not present
        let mut ctx = Context::new(broker.get_broker(), "".to_string());
        ctx.id = payload.id;
        ctx.event_name = Some(payload.event);

        ctx.event_groups = Some(payload.groups);
        ctx.set_params(payload.data);
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
        let send_res = broker_sender.send(ServiceBrokerMessage::EmitLocalServices {
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

    async fn request_handler(
        node_id: String,
        payload: PayloadRequest,
        broker: Arc<BrokerDelegate>,
        broker_started: bool,
    ) -> anyhow::Result<()> {
        debug!(
            "<= Request '{}' received from '{}' node.",
            payload.action, payload.sender
        );

        if !broker_started {
            warn!(
                "Incoming '{}' request from '{}' node is dropped because broker is stopped.",
                payload.action, payload.sender
            );
            bail!(ServiceBrokerError::ServiceNotAvailable {
                action_name: payload.action,
                node_id: node_id
            })
        }
        //TODO: check for stream dont't whats that for now
        let action_name = payload.action.clone();
        let service = utils::service_from_action(&action_name);
        let mut ctx = Context::new(broker.get_broker(), service);
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

        let endpoint = broker
            .get_local_action_endpoint(&payload.action, &ctx)
            .await?;
        let endpoint = endpoint.clone();
        ctx.set_endpoint(&endpoint, Some(action_name), None);
        let params = payload.params;
        let result = task::spawn_blocking(move || (endpoint.action.handler)(ctx)).await?;
        todo!("handler sending response");
        Ok(())
    }

    async fn response_handler(payload: PayloadResponse, pending_requests: PendingRequests) {
        let id = payload.id;
        let mut _pending_requests = pending_requests.write().await;
        let req = _pending_requests.get_mut(&id);
        match req {
            Some(req) => {
                debug!(
                    "<= Response {} is received from {}.",
                    req.action, payload.sender
                );
                req.ctx.node_id = Some(payload.sender);
                //TODO: merge meta
                Transit::<T>::remove_pending_request(pending_requests.clone(), &id);
                if !payload.success {
                    //TODO:Call error
                } else {
                    //TODO: (req.resolve)(packet.data)
                }
            }
            None => {
                debug!("Orphan response is received. Maybe the request is timed out earlier. ID:{} , Sender:{}" , id , payload.sender);
                //TODO: update the metrics
                return;
            }
        }
    }

    async fn request(
        broker_sender: BrokerSender,
        node_id: String,
        ctx: Context,
        resolve: fn(HandlerResult),
        pending_requests: PendingRequests,
        opts: Arc<TransitOptions>,
    ) -> anyhow::Result<()> {
        if opts.max_queue_size.is_some() {
            if pending_requests.read().await.len() >= opts.max_queue_size.unwrap() {
                //TODO: Proper error handling
                bail!("Max queue size reached")
            }
        }
        Transit::<T>::_send_request(broker_sender, node_id, ctx, resolve, pending_requests).await;
        Ok(())
    }

    async fn _send_request(
        broker_sender: BrokerSender,
        self_node_id: String,
        ctx: Context,
        resolve: fn(HandlerResult),
        pending_requests: PendingRequests,
    ) {
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
            sender: self_node_id,
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

        {
            pending_requests.write().await.insert(id, request);
        }
        let result = Transit::<T>::publish(packet).await;
        if let Err(err) = result {
            let message = format!(
                "Unable to send {} request to {} node. {} ",
                action, node_name, err
            );
            publish_error(broker_sender, err, FAILED_SEND_REQUEST_PACKET, message);
        }
    }

    ///Send an event to a remote node.
    /// The event is balanced by transporter
    async fn send_event(
        broker_sender: BrokerSender,
        node_id: String,
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
            sender: node_id,
        };
        let packet = Packet::new(P::Event, ctx.node_id, payload_event);
        let result = Transit::<T>::publish(packet).await;
        if let Err(err) = result {
            let message = format!("Unable to send {} event to groups. {}", event_name, err);
            publish_error(broker_sender, err, FAILED_SEND_EVENT_PACKET, message);
        }
        Ok(())
    }

    async fn remove_pending_request(pending_requests: PendingRequests, id: &str) {
        pending_requests.write().await.remove(id);
    }

    async fn remove_pending_request_by_node(pending_requests: PendingRequests, node_id: &str) {
        debug!("Remove pending requests of {} node.", node_id);
        pending_requests.write().await.retain(|key, value| {
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
        node_id: String,
        self_node_id: String,
        id: String,
        meta: Payload,
        data: Option<HandlerResult>,
        err: Option<String>,
    ) {
        if let Some(result) = data {            
            let payload = PayloadResponse {
                sender: self_node_id,
                id,
                success: err.is_none(),
                data: todo!(),
                error: "".to_string(),
                meta: todo!(),
                stream: false,
                seq: 0,
            };
        }
        if err.is_some() {
            //TODO:
            todo!("add error to the payload");
        }
    }

    async fn discover_nodes(broker_sender: BrokerSender, node_id: String) {
        let packet = Packet::new(P::Discover, None, PayloadDiscover { sender: node_id });
        let result = Transit::<T>::publish(packet).await;
        if let Err(err) = result {
            let message = format!("Unable to send DISCOVER packet. {}", err);
            publish_error(broker_sender, err, FAILED_NODES_DISCOVERY, message);
        }
    }
    async fn discover_node(broker_sender: BrokerSender, self_node_id: String, node_id: String) {
        let packet = Packet::new(
            P::Discover,
            Some(node_id.clone()),
            PayloadDiscover {
                sender: self_node_id,
            },
        );
        let result = Transit::<T>::publish(packet).await;
        if let Err(err) = result {
            let message = format!(
                "Unable to send DISCOVER packet to {} node. {}",
                node_id, err
            );
            publish_error(broker_sender, err, FAILED_NODES_DISCOVERY, message);
        }
    }
    async fn send_node_info(
        &self,
        broker_sender: BrokerSender,
        self_node_id: String,
        info: NodeRawInfo,
        node_id: Option<String>,
    ) {
        if !self.conntected || !self.is_ready {
            return;
        }

        let payload = PayloadInfo::from_node_info(info, self_node_id);
        let packet = Packet::new(P::Info, node_id.clone(), payload);
        let result = Transit::<T>::publish(packet).await;
        if let Err(err) = result {
            let node_id = match node_id {
                Some(id) => id,
                None => "".to_string(),
            };
            let message = format!("Unable to send INFO packet to {} node. {}", node_id, err);
            publish_error(broker_sender, err, FAILED_SEND_INFO_PACKET, message);
        }
    }
    async fn send_ping(
        broker_sender: BrokerSender,
        self_node_id: String,
        node_id: Option<String>,
        id: Option<String>,
    ) {
        let id = match id {
            Some(id) => id,
            None => utils::generate_uuid(),
        };
        let data = json!({"time" :Local::now().to_rfc3339() , "id" : id  });
        let payload = PayloadPing {
            sender: self_node_id,
            time: Local::now().to_rfc3339(),
            id,
        };

        let packet = Packet::new(P::Ping, node_id.clone(), payload);
        let result = Transit::<T>::publish(packet).await;
        if let Err(err) = result {
            let node_id = match node_id {
                Some(id) => id,
                None => "".to_string(),
            };
            let message = format!("Unable to send PING packet to {} node. {}", node_id, err);
            publish_error(broker_sender, err, FAILED_SEND_PING_PACKET, message);
        }
    }

    async fn send_pong(
        broker_sender: BrokerSender,
        node_id: String,
        payload: PayloadPing,
    ) -> anyhow::Result<()> {
        let pong_payload = PayloadPong {
            sender: node_id.to_string(),
            time: payload.time,
            arrived: Local::now().to_rfc3339(),
            id: payload.id,
        };
        let packet = Packet::new(P::Pongs, Some(payload.sender.clone()), pong_payload);
        let result = Transit::<T>::publish(packet).await;
        if let Err(err) = result {
            let message = format!(
                "Unable to send PONG packet to {} node. {}",
                payload.sender, err
            );
            publish_error(broker_sender, err, FAILED_SEND_PONG_PACKET, message);
        }

        Ok(())
    }

    async fn process_pong(broker_sender: BrokerSender, payload: PayloadPong) -> anyhow::Result<()> {
        let now = Local::now();

        let time = extract_time_from_payload(&payload.time)?;
        let elapsed_time = now - time;
        let arrived_time = extract_time_from_payload(&payload.arrived)?;
        let time_diff = (now - arrived_time - elapsed_time / 2).num_milliseconds();
        let elapsed_time = elapsed_time.num_milliseconds();

        let data = json!({"nodeID" : payload.sender ,"elapsedTime" :elapsed_time , "timeDiff":time_diff , "id": payload.id });

        let _ = broker_sender.send(ServiceBrokerMessage::BroadcastLocal {
            event_name: TransporterEvents::Pong.to_string(),
            data,
            opts: Value::Null,
        });
        Ok(())
    }

    async fn send_heartbeat(broker_sender: BrokerSender, node_id: String, local_node_cpu: u32) {
        let payload = PayloadHeartbeat {
            cpu: local_node_cpu,
            sender: node_id,
        };
        let packet = Packet::new(P::Heartbeat, None, payload);
        let result = Transit::<T>::publish::<PayloadHeartbeat>(packet).await;
        if result.is_err() {
            let error = result.unwrap_err();
            let message = format!("Unable to send HEARTBEAT packet. {}", error);
            publish_error(broker_sender, error, FAILED_TO_SEND_HEARTBEAT, message);
        }
    }

    async fn subscribe(&self, topic: String, node_id: String) -> anyhow::Result<()> {
        self.tx.subscibe(topic, Some(node_id)).await
    }

    async fn publish<P: PacketPayload>(packet: Packet<P>) -> anyhow::Result<()> {
        todo!()
    }
}

#[derive(Debug, Display)]
enum TransporterEvents {
    #[display(fmt = "$transporter.disconnected")]
    Disconnected,
    #[display(fmt = "$transporter.connected")]
    Connected,
    #[display(fmt = "$transit.error")]
    Error,
    #[display(fmt = "$node.pong")]
    Pong,
}
pub(crate) enum TransitMessage {
    RecievedMessage {
        cmd: PacketType,
        data: Vec<u8>,
    },
    DiscoverNodes,
    DiscoverNode {
        node_id: String,
    },
    SendNodeInfo {
        info: NodeRawInfo,
        node_id: Option<String>,
    },
    MakeBalancedSubscription(oneshot::Sender<()>),
}
fn packet_topic(node_id: &str, packet_type: PacketType) -> Topic {
    Topic {
        cmd: packet_type.into(),
        node_id: Some(node_id.to_string()),
    }
}
fn packet_topic_witout_node_id(packet_type: PacketType) -> Topic {
    Topic {
        cmd: packet_type.into(),
        node_id: None,
    }
}

fn publish_error(broker_sender: BrokerSender, error: anyhow::Error, tipe: &str, message: String) {
    error!("{}", message);
    let err = error.to_string();
    let data = json!({
    "error": err,
    "module":"transit",
    "type":tipe
    }
    );
    let _ = broker_sender.send(ServiceBrokerMessage::BroadcastLocal {
        event_name: TransporterEvents::Error.to_string(),
        data,
        opts: Value::Null,
    });
}
fn extract_time_from_payload(value: &str) -> anyhow::Result<DateTime<Local>> {
    match DateTime::parse_from_rfc3339(value) {
        Ok(time) => Ok(DateTime::from(time)),
        Err(_) => bail!(PacketError::CannotParse(
            "Cannot extract time from the packet.".to_string()
        )),
    }
}

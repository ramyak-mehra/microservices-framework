pub mod nats;
pub mod transit;
use std::sync::Arc;

pub(crate) use crate::{errors::ServiceBrokerError, packet::*};
use crate::{serializers::BaseSerializer, ServiceBroker};
use anyhow::bail;
use async_trait::async_trait;
use lazy_static::lazy_static;
use regex::Regex;
use serde::Serialize;
pub(crate) use transit::Transit;
type PT = PacketType;
fn balanced_event_regex_replace(topic: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\*\*.*$").unwrap();
    }
    let result = RE.replace(topic, ">");
    result.into_owned()
}
#[async_trait]
pub(crate) trait Transporter {
    fn broker(&self) -> &Arc<ServiceBroker>;
    fn connected(&self) -> bool;
    fn prefix(&self) -> &String;
    async fn connect(&self);
    fn on_connected(&mut self, was_reconnect: bool);
    async fn disconnect(&self);

    async fn make_subsciptions(&self, topics: Vec<Topic>) -> anyhow::Result<()> {
        for topic in topics {
            self.subscibe(topic).await?;
        }
        Ok(())
    }

    async fn incoming_message(&self);
    //Received data. It's a wrapper for middlewares.
    async fn receive(&self, cmd: PacketType, data: Vec<u8>) {
        self.incoming_message().await;
    }

    async fn subscibe(&self, topic: Topic) -> anyhow::Result<()>;
    async fn subscibe_balanced_request(&self) -> anyhow::Result<()>;
    async fn subscibe_balanced_event(&self) -> anyhow::Result<()>;

    async fn pre_publish<P: PacketPayload + Send + Copy + Serialize>(
        &self,
        packet: Packet<P>,
    ) -> anyhow::Result<()> {
        //Safely handle disconnected state.
        if !Self::connected(self) {
            //For packets that are triggered intentionally by users, throws a retryable error.
            let not_valid = vec![PT::Request, PT::Event, PT::Ping];
            if not_valid.contains(&packet.tipe) {
                bail!(ServiceBrokerError::BrokerDisconnectedError)
            } else {
                return Ok(());
            }
        }
        let payload = packet.payload;
        if packet.tipe == PT::Event && packet.target.is_none() && payload.tipe() == PT::Event {
            let payload = payload.event_payload()?;

            let groups = payload.groups.clone();
            // If the packet contains groups, we don't send the packet to
            // the targetted node, but we push them to the event group queues
            // and AMQP will load-balanced it.
            if !groups.is_empty() {
                groups.iter().for_each(|group| {
                    let mut payload_copy = payload.clone();
                    //Change the groups to this group to avoid multi handling in consumers.
                    payload_copy.groups = vec![group.clone()];
                    let packet_copy = Packet::new(PT::Event, None, payload_copy);

                    //TODO: make is parallel
                    self.publish_balanced_event(packet_copy, group.clone());
                });
                return Ok(());
            }
        } else if packet.tipe == PT::Request && packet.target.is_none() {
            let payload = payload.request_paylaod()?;
            let request_packet = packet.from_payload::<PayloadRequest>(payload);
            let _ = self.publish_balanced_request(request_packet).await?;
            return Ok(());
        }

        self.publish(packet).await?;
        Ok(())
    }

    async fn publish<P: PacketPayload + Send + Serialize>(
        &self,
        packet: Packet<P>,
    ) -> anyhow::Result<()> {
        let topic = self.get_topic_name(&packet.tipe.to_string(), &packet.target);

        let data = self.serialize(packet);
        self.send(topic, data, None).await
    }
    async fn publish_balanced_event(
        &self,
        packet: Packet<PayloadEvent>,
        group: String,
    ) -> anyhow::Result<()> {
        let topic = format!(
            "{}.{}B.{}.{}",
            self.prefix(),
            PT::Event,
            group,
            packet.payload.event
        );
        let data = self.serialize(packet);
        let meta = TransporterMeta { balanced: true };
        self.send(topic, data, Some(meta)).await
    }
    async fn publish_balanced_request(&self, packet: Packet<PayloadRequest>) -> anyhow::Result<()> {
        let topic = format!(
            "{}.{}B.{}",
            self.prefix(),
            PT::Request,
            packet.payload.action
        );
        let data = self.serialize(packet);
        let meta = TransporterMeta { balanced: true };
        self.send(topic, data, Some(meta)).await
    }
    async fn send(
        &self,
        topic: String,
        data: String,
        meta: Option<TransporterMeta>,
    ) -> anyhow::Result<()>;
    fn get_topic_name(&self, cmd: &str, node_id: &Option<String>) -> String {
        let prefix = self.prefix();
        let mut topic_name = format!("{}.{}", prefix, cmd);

        if let Some(node_id) = node_id {
            topic_name = format!("{}.{}", topic_name, node_id);
        };
        topic_name
    }
    fn serialize<P: PacketPayload + Send + Serialize>(&self, packet: Packet<P>) -> String {
        let serializer = self.broker().serializer();
        //TODO: handle the error
        let data = serializer.serialize(packet).unwrap();
        data
    }
}

pub(crate) struct Topic {
    cmd: String,
    node_id: Option<String>,
}
pub(crate) struct TransporterMeta {
    balanced: bool,
}
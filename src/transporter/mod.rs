pub mod transit;

use anyhow::bail;
use async_trait::async_trait;

use crate::packet;
pub(crate) use crate::{errors::ServiceBrokerError, packet::*};
type PT = PacketType;

#[async_trait]
trait Transporter {
    fn connected(&self) -> bool;
    fn prefix(&self) -> String;
    async fn disconnect(&self);
    async fn make_subsciptions(&self, topics: Vec<Topic>);
    async fn subscibe(&self, topic: String, node_id: String);
    async fn pre_publish<P: PacketPayload + Send + Copy>(
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

    async fn publish_balanced_event(&self, packet: Packet<PayloadEvent>, group: String);
    async fn publish_balanced_request(&self, packet: Packet<PayloadRequest>) -> anyhow::Result<()>;
    async fn publish<P: PacketPayload + Send>(&self, packet: Packet<P>) -> anyhow::Result<()> {
        let topic = self.get_topic_name(&packet);
        //TODO: get serialized data
        Ok(())
    }
    fn get_topic_name<P: PacketPayload>(&self, packet: &Packet<P>) -> String {
        let prefix = self.prefix();
        let mut topic_name = format!("{}.{}", prefix, packet.tipe.to_string());

        if let Some(node_id) = &packet.target {
            topic_name = format!("{}.{}", topic_name, node_id);
        };
        topic_name
    }
}

struct Topic {
    cmd: String,
    node_id: Option<String>,
}

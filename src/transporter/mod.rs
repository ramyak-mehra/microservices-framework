pub mod transit;

use anyhow::bail;
use async_trait::async_trait;

use crate::{
    errors::ServiceBrokerError,
    packet::{self, Packet, PacketPayload, PacketType},
};
type P = PacketType;

#[async_trait]
trait Transporter {
    fn connected(&self) -> bool;
    fn prefix(&self) -> String;
    async fn disconnect(&self);
    async fn make_subsciptions(&self, topics: Vec<Topic>);
    async fn subscibe(&self, topic: String, node_id: String);
    async fn pre_publish(&self, packet: Packet) -> anyhow::Result<()> {
        //Safely handle disconnected state.
        if !Self::connected(self) {
            //For packets that are triggered intentionally by users, throws a retryable error.
            let not_valid = vec![P::Request, P::Event, P::Ping];
            if not_valid.contains(&packet.tipe) {
                bail!(ServiceBrokerError::BrokerDisconnectedError)
            } else {
                return Ok(());
            }
        }
        let payload = match &packet.payload {
            PacketPayload::FullPayload(payload) => payload,
            PacketPayload::Custom(_) => {
                bail!("Full packet payload expected.")
            }
        };
        if packet.tipe == P::Event && packet.target.is_none() && payload.groups.is_some() {
            let groups = payload.groups.as_ref().unwrap();
            // If the packet contains groups, we don't send the packet to
            // the targetted node, but we push them to the event group queues
            // and AMQP will load-balanced it.
            if !groups.is_empty() {
                groups.iter().for_each(|group| {
                    let mut payload_copy = payload.clone();
                    //Change the groups to this group to avoid multi handling in consumers.
                    payload_copy.groups = Some(vec![group.clone()]);
                    let packet_copy =
                        Packet::new(P::Event, None, PacketPayload::FullPayload(payload_copy));

                    //TODO: make is parallel
                    self.publish_balanced_event(packet_copy, group.clone());
                });
                return Ok(());
            }
        } else if packet.tipe == P::Request && packet.target.is_none() {
            return self.publish_balanced_request(packet).await;
        }

        self.publish(packet)
    }

    async fn publish_balanced_event(&self, packet: Packet, group: String);
    async fn publish_balanced_request(&self, packet: Packet) -> anyhow::Result<()>;
    fn publish(&self, packet: Packet) -> anyhow::Result<()> {
        let topic = self.get_topic_name(&packet);
        //TODO: get serialized data
        Ok(())
    }
    fn get_topic_name(&self, packet: &Packet) -> String {
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

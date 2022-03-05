use std::{any, sync::Arc};

use crate::ServiceBroker;

use super::{balanced_event_regex_replace, PacketType, Transporter};
use anyhow::bail;
use async_nats::*;
use log::info;
struct NatsTransporter {
    opts: NatsOptions,
    connected: bool,
    has_built_in_balancer: bool,
    subscriptions: Vec<Arc<Subscription>>,
    client: Option<Connection>,
    broker: Arc<ServiceBroker>,
    prefix: String,
}

impl NatsTransporter {
    fn new(opts: NatsOptions, broker: Arc<ServiceBroker>) -> Self {
        let prefix = match &broker.namespace {
            Some(prefix) => prefix.to_owned(),
            None => "MOL".to_string(),
        };
        Self {
            opts,
            connected: false,
            has_built_in_balancer: true,
            subscriptions: Vec::new(),
            client: None,
            broker,
            prefix,
        }
    }
    async fn connect(&mut self, url: String) -> anyhow::Result<()> {
        let connection = async_nats::connect(&url).await?;
        info!("NATS client is connected");
        self.client = Some(connection);

        Ok(())
    }
    async fn disconnect(&mut self) -> anyhow::Result<()> {
        match &self.client {
            Some(client) => {
                let result = client.flush().await?;
                let _ = client.close().await?;
                self.client = None;
            }
            None => {}
        }
        Ok(())
    }
    async fn subscibe(&self, cmd: &str, node_id: &Option<String>) -> anyhow::Result<()> {
        let t = self.get_topic_name(cmd, node_id);
        let client = self.get_client()?;
        let subscription = client.subscribe(&t).await?;
        tokio::spawn(async move {
            while let Some(msg) = subscription.next().await {
                todo!("HANDLE SUBS MESSAGES")
                // self.receive(cmd, msg.data);
            }
        });
        Ok(())
    }

    async fn subscibe_balanced_request(&'static mut self, action: &str) -> anyhow::Result<()> {
        let topic = format!("{}.{}B.{}", self.prefix(), PacketType::Request, action);

        let client = self.get_client()?;
        let sub = client.queue_subscribe(&topic, action).await?;
        let sub = Arc::new(sub);
        self.subscriptions.push(Arc::clone(&sub));

        tokio::spawn(async move {
            while let Some(msg) = sub.next().await {
                self.receive(PacketType::Request, msg.data);
                // todo!("HANDLE SUBS MESSGES IN BALANCED REQ")
            }
        });

        Ok(())
    }
    async fn subscibe_balanced_event(&mut self, event: &str, group: &str) -> anyhow::Result<()> {
        let topic = format!(
            "{}.{}B.{}.{}",
            self.prefix(),
            PacketType::Event,
            group,
            event
        );
        let topic = balanced_event_regex_replace(&topic);

        let client = self.get_client()?;
        let sub = client.queue_subscribe(&topic, group).await?;
        let sub = Arc::new(sub);
        self.subscriptions.push(Arc::clone(&sub));

        tokio::spawn(async move {
            while let Some(msg) = sub.next().await {
                todo!("HANDLE SUBS MESSAGE IN BALANCED EVENT")
                // self.receive(PacketType::Event, msg.data);
            }
        });

        Ok(())
    }

    async fn ubsubscribe_from_balanced_commands(&mut self) -> anyhow::Result<()> {
        for sub in &self.subscriptions {
            sub.unsubscribe().await?;
        }
        self.subscriptions.clear();
        let client = self.get_client()?;
        client.flush().await?;
        Ok(())
    }

    async fn send(&self, topic: &str, data: Vec<u8>) -> anyhow::Result<()> {
        match &self.client {
            Some(client) => {
                client.publish(topic, data).await?;
            }
            None => return Ok(()),
        }
        Ok(())
    }

    fn get_client(&self) -> anyhow::Result<&Connection> {
        match &self.client {
            Some(client) => Ok(client),
            None => bail!("NATS client not connected"),
        }
    }
}

impl Transporter for NatsTransporter {
    fn broker(&self) -> &Arc<crate::ServiceBroker> {
        &self.broker
    }

    fn connected(&self) -> bool {
        self.connected
    }

    fn prefix(&self) -> &String {
        &self.prefix
    }

    fn connect<'life0, 'async_trait>(
        &'life0 self,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = ()> + core::marker::Send + 'async_trait>,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        todo!()
    }

    fn on_connected(&mut self, was_reconnect: bool) {
        todo!()
    }

    fn disconnect<'life0, 'async_trait>(
        &'life0 self,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = ()> + core::marker::Send + 'async_trait>,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        todo!()
    }

    fn incoming_message<'life0, 'async_trait>(
        &'life0 self,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = ()> + core::marker::Send + 'async_trait>,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        todo!()
    }

    fn subscibe<'life0, 'async_trait>(
        &'life0 self,
        topic: super::Topic,
    ) -> core::pin::Pin<
        Box<
            dyn core::future::Future<Output = anyhow::Result<()>>
                + core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        todo!()
    }

    fn subscibe_balanced_request<'life0, 'async_trait>(
        &'life0 self,
    ) -> core::pin::Pin<
        Box<
            dyn core::future::Future<Output = anyhow::Result<()>>
                + core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        todo!()
    }

    fn subscibe_balanced_event<'life0, 'async_trait>(
        &'life0 self,
    ) -> core::pin::Pin<
        Box<
            dyn core::future::Future<Output = anyhow::Result<()>>
                + core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        todo!()
    }

    fn send<'life0, 'async_trait>(
        &'life0 self,
        topic: String,
        data: String,
        meta: Option<super::TransporterMeta>,
    ) -> core::pin::Pin<
        Box<
            dyn core::future::Future<Output = anyhow::Result<()>>
                + core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        todo!()
    }
}

struct NatsOptions {
    preserve_buffers: bool,
    max_reconnect_attempts: i16,
}
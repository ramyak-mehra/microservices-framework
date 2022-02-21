use std::{
    any::Any,
    collections::HashMap,
    sync::{mpsc::Receiver, Arc},
};

use anyhow::{bail, Result};
use log::{debug, info, warn};

use crate::{
    context::Context,
    errors::ServiceBrokerError,
    registry::{self, endpoint_list, node, ActionEndpoint, EndpointTrait, EventEndpoint, Payload},
    strategies::{RoundRobinStrategy, Strategy},
    utils,
};
use chrono::{DateTime, Duration, Local, NaiveDateTime, Utc};
use serde_json::Value;
use tokio::{
    sync::{
        mpsc::{UnboundedReceiver, UnboundedSender},
        oneshot, RwLock,
    },
    task,
};

use crate::{registry::Logger, service::ServiceSpec, Registry, Service};

#[derive(Debug)]
struct RetryPolicy {
    enabled: bool,
    retries: usize,
    delay: usize,
    max_delay: usize,
    factor: usize,
    /*
    check :
    */
}
impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            enabled: false,
            retries: 5,
            delay: 100,
            max_delay: 1000,
            factor: 2,
        }
    }
}

#[derive(Debug)]
pub struct BrokerOptions {
    transporter: String,
    heartbeat_frequency: Duration,
    heartbeat_timeout: Duration,
    offline_check_frequency: Duration,
    offline_timeout: Duration,
    neighbours_checkout_timeout: Duration,
    wait_for_dependencies_timeout: Duration,
    namespace: String,
    request_timeout: Duration,
    mcall_timeout: Duration,
    retry_policy: RetryPolicy,
    pub max_call_level: usize,
    pub metrics: bool,
    metrics_rate: f32,
    wait_for_neighbours_interval: Duration,
    dont_wait_for_neighbours: bool,
    strategy_factory: RwLock<RoundRobinStrategy>,
   pub metadata : Value
    /*
    discover_node_id : fn()->String,
    metrics bool
    metric
    middleware
    loglevel
    logformat
    transporter factory
    */
}

impl Default for BrokerOptions {
    fn default() -> Self {
        Self {
            transporter: "TCP".to_string(),
            heartbeat_frequency: Duration::seconds(5),
            heartbeat_timeout: Duration::seconds(15),
            offline_check_frequency: Duration::seconds(20),
            offline_timeout: Duration::minutes(10),
            dont_wait_for_neighbours: true,
            neighbours_checkout_timeout: Duration::seconds(2),
            wait_for_dependencies_timeout: Duration::seconds(2),
            namespace: "".to_string(),
            request_timeout: Duration::seconds(3),
            mcall_timeout: Duration::seconds(5),
            retry_policy: RetryPolicy::default(),
            metrics: false,
            metrics_rate: 1.0,
            max_call_level: 100,
            wait_for_neighbours_interval: Duration::milliseconds(200),
            strategy_factory: RwLock::new(RoundRobinStrategy::new()),metadata:Value::Null
        }
    }
}

#[derive(Debug)]

pub struct ServiceBroker {
    reciever: UnboundedReceiver<ServiceBrokerMessage>,
    pub(crate) sender: UnboundedSender<ServiceBrokerMessage>,
    started: bool,
    namespace: Option<String>,
    metdata: Payload,
    pub node_id: String,
    pub instance: String,
    services: Vec<Service>,
    pub transit: Option<Transit>,
    pub logger: Arc<Logger>,
    pub options: BrokerOptions,

    /*
    local bus

    logger
    metricss
    middlewere
    cacher
    serializer
    error generator
    validator
    tracer
    transporter
    */
    registry: Option<Registry>,
}
#[derive(Debug)]

pub struct Transit {}

impl ServiceBroker {
    fn start(&mut self) {
        let time = Utc::now();
        self.started = true;
    }

    fn stop(&mut self) {
        todo!("handle stopping the broker")
    }

    fn add_local_service(&mut self, service: Service) {
        self.services.push(service);
    }
    fn register_local_service(&mut self, service: ServiceSpec) -> anyhow::Result<()> {
        match &mut self.registry {
            Some(registry) => registry.register_local_service(service),
            None => todo!(),
        }
    }

    async fn destroy_service(&mut self, name: &str, version: &str) -> Result<()> {
        let service_index = self.get_local_service_index(name, version);
        if service_index.is_none() {
            bail!(
                "no service with the name {} and version {} found",
                name,
                version
            );
        }
        let service_index = service_index.unwrap();
        let mut full_name = "".to_string();

        {
            let service = self.services.get_mut(service_index).unwrap();
            full_name = service.full_name.clone();
            service.stop().await;
        }
        {
            self.services.remove(service_index);
        }
        match &mut self.registry {
            Some(registry) => registry.unregister_service(&full_name, Some(&self.node_id)),
            None => todo!(),
        }

        self.services_changed(true);
        Ok(())
    }

    fn services_changed(&self, local_service: bool) {
        if self.started && local_service {
            todo!("notifify remote nodes")
        }
    }
    fn get_local_service_index(&self, name: &str, version: &str) -> Option<usize> {
        self.services.iter().position(|s| {
            if s.name == name && s.version == version {
                return true;
            }
            false
        })
    }

    fn wait_for_services(&self, service_names: Vec<String>, timeout: i64, interval: i64) {
        info!("Waiting for service(s) {:?}", service_names);
        let start_time = Local::now();
        let check = async {
            let service_statuses = service_names.iter().map(|service_name| {
                let status = self
                    .registry
                    .as_ref()
                    .unwrap()
                    .has_services(service_name, None);
                return ServiceStatus {
                    name: service_name,
                    available: status,
                };
            });

            let available_services: Vec<ServiceStatus> = service_statuses
                .clone()
                .filter(|service| service.available)
                .collect();
            if available_services.len() == service_names.len() {
                info!("Service(s) {:?} are available.", service_names);
                return;
            }

            let unavailable_services: Vec<ServiceStatus> = service_statuses
                .filter(|service| !service.available)
                .collect();

            debug!("{} {:?} of {} services are availablle. {} {} are still unavailable. Waiting for further..." , 
            available_services.len(),
            available_services.iter().map(|service_status| service_status.name).collect::<String>(),
            service_names.len(),
            unavailable_services.len(),
            unavailable_services.iter().map(|service_status|service_status.name).collect::<String>());

            if Local::now() - start_time > Duration::milliseconds(timeout) {
                //TODO: reject the future.
                return;
            }
            //TODO: add the setTimeout thing
            // Delay::new()
        };
    }

    async fn call(
        &self,
        action_name: &str,
        params: Payload,
        opts: CallOptions,
        sender: oneshot::Sender<Result<HandlerResult>>,
    ) -> anyhow::Result<()> {
        let ctx = Context::new(&self, "test_service".to_string());
        let ctx = ctx.child_action_context(&self, params.clone(), Some(opts.clone()), action_name);
        let endpoint = self.find_next_action_endpoint(action_name, &opts, &ctx)?;
        let endpoint = endpoint.clone();
        if endpoint.is_local() {
            debug!(
                "Call action locally. {{ action: {} , request_id : {:?} }}",
                ctx.action(),
                ctx.request_id
            )
        } else {
            debug!(
                "Call action on remote node. {{ action: {} , node_id : {} ,  request_id : {:?} }}",
                ctx.action(),
                ctx.node_id(),
                ctx.request_id
            )
        }
        task::spawn( async move {
            let result = (endpoint.action.handler)(ctx, Some(params));
            let _ = sender.send(Ok(result));
        });
        Ok(())
    }

    fn find_next_action_endpoint(
        &self,
        action_name: &str,
        opts: &CallOptions,
        ctx: &Context,
    ) -> anyhow::Result<&ActionEndpoint> {
        if let Some(node_id) = &opts.node_id {
            let ep = self
                .registry
                .as_ref()
                .unwrap()
                .get_action_endpoint_by_node_id(action_name, node_id);
            match ep {
                Some(ep) => Ok(ep),
                None => {
                    warn!("Service {} is not found on {} node.", action_name, node_id);

                    bail!(ServiceBrokerError::ServiceNotFound {
                        action_name: action_name.to_string(),
                        node_id: node_id.to_string()
                    })
                }
            }
        } else {
            //Get endpoint list by action name.
            let ep_list = self
                .registry
                .as_ref()
                .unwrap()
                .get_action_endpoints(action_name);
            match ep_list {
                Some(ep_list) => {
                    let ep = ep_list.next(ctx, &self.options.strategy_factory);
                    match ep {
                        Some(ep) => Ok(ep),
                        None => {
                            warn!("Service {} is not available.", action_name);
                            bail!(ServiceBrokerError::ServiceNotAvailable {
                                action_name: action_name.to_string(),
                                node_id: "".to_string()
                            });
                        }
                    }
                }
                None => {
                    warn!("Service {} is not registered.", action_name);

                    bail!(ServiceBrokerError::ServiceNotFound {
                        action_name: action_name.to_string(),
                        node_id: "".to_string()
                    })
                }
            }
        }
    }

    fn get_local_action_endpoint(
        &self,
        action_name: &str,
        ctx: &Context,
    ) -> anyhow::Result<&ActionEndpoint> {
        //Find action endpoints by name.
        let ep_list = self
            .registry
            .as_ref()
            .unwrap()
            .get_action_endpoints(action_name);
        let available = match ep_list {
            Some(endpoint_list) => !endpoint_list.has_local(),
            None => false,
        };
        if !available {
            bail!(ServiceBrokerError::ServiceNotFound {
                action_name: action_name.to_string(),
                node_id: self.node_id.clone()
            })
        }
        //Get local endpoint.
        match ep_list
            .unwrap()
            .next_local(ctx, &self.options.strategy_factory)
        {
            Some(ep) => Ok(ep),
            None => {
                bail!(ServiceBrokerError::ServiceNotAvailable {
                    action_name: action_name.to_string(),
                    node_id: self.node_id.clone()
                })
            }
        }
    }

    fn get_local_node_info(&self) -> anyhow::Result<Value> {
        self.registry.as_ref().unwrap().get_local_node_info(false)
    }

    fn emit_local_services(&self, ctx: Context) {
        todo!("event catalog implementation left")

        // self.registry.
    }

    fn get_cpu_usage() {
        todo!("get cpu usageI")
    }

    fn generate_uid() -> String {
        utils::generate_uuid()
    }
}

#[derive(Debug)]
pub enum ServiceBrokerMessage {
    AddLocalService(Service),
    RegisterLocalService(ServiceSpec),
    WaitForServices {
        dependencies: Vec<String>,
        timeout: i64,
        interval: i64,
    },
    Broadcast {
        event_name: String,
        data: Value,
        opts: Value,
    },
    Emit {
        event_name: String,
        data: Value,
        opts: Value,
    },
    Call {
        action_name: String,
        params: Payload,
        opts: CallOptions,
        result_channel: oneshot::Sender<anyhow::Result<HandlerResult>>,
    },
    Close,
}
impl PartialEq for ServiceBrokerMessage {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::AddLocalService(l0), Self::AddLocalService(r0)) => l0 == r0,
            (Self::RegisterLocalService(l0), Self::RegisterLocalService(r0)) => l0 == r0,
            (
                Self::WaitForServices {
                    dependencies: l_dependencies,
                    timeout: l_timeout,
                    interval: l_interval,
                },
                Self::WaitForServices {
                    dependencies: r_dependencies,
                    timeout: r_timeout,
                    interval: r_interval,
                },
            ) => {
                l_dependencies == r_dependencies
                    && l_timeout == r_timeout
                    && l_interval == r_interval
            }
            (
                Self::Broadcast {
                    event_name: l_event_name,
                    data: l_data,
                    opts: l_opts,
                },
                Self::Broadcast {
                    event_name: r_event_name,
                    data: r_data,
                    opts: r_opts,
                },
            ) => l_event_name == r_event_name && l_data == r_data && l_opts == r_opts,
            (
                Self::Emit {
                    event_name: l_event_name,
                    data: l_data,
                    opts: l_opts,
                },
                Self::Emit {
                    event_name: r_event_name,
                    data: r_data,
                    opts: r_opts,
                },
            ) => l_event_name == r_event_name && l_data == r_data && l_opts == r_opts,
            (
                Self::Call {
                    action_name: l_action_name,
                    params: l_params,
                    opts: l_opts,
                    result_channel: l_result_channel,
                },
                Self::Call {
                    action_name: r_action_name,
                    params: r_params,
                    opts: r_opts,
                    result_channel: r_result_channel,
                },
            ) => l_action_name == r_action_name && l_params == r_params && l_opts == r_opts,
            (Self::Close, Self::Close) => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct HandlerResult {
    // pub(crate) data: u32,
    pub(crate) data : Box<dyn Any + Send + Sync>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallOptions {
    meta: Payload,
    node_id: Option<String>,
}

struct ServiceStatus<'a> {
    name: &'a str,
    available: bool,
}

#[cfg(test)]
mod tests {
    use crate::{
        registry::{Action, Visibility},
        service::{Schema, SchemaMerged},
        Registry, Service,
    };
    use tokio::{sync::mpsc, task};

    use super::*;
    fn test_merge_func() {
        println!("test merge function");
    }
    fn test_started_func() {
        println!("test start func");
    }
    fn test_created_func() {
        println!("test created func");
    }
    fn test_stop_func() {
        println!("test stop func");
    }
    fn action_func(context: Context, payload: Option<Payload>) -> HandlerResult {

        let data = fibonacci(40);
        
        HandlerResult {data: Box::new(data) }
    }

    fn get_test_broker(
        recv: mpsc::UnboundedReceiver<ServiceBrokerMessage>,
        sender: mpsc::UnboundedSender<ServiceBrokerMessage>,
    ) -> ServiceBroker {
        ServiceBroker {
            reciever: recv,
            started: false,
            namespace: None,
            metdata: Payload {},
            sender,
            node_id: "test_node".to_string(),
            instance: "test_instance".to_string(),
            services: Vec::new(),
            transit: None,
            logger: Arc::new(Logger {}),
            options: BrokerOptions::default(),
            registry: None,
        }
    }
    fn get_test_schema(dependencies: Option<Vec<String>>) -> Schema {
        let merged = SchemaMerged::MergedFn(test_merge_func);
        let schema = Schema {
            mixins: None,
            actions: None,
            events: None,
            merged,
            name: "test_service".to_string(),
            version: None,
            settings: HashMap::new(),
            metadata: None,
            created: Some(test_created_func),
            started: Some(test_started_func),
            stopped: Some(test_stop_func),
            dependencies: dependencies,
        };

        schema
    }
    fn fibonacci(n: u32) -> u32 {
        match n {
            0 => 1,
            1 => 1,
            _ => fibonacci(n - 1) + fibonacci(n - 2),
        }
    }
    fn get_test_service(
        schema: Option<Schema>,
        settings: Option<HashMap<String, String>>,
        actions: Option<Vec<Action>>,
        broker_sender: Option<mpsc::UnboundedSender<ServiceBrokerMessage>>,
    ) -> Service {
        let name = "test_service".to_string();
        let version = "1.0".to_string();
        let full_name = Service::get_versioned_full_name(&name, Some(&version));
        let settings = match settings {
            Some(settings) => settings,
            None => HashMap::new(),
        };
        let schema = match schema {
            Some(schema) => schema,
            None => get_test_schema(None),
        };

        let original_schema = get_test_schema(None);
        let broker_sender = match broker_sender {
            Some(sender) => sender,
            None => {
                let (sender, recv) = mpsc::unbounded_channel::<ServiceBrokerMessage>();
                sender
            }
        };
        let service = Service {
            name,
            full_name,
            version,
            settings,
            schema,
            original_schema: Some(original_schema),
            metadata: HashMap::new(),
            actions: actions,
            events: None,
            broker_sender,
        };
        service
    }
    #[test]
    fn broker_call() {
        let (sender, recv) = mpsc::unbounded_channel::<ServiceBrokerMessage>();
        let (_, fake_recv) = mpsc::unbounded_channel();
        let mut broker_original = get_test_broker(recv, sender.clone());
        let broker = get_test_broker(fake_recv, sender.clone());
        let broker_arc = Arc::new(broker);
        let action = Action {
            name: "action_func".to_string(),
            visibility: Visibility::Public,
            handler: action_func,
        };
        let service = get_test_service(None, None, Some(vec![action]), Some(sender.clone()));
        let registry = Registry::new(Arc::clone(&broker_arc), sender.clone());
        let actions = registry.actions.clone();
        broker_original.registry = Some(registry);
        broker_original.start();
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let join_handle = rt.spawn(async move {
                while let Some(message) = broker_original.reciever.recv().await {
                    match message {
                        ServiceBrokerMessage::AddLocalService(service) => {
                            broker_original.add_local_service(service)
                        }
                        ServiceBrokerMessage::RegisterLocalService(service_spec) => broker_original
                            .register_local_service(service_spec)
                            .unwrap(),
                        ServiceBrokerMessage::Call {
                            action_name,
                            params,
                            opts,
                            result_channel,
                        } => {
                            let result = broker_original
                                .call(&action_name, params, opts, result_channel)
                                .await;
                        }
                        ServiceBrokerMessage::Close => return,
                        _ => {}
                    }
                }
            });
            // let sender = sender.clone();
            task::spawn(async move {
                service.init().await;
                service.start().await;
                println!("{:?}", actions);
                let start = Local::now();
                let sender = sender.clone();
                let mut jhs = Vec::new();
                for _ in 0..10 {
                    let sender = sender.clone();
                    let jh = task::spawn(async move {
                        let (one_sender, recv) = oneshot::channel();

                        let _ = sender.send(ServiceBrokerMessage::Call {
                            action_name: "action_func".to_string(),
                            params: Payload {},
                            opts: CallOptions {
                                meta: Payload {},
                                node_id: Some("test_node".to_string()),
                            },
                            result_channel: one_sender,
                        });
                        let _result = recv.await;
                        println!("{:?}" , _result);
                    });
                    jhs.push(jh);
                }
                for jh in jhs{
                    jh.await;
                }
                let end = Local::now();
                let __ = sender.send(ServiceBrokerMessage::Close);
                println!("{:?}", end - start);
                drop(sender);
            });

            let _ = join_handle.await;
        });
    }
}

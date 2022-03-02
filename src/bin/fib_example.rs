use serde::*;
use std::collections::HashMap;
use std::sync::Arc;

use molecular_rust::broker::*;
use molecular_rust::context::Context;
use molecular_rust::registry::*;
use molecular_rust::service::*;
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;

#[tokio::main]
async fn main() {
    let mut broker = get_broker();
    let registry = get_registry(broker.sender.clone());
    let action = Action {
        name: "fib_func".to_string(),
        visibility: Visibility::Public,
        handler: fib_func,
    };
    let broker_sender = broker.sender.clone();
    let service = get_test_service(None, None, Some(vec![action]), Some(broker_sender.clone()));
    broker.registry = Some(registry);
    broker.start();

    let message_handler = tokio::spawn(async move {
        while let Some(message) = broker.reciever.recv().await {
            match message {
                ServiceBrokerMessage::AddLocalService(service) => broker.add_local_service(service),
                ServiceBrokerMessage::RegisterLocalService(service_spec) => {
                    broker.register_local_service(service_spec).unwrap()
                }
                ServiceBrokerMessage::Call {
                    action_name,
                    params,
                    opts,
                    result_channel,
                } => {
                    let result = broker
                        .call(&action_name, params, opts, result_channel)
                        .await;
                }
                ServiceBrokerMessage::Close => return,
                _ => {}
            }
        }
    });
    let _ = tokio::spawn(async move {
        service.init().await;
        service.start().await;
    })
    .await;
    let sender = broker_sender.clone();
    let __ = tokio::spawn(async move {
        let (one_sender, one_recv) = oneshot::channel();

        let _ = sender.send(ServiceBrokerMessage::Call {
            action_name: "fib_func".to_string(),
            params: Payload {
                data: serde_json::to_value(ActionParams { num: 9 }).unwrap(),
            },
            opts: CallOptions {
                meta: Payload::default(),
                node_id: Some("test_node".to_string()),
            },
            result_channel: one_sender,
        });
        let _result = one_recv.await.unwrap().unwrap();
        println!("{:?}", _result.data.downcast::<u32>().unwrap());
        drop(sender);
    })
    .await;
    broker_sender.send(ServiceBrokerMessage::Close);
    let _ = message_handler.await;
}

#[derive(Serialize, Deserialize)]
struct ActionParams {
    num: u32,
}

fn get_broker() -> ServiceBroker {
    let (sender, recv) = mpsc::unbounded_channel::<ServiceBrokerMessage>();
    let broker = get_test_broker(recv, sender.clone());
    broker
}

fn get_registry(sender: UnboundedSender<ServiceBrokerMessage>) -> Registry {
    let (_, temp_recv) = mpsc::unbounded_channel();
    let temp_broker = get_test_broker(temp_recv, sender.clone());
    let broker_arc = Arc::new(temp_broker);
    Registry::new(Arc::clone(&broker_arc), sender)
}

fn test_merge_func() {
    println!("test merge function");
}
fn test_started_func() {
    println!("service started");
}
fn test_created_func() {
    println!("service created ");
}
fn test_stop_func() {
    println!("test stop func");
}
fn fib_func(context: Context, payload: Option<Payload>) -> HandlerResult {
    let invalid_params = HandlerResult {
        data: Box::new("Invalid parameters".to_string()),
    };
    if let Some(payload) = payload {
        let params: Result<ActionParams, serde_json::Error> = serde_json::from_value(payload.data);
        match params {
            Ok(params) => {
                let result = fibonacci(params.num);
                return HandlerResult {
                    data: Box::new(result),
                };
            }
            Err(_) => return invalid_params,
        }
    }
    return invalid_params;
}

fn get_test_broker(
    recv: mpsc::UnboundedReceiver<ServiceBrokerMessage>,
    sender: mpsc::UnboundedSender<ServiceBrokerMessage>,
) -> ServiceBroker {
    ServiceBroker {
        reciever: recv,
        started: false,
        namespace: None,
        metdata: Payload::default(),
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
        0 => 0,
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

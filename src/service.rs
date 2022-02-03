use std::{collections::HashMap, sync::Arc};

use crate::{
    registry::{Action, Event, Logger},
    ServiceBrokerMessage,
};
use chrono::Duration;
use log::info;
use tokio::sync::mpsc::Sender;

#[derive(Clone)]
pub(crate)struct Service {
    pub(crate)name: String,
    pub(crate)full_name: String,
    pub(crate)version: String,
    settings: HashMap<String, String>,
    logger: Arc<Logger>,
    schema: Schema,
    original_schema: Option<Schema>,
    metadata: HashMap<String, String>,
    pub(crate)actions: Option<Vec<Action>>,
    pub(crate)events: Option<HashMap<String, Event>>,
    broker_sender: Sender<ServiceBrokerMessage>,
}

#[derive(PartialEq, Eq, Clone)]
struct Schema {
    mixins: Option<Vec<SchemaMixins>>,
    actions: Option<Vec<SchemaActions>>,
    events: Option<Vec<SchemaEvents>>,
    merged: SchemaMerged,
    name: Option<String>,
    version: Option<String>,
    settings: HashMap<String, String>,
    metadata: Option<HashMap<String, String>>,
    created: Option<fn()>,
    started: Option<fn()>,
    stopped: Option<fn()>,
    dependencies: Option<Vec<String>>,
}

#[derive(PartialEq, Eq, Clone)]
struct SchemaMixins {}

#[derive(PartialEq, Eq, Clone)]
pub(crate)struct SchemaActions {
    name: String,
    handler: fn(),
}

#[derive(PartialEq, Eq, Clone)]
struct SchemaEvents {}
#[derive(PartialEq, Eq, Clone)]
enum SchemaMerged {
    MergedFn(fn()),
    MergedFnVec(Vec<fn()>),
}
pub(crate)struct ServiceSpec {
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) full_name: String,
    pub(crate) settings: HashMap<String, String>,
    /*
    pub(crate)metadata
    pub(crate)*/
    pub(crate) actions: Option<Vec<Action>>,
    pub(crate) events: Option<Vec<Event>>,
}

impl Service {
    fn get_service_spec(&self) -> ServiceSpec {
        //TODO: do something about actions and events
        let service_spec = ServiceSpec {
            name: self.name.clone(),
            full_name: self.full_name.clone(),
            settings: Service::get_public_settings(&self.settings),
            version: self.version.clone(),
            actions: None,
            events: None,
        };
        service_spec
    }

    fn parse_service_schema(&mut self, schema: Schema) {
        self.original_schema = Some(schema.clone());

        match schema.merged {
            SchemaMerged::MergedFn(func) => {

                //TODO research about functions
            }
            SchemaMerged::MergedFnVec(func_vec) => {
                for func in func_vec {
                    //TODO resarch more about functions
                }
            }
        }

        if let None = schema.name {
            //TODO throw error if no name available
        }
        self.name = schema.name.unwrap().clone();
        self.version = match schema.version {
            Some(v) => v,
            None => "0.0.1".to_string(),
        };
        self.settings = schema.settings;
        self.metadata = match schema.metadata {
            Some(metadata) => metadata,
            None => HashMap::new(),
        };
        //TODO:
        //self.schema = schema;
        let version = self.settings.get("$noVersionPrefix");

        self.full_name = Service::get_versioned_full_name(&self.name, version);
        //TODO: get the logger from the broker.
        //self.logger =

        //TODO:register methods.

        todo!("add service specification")
    }

    fn get_public_settings(settings: &HashMap<String, String>) -> HashMap<String, String> {
        settings.clone()
    }

    pub(crate)async fn init(&self) {
        info!("Service {} is createing....", self.full_name);
        if let Some(created) = self.schema.created {
            created();
        }
        let _result = self
            .broker_sender
            .send(ServiceBrokerMessage::AddLocalService(self.clone()))
            .await;
        info!("Service {} created.", self.full_name);

        todo!("call broker middlware")
    }

    pub(crate)async fn start(&self) {
        info!("Service {} is starting...", self.full_name);

        if let Some(dependencies) = &self.schema.dependencies {
            let timeout: i64 = match self.settings.get("$dependencyTimeout") {
                //TODO:raise an error rateher than default.
                Some(val) => val.parse().unwrap_or(0),
                None => 0,
            };
            //TODO: get interal from broker options
            let interval: i64 = match self.settings.get("$dependencyInterval") {
                Some(val) => val.parse().unwrap_or(0),
                None => 0,
            };

            self.wait_for_services(dependencies, timeout, interval)
                .await;
        }

        if let Some(started) = &self.schema.started {
            started();
        }

        let _result = self
            .broker_sender
            .send(ServiceBrokerMessage::RegisterLocalService(
                Service::get_service_spec(&self),
            ))
            .await;

        info!("Service {} started.", self.full_name);

        todo!("call service starting middleware");
        todo!("call service started middleware")
    }

    pub(crate)async fn stop(&self) {
        info!("Service {} is stopping...", self.full_name);

        if let Some(stopped) = self.schema.stopped {
            stopped();
        }

        info!("Service {} stopped.", self.full_name);

        todo!("call service stopping middlewares");
        todo!("call service stopped middleware");
    }

    fn create_action(&self, action_def: fn(), name: &str) -> Action {
        let mut action = Action::new(name.to_string(), action_def);
        let name_prefix = self.settings.get("$noServiceNamePrefix");
        if let Some(name_prefix) = name_prefix {
            let name_prefix: bool = name_prefix.parse().unwrap();
            if name_prefix {
                action.name = format!("{}.{}", self.full_name.to_string(), action.name);
            }
        }
        //TODO add caching settings from settins
        //TODO better way to handle this instead of clone.
        //TODO see if it is even necessary to give action access to the service.
        // action = action.set_service(self.clone());
        action
    }

    /// create an interal service method.
    fn create_method() {
        todo!()
    }

    ///create an event subscription for broker
    fn create_event() {
        todo!()
    }

    async fn wait_for_services(&self, service_names: &Vec<String>, timeout: i64, interval: i64) {
        let _result = self
            .broker_sender
            .send(ServiceBrokerMessage::WaitForServices {
                dependencies: service_names.clone(),
                interval,
                timeout,
            })
            .await;
    }

    fn get_versioned_full_name(name: &str, version: Option<&String>) -> String {
        let mut name = name.to_string();
        if let Some(v) = version {
            name = format!("{}.{}", v, name);
        }
        name
    }
}

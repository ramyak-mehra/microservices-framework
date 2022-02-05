use std::collections::HashMap;

use crate::{
    registry::{Action, Event},
    ServiceBrokerMessage,
};
use log::info;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Clone, Debug)]
pub struct Service {
    pub name: String,
    pub full_name: String,
    pub version: String,
    settings: HashMap<String, String>,
    schema: Schema,
    original_schema: Option<Schema>,
    metadata: HashMap<String, String>,
    pub actions: Option<Vec<Action>>,
    pub events: Option<HashMap<String, Event>>,
    broker_sender: UnboundedSender<ServiceBrokerMessage>,
}

impl PartialEq for Service {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.full_name == other.full_name
            && self.version == other.version
            && self.settings == other.settings
            && self.schema == other.schema
            && self.original_schema == other.original_schema
            && self.metadata == other.metadata
            && self.actions == other.actions
            && self.events == other.events
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
struct Schema {
    mixins: Option<Vec<SchemaMixins>>,
    actions: Option<Vec<SchemaActions>>,
    events: Option<Vec<SchemaEvents>>,
    merged: SchemaMerged,
    name: String,
    version: Option<String>,
    settings: HashMap<String, String>,
    metadata: Option<HashMap<String, String>>,
    created: Option<fn()>,
    started: Option<fn()>,
    stopped: Option<fn()>,
    dependencies: Option<Vec<String>>,
}

#[derive(PartialEq, Eq, Clone, Debug)]
struct SchemaMixins {}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct SchemaActions {
    name: String,
    handler: fn(),
}

#[derive(PartialEq, Eq, Clone, Debug)]
struct SchemaEvents {}
#[derive(PartialEq, Eq, Clone, Debug)]
enum SchemaMerged {
    MergedFn(fn()),
    MergedFnVec(Vec<fn()>),
}
#[derive(PartialEq, Debug)]
pub struct ServiceSpec {
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) full_name: String,
    settings: HashMap<String, String>,
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
            settings: self.get_public_settings(),
            version: self.version.clone(),
            actions: None,
            events: None,
        };
        service_spec
    }

    fn parse_service_schema(&mut self, schema: Schema) {
        self.original_schema = Some(schema.clone());

        match schema.merged {
            SchemaMerged::MergedFn(merged) => merged(),
            SchemaMerged::MergedFnVec(func_vec) => {
                for func in func_vec {
                    func()
                }
            }
        }

        self.name = schema.name;
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

    fn get_public_settings(&self) -> HashMap<String, String> {
        self.settings.clone()
    }

    pub async fn init(&self) {
        info!("Service {} is createing....", self.full_name);
        if let Some(created) = self.schema.created {
            created();
        }
        let _result = self
            .broker_sender
            .send(ServiceBrokerMessage::AddLocalService(self.clone()));
        info!("Service {} created.", self.full_name);

        //  todo!("call broker middlware")
    }

    pub async fn start(&self) {
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
            ));

        info!("Service {} started.", self.full_name);

        //  todo!("call service starting middleware");
        //todo!("call service started middleware")
    }

    pub async fn stop(&self) {
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
            if !name_prefix {
                action.name = format!("{}.{}", self.full_name.to_string(), action.name);
            }
        }
        //TODO add caching settings from settins
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
            });
    }

    fn get_versioned_full_name(name: &str, version: Option<&String>) -> String {
        let mut name = name.to_string();
        if let Some(v) = version {
            name = format!("{}.{}", v, name);
        }
        name
    }
}
#[cfg(test)]
mod tests {
    use tokio::sync::mpsc::{self, UnboundedSender};

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
    fn action_func() {
        println!("action_func");
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

    fn get_test_service(
        schema: Option<Schema>,
        settings: Option<HashMap<String, String>>,
        actions: Option<Vec<Action>>,
        broker_sender: Option<UnboundedSender<ServiceBrokerMessage>>,
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
    fn service_get_service_spec() {
        let service = get_test_service(None, None, None, None);
        let service_spec = ServiceSpec {
            name: service.name.clone(),
            version: service.version.clone(),
            full_name: service.full_name.clone(),
            settings: service.settings.clone(),
            actions: None,
            events: None,
        };

        let service_spec_gen = service.get_service_spec();
        assert_eq!(service_spec_gen, service_spec)
    }
    #[test]
    fn service_get_public_settings() {
        let mut settings = HashMap::new();
        settings.insert("test".to_string(), "settings".to_string());
        let service = get_test_service(None, Some(settings.clone()), None, None);
        assert_eq!(service.get_public_settings(), settings);
    }
    #[test]
    fn service_init() {
        let (sender, mut recv) = mpsc::unbounded_channel::<ServiceBrokerMessage>();
        let service = get_test_service(None, None, None, Some(sender));
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async { service.init().await });
        rt.block_on(async {
            let result = recv.recv().await;

            let expected_result = ServiceBrokerMessage::AddLocalService(service);
            assert_eq!(result, Some(expected_result));
        });
    }
    #[test]
    fn service_start() {
        let (sender, mut recv) = mpsc::unbounded_channel::<ServiceBrokerMessage>();
        let service = get_test_service(None, None, None, Some(sender));
        let service_spec = service.get_service_spec();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async { service.start().await });
        rt.block_on(async {
            let result = recv.recv().await;
            let expected_result = ServiceBrokerMessage::RegisterLocalService(service_spec);
            assert_eq!(result, Some(expected_result));
        });
    }
    #[test]
    fn service_start_with_dependencies() {
        let (sender, mut recv) = mpsc::unbounded_channel::<ServiceBrokerMessage>();
        let service_names: Vec<String> =
            vec!["Service one".to_string(), " Service two".to_string()];

        let schema = get_test_schema(Some(service_names.clone()));
        let service = get_test_service(Some(schema), None, None, Some(sender));
        let service_spec = service.get_service_spec();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async { service.start().await });
        rt.block_on(async {
            let result = recv.recv().await;
            let expected_result = ServiceBrokerMessage::WaitForServices {
                dependencies: service_names,
                timeout: 0,
                interval: 0,
            };
            assert_eq!(result, Some(expected_result));

            let result = recv.recv().await;
            let expected_result = ServiceBrokerMessage::RegisterLocalService(service_spec);
            assert_eq!(result, Some(expected_result));
        });
    }
    #[test]
    fn service_create_action_name() {
        let no_service_name_prefix = "$noServiceNamePrefix".to_string();
        let name = "action_func";
        let mut settings = HashMap::new();
        settings.insert(no_service_name_prefix.clone(), "true".to_string());
        let service = get_test_service(None, Some(settings.clone()), None, None);
        let action = service.create_action(action_func, name);
        let expected_action = Action::new(name.to_string(), action_func);
        assert_eq!(action, expected_action);
        settings.insert(no_service_name_prefix.clone(), "false".to_string());
        let service = get_test_service(None, Some(settings), None, None);
        let action_2 = service.create_action(action_func, name);
        let name = format!("{}.{}", service.full_name, name);
        let expected_action_2 = Action::new(name, action_func);
        assert_eq!(action_2, expected_action_2);
        let service = get_test_service(None, None, None, None);
        let name = "action_func";
        let expected_action_3 = Action::new(name.to_string(), action_func);
        let action_3 = service.create_action(action_func, name);
        assert_eq!(action_3, expected_action_3);
    }
    #[test]
    #[should_panic]
    fn service_create_action_name_panic() {
        let no_service_name_prefix = "$noServiceNamePrefix".to_string();
        let mut settings = HashMap::new();
        let name = "action_func";
        settings.insert(no_service_name_prefix, "non_bool_value".to_string());
        let service = get_test_service(None, Some(settings), None, None);
        let action = service.create_action(action_func, name);
    }
    #[test]
    fn service_wait_for_service() {
        let (sender, mut recv) = mpsc::unbounded_channel::<ServiceBrokerMessage>();
        let service = get_test_service(None, None, None, Some(sender));
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let service_names: Vec<String> =
            vec!["Service one".to_string(), " Service two".to_string()];
        let timeout = 10;
        let interval = 20;
        rt.block_on(async {
            let _result = service
                .wait_for_services(&service_names, timeout, interval)
                .await;
        });
        rt.block_on(async {
            let result = recv.recv().await;
            let expected_result = ServiceBrokerMessage::WaitForServices {
                dependencies: service_names,
                timeout,
                interval,
            };
            assert_eq!(result, Some(expected_result));
        });
    }
    #[test]
    fn service_get_versioned_full_name() {
        let version = "1.0";
        let name = "test_service";
        let expected_full_name = format!("{}.{}", version, name);
        let version = "1.0".to_string();
        let version = Some(&version);
        let full_name = Service::get_versioned_full_name(name, version);
        assert_eq!(full_name , expected_full_name);
        let full_name_2 = Service::get_versioned_full_name(name, None);
        assert_eq!(full_name_2  , name);
    }   
}

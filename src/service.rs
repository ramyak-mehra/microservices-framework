use std::{collections::HashMap, sync::Arc};

use chrono::Duration;

use crate::registry::{Action, Event, Logger};

#[derive(PartialEq, Eq, Clone)]
pub struct Service {
    pub name: String,
    pub full_name: String,
    pub version: String,
    settings: HashMap<String, String>,
    logger: Arc<Logger>,
    schema: Schema,
    original_schema: Option<Schema>,
    metadata: HashMap<String, String>,
    actions: HashMap<String, Action>,
    events: HashMap<String, Event>,
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
}

#[derive(PartialEq, Eq, Clone)]
struct SchemaMixins {}

#[derive(PartialEq, Eq, Clone)]
struct SchemaActions {}

#[derive(PartialEq, Eq, Clone)]
struct SchemaEvents {}
#[derive(PartialEq, Eq, Clone)]
enum SchemaMerged {
    MergedFn(fn()),
    MergedFnVec(Vec<fn()>),
}

impl Service {
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

    pub fn init(&mut self) {
        todo!("call broker to initialise the service and call the init method of service")
    }

    pub fn start(&mut self) {
        todo!("call the broker to start the services and call the start method of services")
    }

    pub fn stop(&mut self) {
        todo!("call the broker to stop the service and call the stop method of service")
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
        action = action.set_service(self.clone());
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

    fn wait_for_services(&self, service_names: Vec<String>, timout: Duration, interval: Duration) {
        todo!("call broker to wait for services")
    }

    fn get_versioned_full_name(name: &str, version: Option<&String>) -> String {
        let mut name = name.to_string();
        if let Some(v) = version {
            name = format!("{}.{}", v, name);
        }
        name
    }
}

use crate::service::ServiceSpec;

use super::*;

#[derive(PartialEq, Eq, Clone, Debug)]
pub(crate) struct ServiceItem {
    pub(crate) name: String,
    pub(crate) node: String,
    pub(crate) local: bool,
    pub(crate) full_name: String,
    version: String,
    pub(crate) actions: HashMap<String, Action>,
    pub(crate) events: HashMap<String, Event>,
    metadata: Payload,
    dependencies: Option<Vec<String>>, /*
                                       eventsmap
                                       settings
                                       */
}
impl ServiceItem {
    pub(crate) fn new(node: &Node, service: &ServiceSpec, local: bool) -> Self {
        Self {
            node: node.id.clone(),
            local,
            actions: HashMap::new(),
            events: HashMap::new(),
            full_name: service.full_name.to_string(),
            version: service.version.to_string(),
            name: service.name.to_string(),
            metadata: Payload::default(),
            dependencies: service.dependencies.clone(),
        }
    }
    pub(crate) fn equals(&self, full_name: &str, node_id: Option<&str>) -> bool {
        match node_id {
            Some(id) => self.node == id && self.full_name == full_name,
            None => self.full_name == full_name,
        }
    }

    ///Update service properties
    pub(crate) fn update(&mut self, service: &Service) {
        self.full_name = service.full_name.to_string();
        self.version = service.version.to_string();
        /*
        settings
        metadata
        */
        todo!()
    }
    ///Add action to service
    pub(crate) fn add_action(&mut self, action: Action) {
        let name = action.name.clone();
        self.actions.insert(name, action);
        todo!("Decide if we want an arc of action or make a copy of that actions")
    }
    pub(crate) fn add_event(&mut self, event: Event) {
        let name = event.name.clone();
        self.events.insert(name, event);
    }
    pub(crate) fn unique_name(&self) -> String {
        format!("{}{}{}", self.full_name, self.version, self.node)
    }
}
#[derive(Debug , Serialize , Clone  , PartialEq, Eq)]
pub(crate) struct ServiceItemInfo {
    name: String,
    version: String,
    full_name: String,
    metadata: Payload,
    dependencies: Option<Vec<String>>,
    actions: HashMap<String, ActionInfo>,
    events: HashMap<String, EventInfo>,
}

impl From<&ServiceItem> for ServiceItemInfo {
    fn from(item: &ServiceItem) -> Self {
        let mut actions: HashMap<String, ActionInfo> = HashMap::new();
        item.actions.iter().for_each(|action_item| {
            let (name, action) = action_item;
            if action.visibility == Visibility::Protected {
                return;
            }
            actions.insert(name.clone(), action.into());
        });
        let mut events = HashMap::new();
        item.events.iter().for_each(|event_item| {
            let (name, event) = event_item;
            events.insert(name.clone(), event.into());
        });
        Self {
            name: item.name.clone(),
            version: item.version.clone(),
            full_name: item.full_name.clone(),
            metadata: item.metadata.clone(),
            dependencies: item.dependencies.clone(),
            actions: actions,
            events: events,
        }
    }
}
// impl From<&ServiceItem> for ServiceItemInfo{
//     fn from(item:&mut ServiceItem)->Self{

//     }
// }



#[derive(Debug , Serialize , Clone , PartialEq, Eq )]

struct ActionInfo {
    name: String,
    visibility: Visibility,
}

impl From<&Action> for ActionInfo {
    fn from(item: &Action) -> Self {
        Self {
            name: item.name.clone(),
            visibility: item.visibility.clone(),
        }
    }
}
#[derive(Debug , Serialize , Clone , PartialEq , Eq )]

struct EventInfo {
    name: String,
    group: Option<String>,
}

impl From<&Event> for EventInfo {
    fn from(item: &Event) -> Self {
        Self {
            name: item.name.clone(),
            group: item.group.clone(),
        }
    }
}

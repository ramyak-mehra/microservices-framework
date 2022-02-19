use thiserror::Error;
#[derive(Error, Debug)]
pub enum ServiceBrokerError {
    #[error("Service {action_name:?} is not registered locally.")]
    ServiceNotFound {
        action_name: String,
        node_id: String,
    },
    #[error("Service {action_name:?} is not available locally.")]
    ServiceNotAvailable {
        action_name: String,
        node_id: String,
    },
}

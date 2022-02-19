use thiserror::Error;
#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("No local node found")]
    NoLocalNodeFound,
    #[error("No service found")]
    NoServiceItemFound,
}

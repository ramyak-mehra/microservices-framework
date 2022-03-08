use thiserror::Error;
#[derive(Error, Debug)]
pub(crate)enum RegistryError {
    #[error("No local node found")]
    NoLocalNodeFound,
    #[error("No service found")]
    NoServiceItemFound,
}

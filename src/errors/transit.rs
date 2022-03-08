use thiserror::Error;

#[derive(Error, Debug)]
pub enum TransitError {
    #[error("Cannot parse payload. {0}")]
    CannotParse(String),
   
}

use thiserror::Error;

#[derive(Error, Debug)]
pub(crate)enum TransitError {
    #[error("Cannot parse payload. {0}")]
    CannotParse(String),
   
}

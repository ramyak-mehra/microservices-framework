use thiserror::Error;

use crate::transporter::transit::TransitMessage;
#[derive(Error, Debug)]
pub enum TransitError {
    #[error("Cannot parse payload. {0}")]
    CannotParse(String),
    #[error("Failed to send {0}. {1}")]
    SendError(TransitMessage , String),
}

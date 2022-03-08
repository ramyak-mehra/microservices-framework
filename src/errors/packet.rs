use thiserror::Error;
#[derive(Error, Debug)]
pub(crate)enum PacketError {
    #[error("Cannot parse payload. {0}")]
    CannotParse(String),
    #[error("No way to parse payload.")]
    NoFullPacket,
}

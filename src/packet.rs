use core::fmt;




enum PacketType {
    Unknown,
    Event,
    Request,
    Response,
    Discover,
    Info ,
    Disconnect ,
    Heartbeat,
    Ping,
    Pong,
}

impl From<PacketType> for String {
    fn from(p: PacketType) -> Self {
        "as".to_string()
    }
}

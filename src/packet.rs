enum PacketType {
    Unknown,
    Event,
    Request,
    Response,
    Discover,
    Info,
    Disconnect,
    Heartbeat,
    Ping,
    Pongs,
}

impl From<PacketType> for String {
    fn from(p: PacketType) -> Self {
        "as".to_string()
    }
}

use crate::packet::{Packet, PayloadNull, PayloadResponse};

use super::*;

#[derive(Debug)]
pub struct JSONSerializer {}
impl JSONSerializer {
    fn do_it(&self) -> impl PacketPayload {
        PayloadNull {}
    }
}
impl BaseSerializer for JSONSerializer {
    type Data = String;

    fn serialize<S: serde::Serialize>(&self, data: S) -> anyhow::Result<Self::Data> {
        match serde_json::to_string(&data) {
            Ok(result) => Ok(result),
            Err(err) => bail!(err),
        }
    }

    fn deserialize(&self, data: &[u8], tipe: PacketType) {}
}

pub(crate)mod json;


pub(crate)use anyhow::bail;
pub(crate)use serde::{Deserialize, Serialize};

use crate::packet::Packet;
pub(crate) use crate::packet::{PacketPayload, PacketType};

pub(crate)trait BaseSerializer {
    type Data;
    fn serialize<S: Serialize>(&self, data: S) -> anyhow::Result<Self::Data>;
    fn deserialize(&self,data: &[u8], tipe: PacketType);
}

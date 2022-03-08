pub mod json;


pub use anyhow::bail;
pub use serde::{Deserialize, Serialize};

use crate::packet::Packet;
pub(crate) use crate::packet::{PacketPayload, PacketType};

pub trait BaseSerializer {
    type Data;
    fn serialize<S: Serialize>(&self, data: S) -> anyhow::Result<Self::Data>;
    fn deserialize(&self,data: &[u8], tipe: PacketType);
}

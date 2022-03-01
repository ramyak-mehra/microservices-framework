use serde::{Deserialize, Serialize};

use crate::packet::{PacketPayload, PacketType};
pub mod json;

pub trait BaseSerializer {
    type Data;
    fn serialize<S: Serialize>(&self,data: S) -> anyhow::Result<Self::Data>;
    fn deserialize<'a, D: Deserialize<'a>>(data: D) -> Self::Data;
 
}

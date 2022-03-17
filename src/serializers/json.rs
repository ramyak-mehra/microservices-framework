use super::*;

#[derive(Debug)]
pub(crate) struct JSONSerializer {}
impl JSONSerializer {}
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

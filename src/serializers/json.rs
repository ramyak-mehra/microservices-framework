use anyhow::bail;

use super::BaseSerializer;

#[derive(Debug)]
pub struct JSONSerializer {}
impl BaseSerializer for JSONSerializer {
    type Data = String;

     fn serialize<S: serde::Serialize>(&self,data: S) -> anyhow::Result<Self::Data> {
        match serde_json::to_string(&data) {
            Ok(result) => Ok(result),
            Err(err) => bail!(err),
        }
    }

    fn deserialize<'a, D: serde::Deserialize<'a>>(data: D) -> Self::Data {
        todo!()
    }
}

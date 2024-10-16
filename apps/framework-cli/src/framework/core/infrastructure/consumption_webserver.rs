use crate::proto::infrastructure_map::ConsumptionApiWebServer as ProtoConsumptionApiWebServer;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumptionApiWebServer {}

impl ConsumptionApiWebServer {
    pub fn to_proto(&self) -> ProtoConsumptionApiWebServer {
        ProtoConsumptionApiWebServer {
            special_fields: Default::default(),
        }
    }
}

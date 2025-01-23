use crate::framework::core::infrastructure::table::ColumnType;
use crate::proto::infrastructure_map::ConsumptionQueryParam as ProtoConsumptionQueryParam;
use hex::encode;
use protobuf::MessageField;
use serde::{Deserialize, Serialize};
use sha2::{digest::Output, Sha256};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConsumptionQueryParam {
    pub name: String,
    pub data_type: ColumnType,
    pub required: bool,
}

impl ConsumptionQueryParam {
    pub fn to_proto(&self) -> ProtoConsumptionQueryParam {
        ProtoConsumptionQueryParam {
            name: self.name.clone(),
            data_type: MessageField::some(self.data_type.to_proto()),
            required: self.required,
            special_fields: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct EndpointFile {
    pub path: PathBuf,
    pub hash: Output<Sha256>,
    pub query_params: Vec<ConsumptionQueryParam>,
}

impl EndpointFile {
    pub fn id(&self) -> String {
        format!("{}-{}", self.path.to_string_lossy(), encode(self.hash))
    }
}

#[derive(Debug, Clone, Default)]
pub struct Consumption {
    pub endpoint_files: Vec<EndpointFile>,
}

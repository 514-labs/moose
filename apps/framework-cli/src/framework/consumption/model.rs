use crate::framework::core::infrastructure::table::ColumnType;
use hex::encode;
use serde::{Deserialize, Serialize};
use sha2::{digest::Output, Sha256};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConsumptionQueryParam {
    pub name: String,
    pub data_type: ColumnType,
    pub required: bool,
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

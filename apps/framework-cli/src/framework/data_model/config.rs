use std::collections::{HashMap, HashSet};
use std::path::Path;

use log::info;
use serde::Deserialize;
use serde::Serialize;
use std::ffi::OsStr;

use crate::framework::typescript::export_collectors::get_data_model_configs;

pub type ConfigIdentifier = String;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum EndpointIngestionFormat {
    #[serde(alias = "JSON", alias = "json")]
    Json,
    #[serde(alias = "JSON_ARRAY", alias = "jsonArray")]
    JsonArray,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct IngestionConfig {
    pub format: EndpointIngestionFormat,
}

impl Default for IngestionConfig {
    fn default() -> Self {
        Self {
            format: EndpointIngestionFormat::Json,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct StorageConfig {
    #[serde(default = "_true")]
    pub enabled: bool,
    #[serde(default)]
    pub order_by_fields: Vec<String>,
}
const fn _true() -> bool {
    true
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            order_by_fields: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct DataModelConfig {
    #[serde(default)]
    pub ingestion: IngestionConfig,
    #[serde(default)]
    pub storage: StorageConfig,
}

#[derive(Debug, thiserror::Error)]
#[error("Failed to get the Data Model configuration")]
#[non_exhaustive]
pub enum ModelConfigurationError {
    TypescriptRunner(#[from] crate::framework::typescript::export_collectors::ExportCollectorError),
}

pub fn get(
    path: &Path,
    enums: HashSet<&str>,
) -> Result<HashMap<ConfigIdentifier, DataModelConfig>, ModelConfigurationError> {
    if path.extension() == Some(OsStr::new("ts")) {
        let config = get_data_model_configs(path, enums)?;
        info!("Data Model configuration for {:?}: {:?}", path, config);
        Ok(config)
    } else {
        // We currently fail transparently if the file is not a typescript file and
        // we will use defaults values for the configuration for each data model.
        Ok(HashMap::new())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_partial_config() {
        let config: super::DataModelConfig =
            serde_json::from_str("{\"storage\":{\"enabled\": true}}").unwrap();
        println!("{:?}", config)
    }
}

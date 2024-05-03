use std::collections::HashMap;
use std::path::Path;

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

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct DataModelConfig {
    pub ingestion: IngestionConfig,
}

impl Default for DataModelConfig {
    fn default() -> Self {
        Self {
            ingestion: IngestionConfig {
                format: EndpointIngestionFormat::Json,
            },
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Failed to get the Data Model configuration")]
#[non_exhaustive]
pub enum ModelConfigurationError {
    TypescriptRunner(#[from] crate::framework::typescript::export_collectors::ExportCollectorError),
}

pub fn get(
    path: &Path,
) -> Result<HashMap<ConfigIdentifier, DataModelConfig>, ModelConfigurationError> {
    if path.extension() == Some(OsStr::new("ts")) {
        Ok(get_data_model_configs(path)?)
    } else {
        // We currently fail transparently if the file is not a typescript file and
        // we will use defaults values for the configuration for each data model.
        Ok(HashMap::new())
    }
}

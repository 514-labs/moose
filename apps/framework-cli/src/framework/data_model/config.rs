use std::collections::{HashMap, HashSet};
use std::path::Path;

use tokio::io::AsyncReadExt;

use log::info;
use serde::Deserialize;
use serde::Serialize;
use std::ffi::OsStr;

use crate::framework::typescript::export_collectors::get_data_model_configs;

use crate::framework::python::executor::run_python_file;

pub type ConfigIdentifier = String;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum EndpointIngestionFormat {
    #[serde(alias = "JSON", alias = "json")]
    Json,
    #[serde(alias = "JSON_ARRAY", alias = "jsonArray")]
    JsonArray,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default, Hash)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default, Hash)]
pub struct PythonDataModelConfig {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub config: DataModelConfig,
}

async fn parse_python_model_file(
    path: &Path,
) -> Result<HashMap<ConfigIdentifier, DataModelConfig>, ()> {
    let process = run_python_file(path).await.map_err(|_| ())?;

    let mut stdout = match process.stdout {
        Some(handle) => handle,
        None => return Ok(HashMap::new()),
    };

    let mut raw_string_stdout: String = String::new();

    if stdout.read_to_string(&mut raw_string_stdout).await.is_err() {
        return Ok(HashMap::new());
    }
    println!("config from py lib: {:?}", raw_string_stdout);

    let configs: HashMap<ConfigIdentifier, DataModelConfig> = raw_string_stdout
        .split("___DATAMODELCONFIG___")
        .filter_map(|entry| {
            let raw_value = serde_json::from_str(entry).ok()?;
            let config: PythonDataModelConfig = serde_json::from_value(raw_value).ok()?;
            Some((config.name.clone(), config.config))
        })
        .collect();

    println!("Parsed PythonDataModelConfig: {:?}", configs);

    Ok(configs)
}

pub async fn get(
    path: &Path,
    enums: HashSet<&str>,
) -> Result<HashMap<ConfigIdentifier, DataModelConfig>, ModelConfigurationError> {
    println!("config.get path: {:?}", path);
    if path.extension() == Some(OsStr::new("ts")) {
        let config = get_data_model_configs(path, enums).await?;
        println!("config.get config: {:?}", config);
        info!("Data Model configuration for {:?}: {:?}", path, config);
        Ok(config)
    } else if path.extension() == Some(OsStr::new("py"))
        && path.file_name() != Some(OsStr::new("__init__.py"))
    {
        match parse_python_model_file(path).await {
            Ok(result) => return Ok(result),
            Err(_) => return Ok(HashMap::new()),
        }
    } else {
        // We will use defaults values for the configuration for each data model.
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

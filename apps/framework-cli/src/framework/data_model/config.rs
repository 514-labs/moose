use std::collections::{HashMap, HashSet};
use std::path::{absolute, Path};

use tokio::io::AsyncReadExt;

use crate::framework::typescript::export_collectors::get_data_model_configs;
use log::{info, warn};
use serde::Deserialize;
use serde::Serialize;
use std::ffi::OsStr;

use crate::framework::python::executor::run_python_file;

pub type ConfigIdentifier = String;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
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
    PythonRunner(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default, Hash)]
pub struct PythonDataModelConfig {
    #[serde(default)]
    pub class_name: String,
    #[serde(default)]
    pub config: DataModelConfig,
}

async fn execute_python_model_file_for_config(
    path: &Path,
) -> Result<HashMap<ConfigIdentifier, DataModelConfig>, ModelConfigurationError> {
    let abs_path = path.canonicalize().or_else(|_| absolute(path));
    let path_str = abs_path.as_deref().unwrap_or(path).to_string_lossy();
    let process = run_python_file(path, &[("MOOSE_PYTHON_DM_DUMP", &*path_str)])
        .await
        .map_err(|e| ModelConfigurationError::PythonRunner(e.to_string()))?;

    let mut stdout = match process.stdout {
        Some(handle) => handle,
        None => return Ok(HashMap::new()),
    };

    let mut raw_string_stdout: String = String::new();

    if stdout.read_to_string(&mut raw_string_stdout).await.is_err() {
        warn!("Unable to read stdout for python config dump");
        return Ok(HashMap::new());
    }

    let configs: HashMap<ConfigIdentifier, DataModelConfig> = raw_string_stdout
        .split("___DATAMODELCONFIG___")
        .filter_map(|entry| {
            let config = serde_json::from_str::<PythonDataModelConfig>(entry).ok()?;
            Some((config.class_name, config.config))
        })
        .collect();

    info!("Data Model configuration for {:?}: {:?}", path, configs);

    Ok(configs)
}

pub async fn get(
    path: &Path,
    project_path: &Path,
    enums: HashSet<&str>,
) -> Result<HashMap<ConfigIdentifier, DataModelConfig>, ModelConfigurationError> {
    if path.extension() == Some(OsStr::new("ts")) {
        let config = get_data_model_configs(path, project_path, enums).await?;
        info!("Data Model configuration for {:?}: {:?}", path, config);
        Ok(config)
    } else if path.extension() == Some(OsStr::new("py"))
        && path.file_name() != Some(OsStr::new("__init__.py"))
    {
        return execute_python_model_file_for_config(path).await;
    } else {
        // We will use defaults values for the configuration for each data model.
        Ok(HashMap::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partial_config() {
        let config: DataModelConfig =
            serde_json::from_str("{\"storage\":{\"enabled\": true}}").unwrap();
        println!("{:?}", config)
    }
}

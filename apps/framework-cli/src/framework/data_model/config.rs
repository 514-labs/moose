use std::collections::{HashMap, HashSet};
use std::path::Path;

use tokio::io::AsyncReadExt;

use log::info;
use serde::Deserialize;
use serde::Serialize;
use std::ffi::OsStr;

use crate::framework::typescript::export_collectors::get_data_model_configs;

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

// TODO: Clean up all of this

use crate::utilities::constants::{CLI_INTERNAL_VERSIONS_DIR, CLI_PROJECT_INTERNAL_DIR};
const PYTHON_PATH: &str = "PYTHONPATH";
fn python_path_with_version() -> String {
    let mut paths = std::env::var(PYTHON_PATH).unwrap_or_else(|_| String::from(""));
    if !paths.is_empty() {
        paths.push(':');
    }
    paths.push_str(CLI_PROJECT_INTERNAL_DIR);
    paths.push('/');
    paths.push_str(CLI_INTERNAL_VERSIONS_DIR);
    paths
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
    let mut command = tokio::process::Command::new("python3");
    let process = command
        .env(PYTHON_PATH, python_path_with_version())
        .arg("-u")
        .arg(path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|_| ())?;

    let mut stdout = process
        .stdout
        .expect("Python process did not have a handle to stdout");

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

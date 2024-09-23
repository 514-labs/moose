use std::collections::{HashMap, HashSet};
use std::path::Path;

use serde_json::Value;
use tokio::io::AsyncReadExt;

use super::bin;
use crate::framework::data_model::config::{ConfigIdentifier, DataModelConfig};

const EXPORT_SERIALIZER_BIN: &str = "export-serializer";

#[derive(Debug, thiserror::Error)]
#[error("Failed to run code")]
#[non_exhaustive]
pub enum ExportCollectorError {
    Tokio(#[from] tokio::io::Error),
    JsonParsing(#[from] serde_json::Error),
    #[error("{message}")]
    Other {
        message: String,
    },
}

async fn collect_exports(file: &Path, project_path: &Path) -> Result<Value, ExportCollectorError> {
    let file_path_str = file.to_str().ok_or(ExportCollectorError::Other {
        message: "Did not get a proper file path to load exports from".to_string(),
    })?;

    let args = vec![file_path_str];
    let process = bin::run(EXPORT_SERIALIZER_BIN, project_path, &args)?;

    let mut stdout = process
        .stdout
        .expect("Data model config process did not have a handle to stdout");

    let mut stderr = process
        .stderr
        .expect("Data model config process did not have a handle to stderr");

    let mut raw_string_stderr: String = String::new();
    stderr.read_to_string(&mut raw_string_stderr).await?;

    if !raw_string_stderr.is_empty() {
        Err(ExportCollectorError::Other {
            message: format!(
                "Error collecting exports in the file {:?}: \n{}",
                file, raw_string_stderr
            ),
        })
    } else {
        let mut raw_string_stdout: String = String::new();
        stdout.read_to_string(&mut raw_string_stdout).await?;

        Ok(serde_json::from_str(&raw_string_stdout)?)
    }
}

pub async fn get_data_model_configs(
    file: &Path,
    project_path: &Path,
    enums: HashSet<&str>,
) -> Result<HashMap<ConfigIdentifier, DataModelConfig>, ExportCollectorError> {
    let exports = collect_exports(file, project_path).await?;

    match exports {
        Value::Object(map) => {
            let mut result = HashMap::new();
            for (key, value) in map {
                if enums.contains(key.as_str()) {
                    continue;
                }
                if let Ok(model_config) = serde_json::from_value(value) {
                    result.insert(key, model_config);
                }
            }
            Ok(result)
        }
        _ => Err(ExportCollectorError::Other {
            message: "Expected an object as the root of the exports".to_string(),
        }),
    }
}

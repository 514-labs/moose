use std::collections::HashMap;
use std::io::Read;
use std::{
    io::BufReader,
    path::Path,
    process::{Command, Stdio},
};

use serde_json::Value;

use crate::framework::data_model::config::{ConfigIdentifier, DataModelConfig};

const MODULE_EXPORT_SERIALIZER: &str = include_str!("ts_scripts/moduleExportSerializer.ts");

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

fn collect_std_to_string<R: Read>(container: R) -> Result<String, ExportCollectorError> {
    let mut reader = BufReader::new(container);
    let mut result = String::new();
    reader.read_to_string(&mut result)?;
    Ok(result)
}

fn collect_exports(file: &Path) -> Result<Value, ExportCollectorError> {
    let file_path_str = &file.to_str().ok_or(ExportCollectorError::Other {
        message: "Did not get a proper file path to load exports from".to_string(),
    })?;

    let process = Command::new("npx")
        .arg("--yes")
        .arg("ts-node")
        .arg("--skipProject")
        .arg("-e")
        .arg(MODULE_EXPORT_SERIALIZER)
        .arg("--")
        .arg(file_path_str)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed start data model config process");

    let stdout = process
        .stdout
        .expect("Data model config process did not have a handle to stdout");
    let stderr = process
        .stderr
        .expect("Data model config process did not have a handle to stderr");

    let raw_string_stderr = collect_std_to_string(stderr)?;
    if !raw_string_stderr.is_empty() {
        Err(ExportCollectorError::Other {
            message: format!(
                "Error collecting exports in the file {:?}: \n{}",
                file, raw_string_stderr
            ),
        })
    } else {
        let raw_string_stdout = collect_std_to_string(stdout)?;

        Ok(serde_json::from_str(&raw_string_stdout)?)
    }
}

pub fn get_data_model_configs(
    file: &Path,
) -> Result<HashMap<ConfigIdentifier, DataModelConfig>, ExportCollectorError> {
    let exports = collect_exports(file)?;

    match exports {
        Value::Object(map) => {
            let mut result = HashMap::new();
            for (key, value) in map {
                let _ = serde_json::from_value(value).map(|model_config| {
                    result.insert(key, model_config);
                });
            }
            Ok(result)
        }
        _ => Err(ExportCollectorError::Other {
            message: "Expected an object as the root of the exports".to_string(),
        }),
    }
}

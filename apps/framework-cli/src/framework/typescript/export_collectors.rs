use super::bin;
use crate::framework::consumption::model::ConsumptionQueryParam;
use crate::framework::core::infrastructure::table::ColumnType;
use crate::framework::data_model::config::{ConfigIdentifier, DataModelConfig};
use log::debug;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use tokio::io::AsyncReadExt;

const EXPORT_SERIALIZER_BIN: &str = "export-serializer";
const EXPORT_FUNC_TYPE_BIN: &str = "consumption-type-serializer";

const EXPORT_CONFIG_PROCESS: &str = "Data model config";
const EXPORT_FUNC_TYPE_PROCESS: &str = "API schema";

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

async fn collect_exports(
    command_name: &str,
    process_name: &str,
    file: &Path,
    project_path: &Path,
) -> Result<Value, ExportCollectorError> {
    let file_path_str = file.to_str().ok_or(ExportCollectorError::Other {
        message: "Did not get a proper file path to load exports from".to_string(),
    })?;

    let args = vec![file_path_str];
    let process = bin::run(command_name, project_path, &args)?;

    let mut stdout = process
        .stdout
        .unwrap_or_else(|| panic!("{process_name} process did not have a handle to stdout"));

    let mut stderr = process
        .stderr
        .unwrap_or_else(|| panic!("{process_name} process did not have a handle to stderr"));

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

        println!("raw_string_stdout {}", raw_string_stdout);

        Ok(serde_json::from_str(&raw_string_stdout)?)
    }
}

pub async fn get_data_model_configs(
    file: &Path,
    project_path: &Path,
    enums: HashSet<&str>,
) -> Result<HashMap<ConfigIdentifier, DataModelConfig>, ExportCollectorError> {
    let exports = collect_exports(
        EXPORT_SERIALIZER_BIN,
        EXPORT_CONFIG_PROCESS,
        file,
        project_path,
    )
    .await?;

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

pub async fn get_func_types(
    file: &Path,
    project_path: &Path,
) -> Result<Vec<ConsumptionQueryParam>, ExportCollectorError> {
    let exports = collect_exports(
        EXPORT_FUNC_TYPE_BIN,
        EXPORT_FUNC_TYPE_PROCESS,
        file,
        project_path,
    )
    .await?;

    debug!("Schema for path {}", exports);

    let converted = match exports {
        Value::Null => vec![],
        Value::Object(map) => {
            let schema = map
                .get("components")
                .and_then(|o| o.as_object())
                .and_then(|m| m.get("schemas"))
                .and_then(|o| o.as_object())
                .ok_or_else(|| ExportCollectorError::Other {
                    message: "Unexpected schema shape.".to_string(),
                })?;
            let (_name, schema) = if schema.len() == 1 {
                schema.iter().next().unwrap()
            } else {
                return Err(ExportCollectorError::Other {
                    message: "More than one schema.".to_string(),
                });
            };

            let required_keys = schema
                .as_object()
                .and_then(|m| m.get("required"))
                .and_then(|v| v.as_array());

            schema
                .as_object()
                .and_then(|m| m.get("properties"))
                .and_then(|o| o.as_object())
                .ok_or_else(|| ExportCollectorError::Other {
                    message: "Missing properties in schema.".to_string(),
                })?
                .iter()
                .map(|(k, v)| {
                    let type_object = v.as_object();
                    let data_type = match type_object
                        .and_then(|m| m.get("type"))
                        .and_then(|v| v.as_str())
                    {
                        Some("string") => ColumnType::String,
                        Some("number") => ColumnType::Float,
                        Some("integer") => ColumnType::Int,
                        Some("boolean") => ColumnType::Boolean,
                        // no recursion here, query param does not support nested arrays anyway
                        Some("array") => {
                            let inner_type = match type_object
                                .unwrap()
                                .get("items")
                                .and_then(|v| v.as_object())
                                .and_then(|m| m.get("type"))
                                .and_then(|v| v.as_str())
                            {
                                Some("number") => ColumnType::Float,
                                Some("integer") => ColumnType::Int,
                                Some("boolean") => ColumnType::Boolean,
                                Some("string") | _ => ColumnType::String,
                            };
                            ColumnType::Array(Box::new(inner_type))
                        }

                        unexpected => {
                            debug!("unexpected type {:?} for field {k}", unexpected);
                            ColumnType::String
                        }
                    };

                    ConsumptionQueryParam {
                        name: k.to_string(),
                        data_type,
                        required: required_keys
                            .is_some_and(|arr| arr.iter().any(|v| v.as_str() == Some(k))),
                    }
                })
                .collect()
        }
        _ => Err(ExportCollectorError::Other {
            message: "Expected an object as the root of the exports".to_string(),
        })?,
    };
    Ok(converted)
}

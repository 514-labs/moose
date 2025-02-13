use crate::framework::consumption::model::ConsumptionQueryParam;
use crate::framework::core::infrastructure::table::ColumnType;
use crate::framework::typescript::export_collectors::ExportCollectorError;
use crate::infrastructure::olap::clickhouse::config::ClickHouseConfig;
use crate::infrastructure::processes::consumption_registry::ConsumptionError;
use crate::project::JwtConfig;
use log::{debug, error, info};
use serde_json::{Map, Value};
use std::path::Path;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;

use super::bin;

const CONSUMPTION_RUNNER_BIN: &str = "consumption-apis";

// TODO: Abstract away ClickhouseConfig to support other databases
// TODO: Bubble up compilation errors to the user
pub fn run(
    clickhouse_config: ClickHouseConfig,
    jwt_config: Option<JwtConfig>,
    consumption_path: &Path,
    project_path: &Path,
) -> Result<Child, ConsumptionError> {
    let host_port = clickhouse_config.host_port.to_string();
    let use_ssl = clickhouse_config.use_ssl.to_string();

    let jwt_secret = jwt_config
        .as_ref()
        .map(|jwt| jwt.secret.clone())
        .unwrap_or("".to_string());

    let jwt_issuer = jwt_config
        .as_ref()
        .map(|jwt| jwt.issuer.clone())
        .unwrap_or("".to_string());

    let jwt_audience = jwt_config
        .as_ref()
        .map(|jwt| jwt.audience.clone())
        .unwrap_or("".to_string());

    let enforce_on_all_consumptions_apis = jwt_config
        .as_ref()
        .map(|jwt| jwt.enforce_on_all_consumptions_apis.to_string())
        .unwrap_or("false".to_string());

    let args = vec![
        consumption_path.to_str().unwrap(),
        &clickhouse_config.db_name,
        &clickhouse_config.host,
        &host_port,
        &clickhouse_config.user,
        &clickhouse_config.password,
        &use_ssl,
        &jwt_secret,
        &jwt_issuer,
        &jwt_audience,
        &enforce_on_all_consumptions_apis,
    ];

    let mut consumption_process = bin::run(CONSUMPTION_RUNNER_BIN, project_path, &args)?;

    let stdout = consumption_process
        .stdout
        .take()
        .expect("Consumption process did not have a handle to stdout");

    let stderr = consumption_process
        .stderr
        .take()
        .expect("Consumption process did not have a handle to stderr");

    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    tokio::spawn(async move {
        while let Ok(Some(line)) = stdout_reader.next_line().await {
            info!("{}", line);
        }
    });

    tokio::spawn(async move {
        while let Ok(Some(line)) = stderr_reader.next_line().await {
            error!("{}", line);
        }
    });

    Ok(consumption_process)
}

fn schema_to_params_list(
    schema: &Value,
) -> Result<Vec<ConsumptionQueryParam>, ExportCollectorError> {
    let required_keys = schema
        .as_object()
        .and_then(|m| m.get("required"))
        .and_then(|v| v.as_array());

    let converted = schema
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
                // no recursion here, query param does not support nested arrays
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
                        _ => ColumnType::String,
                    };
                    ColumnType::Array {
                        element_type: Box::new(inner_type),
                        element_nullable: false,
                    }
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
        .collect();
    Ok(converted)
}

pub fn extract_schema(json_schema: &Map<String, Value>) -> Result<&Value, ExportCollectorError> {
    let schemas = json_schema
        .get("schemas")
        .and_then(|o| o.as_array())
        .ok_or_else(|| ExportCollectorError::Other {
            message: "Unexpected schema shape.".to_string(),
        })?;

    let schema = if schemas.len() == 1 {
        schemas.iter().next().unwrap()
    } else {
        return Err(ExportCollectorError::Other {
            message: format!("Unexpected number of schemas: {}", schemas.len()),
        });
    };

    let schema_deref = if let Some(Value::String(s)) = schema.get("$ref") {
        let components_schemas = json_schema
            .get("components")
            .and_then(|o| o.as_object())
            .and_then(|m| m.get("schemas"))
            .and_then(|o| o.as_object())
            .ok_or_else(|| ExportCollectorError::Other {
                message: "Unexpected schema shape.".to_string(),
            })?;
        components_schemas
            .get(s.strip_prefix("#/components/schemas/").unwrap_or(s))
            .ok_or_else(|| ExportCollectorError::Other {
                message: format!("Schema {s} not found."),
            })?
    } else {
        schema
    };
    Ok(schema_deref)
}

pub fn extract_intput_param(
    map: &Map<String, Value>,
) -> Result<Vec<ConsumptionQueryParam>, ExportCollectorError> {
    let input_schema = map
        .get("inputSchema")
        .and_then(|o| o.as_object())
        .ok_or_else(|| ExportCollectorError::Other {
            message: "inputSchema field should be an object.".to_string(),
        })?;

    schema_to_params_list(extract_schema(input_schema)?)
}

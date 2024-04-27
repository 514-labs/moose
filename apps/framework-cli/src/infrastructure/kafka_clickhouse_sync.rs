use std::collections::HashMap;

use crate::framework::controller::FrameworkObject;
use crate::framework::controller::FrameworkObjectVersions;
use crate::infrastructure::olap::clickhouse::config::ClickHouseConfig;
use crate::infrastructure::olap::clickhouse::inserter::Inserter;
use crate::infrastructure::olap::clickhouse::model::ClickHouseValue;
use crate::infrastructure::stream::redpanda::create_subscriber;
use crate::infrastructure::stream::redpanda::RedpandaConfig;
use log::error;
use log::info;
use rdkafka::Message;

use log::debug;
use serde_json::Value;
use tokio::task::JoinHandle;

use super::olap::clickhouse::model::ClickHouseColumn;
use super::olap::clickhouse::model::ClickHouseColumnType;
use super::olap::clickhouse::model::ClickHouseRecord;
use super::olap::clickhouse::model::ClickHouseRuntimeEnum;

const SYNC_GROUP_ID: &str = "clickhouse_sync";

struct SyncingProcess {
    process: JoinHandle<anyhow::Result<()>>,
    topic: String,
    table: String,
}

pub struct SyncingProcessesRegistry {
    registry: HashMap<String, JoinHandle<anyhow::Result<()>>>,
    kafka_config: RedpandaConfig,
    clickhouse_config: ClickHouseConfig,
}

impl SyncingProcessesRegistry {
    pub fn new(kafka_config: RedpandaConfig, clickhouse_config: ClickHouseConfig) -> Self {
        Self {
            registry: HashMap::new(),
            kafka_config,
            clickhouse_config,
        }
    }

    fn format_key(syncing_process: &SyncingProcess) -> String {
        Self::format_key_str(&syncing_process.topic, &syncing_process.table)
    }

    fn format_key_str(topic: &str, table: &str) -> String {
        format!("{}-{}", topic, table)
    }

    fn format_key_framework_obj(framework_object: &FrameworkObject) -> String {
        Self::format_key_str(&framework_object.topic, &framework_object.table.name)
    }

    fn insert(&mut self, syncing_process: SyncingProcess) {
        let key = Self::format_key(&syncing_process);
        self.registry.insert(key, syncing_process.process);
    }

    pub fn start_all(&mut self, framework_object_versions: &FrameworkObjectVersions) {
        info!("<DCM> Starting all syncing processes");

        let kafka_config = self.kafka_config.clone();
        let clickhouse_config = self.clickhouse_config.clone();

        // Spawn sync for the current models
        let current_object_iterator = framework_object_versions
            .current_models
            .models
            .clone()
            .into_iter()
            .map(spawn_sync_process(
                kafka_config.clone(),
                clickhouse_config.clone(),
            ));

        let previous_versions_iterator = framework_object_versions
            .previous_version_models
            .values()
            .flat_map(|schema_version| {
                let schema_version_cloned = schema_version.models.clone();

                schema_version_cloned.into_iter().map(spawn_sync_process(
                    kafka_config.clone(),
                    clickhouse_config.clone(),
                ))
            });

        for syncing_process in current_object_iterator.chain(previous_versions_iterator) {
            self.insert(syncing_process);
        }
    }

    pub fn start(&mut self, framework_object: &FrameworkObject) {
        info!(
            "<DCM> Starting syncing process for topic: {} and table: {}",
            framework_object.topic, framework_object.table.name
        );
        let key = Self::format_key_framework_obj(framework_object);

        // the schema of the currently running process is outdated
        if let Some(process) = self.registry.remove(&key) {
            process.abort();
        }

        let syncing_process = spawn_sync_process_core(
            self.kafka_config.clone(),
            self.clickhouse_config.clone(),
            framework_object.clone(),
        );

        self.insert(syncing_process);
    }

    pub fn stop(&mut self, framework_object: &FrameworkObject) {
        let key = Self::format_key_framework_obj(framework_object);
        if let Some(process) = self.registry.remove(&key) {
            process.abort();
        }
    }
}

fn spawn_sync_process(
    kafka_config: RedpandaConfig,
    clickhouse_config: ClickHouseConfig,
) -> Box<dyn Fn((String, FrameworkObject)) -> SyncingProcess> {
    Box::new(move |(_, schema)| {
        spawn_sync_process_core(
            kafka_config.clone(),
            clickhouse_config.clone(),
            schema.clone(),
        )
    })
}

fn spawn_sync_process_core(
    kafka_config: RedpandaConfig,
    clickhouse_config: ClickHouseConfig,
    framework_object: FrameworkObject,
) -> SyncingProcess {
    let syncing_process = tokio::spawn(sync_kafka_to_clickhouse(
        kafka_config,
        clickhouse_config,
        framework_object.clone(),
    ));

    SyncingProcess {
        process: syncing_process,
        topic: framework_object.topic.clone(),
        table: framework_object.table.name.clone(),
    }
}

async fn sync_kafka_to_clickhouse(
    kafka_config: RedpandaConfig,
    clickhouse_config: ClickHouseConfig,
    framework_object: FrameworkObject,
) -> anyhow::Result<()> {
    let topic = &framework_object.topic;
    let subscriber = create_subscriber(&kafka_config, SYNC_GROUP_ID, topic);

    let clikhouse_columns = framework_object
        .table
        .columns
        .iter()
        .map(|column| column.name.clone())
        .collect();

    let inserter = Inserter::new(
        clickhouse_config,
        framework_object.table.name.clone(),
        clikhouse_columns,
    );

    // WARNING: the code below is very performance sensitive
    // it is run for every message that needs to be written to clickhouse. As such we should
    // optimize as much as possible

    // In that context we would rather not interupt the syncing process if an error occurs.
    // The user doesn't currently get feedback on the error since the process is burried behind the scenes and
    // asynchronous.

    loop {
        match subscriber.recv().await {
            Err(e) => {
                debug!("Error receiving message from {}: {}", topic, e);
            }

            Ok(message) => match message.payload() {
                Some(payload) => match std::str::from_utf8(payload) {
                    Ok(payload_str) => {
                        debug!("Received message from {}: {}", topic, payload_str);

                        let parsed_json: Value = serde_json::from_str(payload_str)?;
                        let clickhouse_record = mapper_json_to_clickhouse_record(
                            &framework_object.table.columns,
                            parsed_json,
                        )?;

                        inserter.insert(clickhouse_record).await?;
                    }
                    Err(_) => {
                        error!("Received message from {} with invalid UTF-8", topic);
                    }
                },
                None => {
                    debug!("Received message from {} with no payload", topic);
                }
            },
        }
    }
}

fn mapper_json_to_clickhouse_record(
    clickhouse_columns: &[ClickHouseColumn],
    json_value: Value,
) -> anyhow::Result<ClickHouseRecord> {
    match json_value {
        Value::Object(map) => {
            let mut record = ClickHouseRecord::new();

            for column in clickhouse_columns.iter() {
                let key = column.name.clone();
                let value = map.get(&key);

                match value {
                    Some(value) => {
                        match map_json_value_to_clickhouse_value(&column.column_type, value) {
                            Ok(clickhouse_value) => {
                                record.insert(key, clickhouse_value);
                            }
                            Err(e) => {
                                log::debug!("Error mapping JSON value to ClickHouse value: {}", e)
                            }
                        };
                    }
                    None => {
                        record.insert(key, ClickHouseValue::new_string("NULL".to_string()));
                    }
                }
            }

            Ok(record)
        }
        _ => Err(anyhow::anyhow!("Invalid JSON")),
    }
}

#[derive(Debug, thiserror::Error)]
enum MappingError {
    #[error("Failed to map the JSON value {value:?} to ClickHouse column typed {column_type:?}")]
    TypeMismatchError {
        column_type: ClickHouseColumnType,
        value: Value,
    },
    #[error("The Column Type {column_type:?} is not supported")]
    UnsupportedColumnTypeError { column_type: ClickHouseColumnType },
}

fn map_json_value_to_clickhouse_value(
    column_type: &ClickHouseColumnType,
    value: &Value,
) -> Result<ClickHouseValue, MappingError> {
    match column_type {
        ClickHouseColumnType::String => {
            if let Some(value_str) = value.as_str() {
                Ok(ClickHouseValue::new_string(value_str.to_string()))
            } else {
                Err(MappingError::TypeMismatchError {
                    column_type: column_type.clone(),
                    value: value.clone(),
                })
            }
        }
        ClickHouseColumnType::Boolean => {
            if let Some(value_bool) = value.as_bool() {
                Ok(ClickHouseValue::new_boolean(value_bool))
            } else {
                Err(MappingError::TypeMismatchError {
                    column_type: column_type.clone(),
                    value: value.clone(),
                })
            }
        }
        ClickHouseColumnType::ClickhouseInt(_) => {
            if let Some(value_int) = value.as_i64() {
                Ok(ClickHouseValue::new_int_64(value_int))
            } else {
                Err(MappingError::TypeMismatchError {
                    column_type: column_type.clone(),
                    value: value.clone(),
                })
            }
        }
        ClickHouseColumnType::ClickhouseFloat(_) => {
            if let Some(value_float) = value.as_f64() {
                Ok(ClickHouseValue::new_float_64(value_float))
            } else {
                Err(MappingError::TypeMismatchError {
                    column_type: column_type.clone(),
                    value: value.clone(),
                })
            }
        }
        ClickHouseColumnType::Decimal => Err(MappingError::UnsupportedColumnTypeError {
            column_type: column_type.clone(),
        }),
        ClickHouseColumnType::DateTime => {
            if let Some(value_str) = value.as_str() {
                if let Ok(date_time) = chrono::DateTime::parse_from_rfc3339(value_str) {
                    Ok(ClickHouseValue::new_date_time(date_time))
                } else {
                    Err(MappingError::TypeMismatchError {
                        column_type: column_type.clone(),
                        value: value.clone(),
                    })
                }
            } else {
                Err(MappingError::TypeMismatchError {
                    column_type: column_type.clone(),
                    value: value.clone(),
                })
            }
        }
        ClickHouseColumnType::Enum(x) => {
            // In this context what could be coming in could be a string or a number depending on
            // how the enum was defined at the source.
            if let Some(value_str) = value.as_str() {
                Ok(ClickHouseValue::new_enum(
                    ClickHouseRuntimeEnum::ClickHouseString(value_str.to_string()),
                    x.clone(),
                ))
            } else if let Some(value_int) = value.as_i64() {
                Ok(ClickHouseValue::new_enum(
                    ClickHouseRuntimeEnum::ClickHouseInt(value_int as u8),
                    x.clone(),
                ))
            } else {
                Err(MappingError::TypeMismatchError {
                    column_type: column_type.clone(),
                    value: value.clone(),
                })
            }
        }
        ClickHouseColumnType::Array(inner_clickhouse_type) => {
            if let Some(arr) = value.as_array() {
                let mut array_values = Vec::new();
                let array_type = *inner_clickhouse_type.clone();
                for value in arr.iter() {
                    let clickhouse_value = map_json_value_to_clickhouse_value(&array_type, value)?;
                    array_values.push(clickhouse_value);
                }

                Ok(ClickHouseValue::new_array(
                    array_values,
                    *inner_clickhouse_type.clone(),
                ))
            } else {
                Err(MappingError::TypeMismatchError {
                    column_type: column_type.clone(),
                    value: value.clone(),
                })
            }
        }
        ClickHouseColumnType::Json => Err(MappingError::UnsupportedColumnTypeError {
            column_type: column_type.clone(),
        }),
        ClickHouseColumnType::Bytes => Err(MappingError::UnsupportedColumnTypeError {
            column_type: column_type.clone(),
        }),
    }
}

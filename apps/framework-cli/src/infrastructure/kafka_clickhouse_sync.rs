use clickhouse_rs::types::column;
use log::error;
use log::info;
use rdkafka::Message;
use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::RandomState;

use log::debug;
use serde_json::Value;
use tokio::task::JoinHandle;

use super::olap::clickhouse::errors::ClickhouseError;
use super::olap::clickhouse::mapper::std_field_type_to_clickhouse_type_mapper;
use super::olap::clickhouse::model::ClickHouseColumn;
use super::olap::clickhouse::model::ClickHouseColumnType;
use super::olap::clickhouse::model::ClickHouseRecord;
use super::olap::clickhouse::model::ClickHouseRuntimeEnum;
use super::olap::clickhouse::version_sync::VersionSync;
use crate::framework::controller::FrameworkObjectVersions;
use crate::framework::data_model::schema::Column;
use crate::framework::data_model::schema::ColumnType;
use crate::framework::data_model::schema::Nested;
use crate::infrastructure::olap::clickhouse::config::ClickHouseConfig;
use crate::infrastructure::olap::clickhouse::inserter::Inserter;
use crate::infrastructure::olap::clickhouse::model::ClickHouseValue;
use crate::infrastructure::olap::clickhouse::version_sync::VersionSyncType;
use crate::infrastructure::stream::redpanda::create_subscriber;
use crate::infrastructure::stream::redpanda::fetch_topics;
use crate::infrastructure::stream::redpanda::RedpandaConfig;

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

    fn insert(&mut self, syncing_process: SyncingProcess) {
        let key = Self::format_key(&syncing_process);
        self.registry.insert(key, syncing_process.process);
    }

    pub async fn start_all(
        &mut self,
        framework_object_versions: &FrameworkObjectVersions,
        version_syncs: &[VersionSync],
    ) -> Result<(), rdkafka::error::KafkaError> {
        info!("<DCM> Starting all syncing processes");

        let kafka_config = self.kafka_config.clone();
        let clickhouse_config = self.clickhouse_config.clone();

        let available_topics: HashSet<String, RandomState> =
            HashSet::from_iter(fetch_topics(&kafka_config).await?);

        // Spawn sync for the current models
        let current_object_iterator = framework_object_versions
            .current_models
            .models
            .clone()
            .into_iter()
            .filter_map(|(_, framework_object)| {
                if available_topics.contains(&framework_object.topic) {
                    framework_object.table.map(|table| {
                        (
                            framework_object.topic,
                            framework_object.data_model.columns,
                            table.name,
                            table.columns,
                        )
                    })
                } else {
                    None
                }
            })
            .map(spawn_sync_process(
                kafka_config.clone(),
                clickhouse_config.clone(),
            ));

        let previous_versions_iterator = framework_object_versions
            .previous_version_models
            .values()
            .flat_map(|schema_version| {
                let schema_version_cloned = schema_version.models.clone();

                schema_version_cloned
                    .into_iter()
                    .filter_map(|(_, framework_object)| {
                        if available_topics.contains(&framework_object.topic) {
                            framework_object.table.map(|table| {
                                (
                                    framework_object.topic,
                                    framework_object.data_model.columns,
                                    table.name,
                                    table.columns,
                                )
                            })
                        } else {
                            None
                        }
                    })
                    .map(spawn_sync_process(
                        kafka_config.clone(),
                        clickhouse_config.clone(),
                    ))
            });

        let version_syncs_iterator = version_syncs.iter().filter_map(|vs| match vs.sync_type {
            VersionSyncType::Sql(_) => None,
            VersionSyncType::Ts(_) => {
                let topic_name = vs.topic_name("output");
                if available_topics.contains(&topic_name) {
                    Some(spawn_sync_process_core(
                        kafka_config.clone(),
                        clickhouse_config.clone(),
                        topic_name,
                        vs.source_data_model.columns.clone(),
                        vs.dest_table.name.clone(),
                        vs.dest_table.columns.clone(),
                    ))
                } else {
                    None
                }
            }
        });

        for syncing_process in current_object_iterator
            .chain(previous_versions_iterator)
            .chain(version_syncs_iterator)
        {
            self.insert(syncing_process);
        }

        Ok(())
    }

    pub fn start(
        &mut self,
        source_topic_name: String,
        source_topic_columns: Vec<Column>,
        target_table_name: String,
        target_table_columns: Vec<ClickHouseColumn>,
    ) {
        info!(
            "<DCM> Starting syncing process for topic: {} and table: {}",
            source_topic_name, target_table_name
        );
        let key = Self::format_key_str(&source_topic_name, &target_table_name);

        // the schema of the currently running process is outdated
        if let Some(process) = self.registry.remove(&key) {
            process.abort();
        }

        let syncing_process = spawn_sync_process_core(
            self.kafka_config.clone(),
            self.clickhouse_config.clone(),
            source_topic_name,
            source_topic_columns,
            target_table_name,
            target_table_columns,
        );

        self.insert(syncing_process);
    }

    pub fn stop(&mut self, topic_name: &str, table_name: &str) {
        let key = Self::format_key_str(topic_name, table_name);
        if let Some(process) = self.registry.remove(&key) {
            process.abort();
        }
    }
}

type FnSyncProcess =
    Box<dyn Fn((String, Vec<Column>, String, Vec<ClickHouseColumn>)) -> SyncingProcess>;

fn spawn_sync_process(
    kafka_config: RedpandaConfig,
    clickhouse_config: ClickHouseConfig,
) -> FnSyncProcess {
    Box::new(
        move |(
            source_topic_name,
            source_topic_columns,
            target_table_name,
            target_table_columns,
        )| {
            info!(
                "Starting Kafka sync to clikchouse from topic: {} to table: {}",
                source_topic_name, target_table_name
            );
            spawn_sync_process_core(
                kafka_config.clone(),
                clickhouse_config.clone(),
                source_topic_name,
                source_topic_columns,
                target_table_name,
                target_table_columns,
            )
        },
    )
}

fn spawn_sync_process_core(
    kafka_config: RedpandaConfig,
    clickhouse_config: ClickHouseConfig,
    source_topic_name: String,
    source_topic_columns: Vec<Column>,
    target_table_name: String,
    target_table_columns: Vec<ClickHouseColumn>,
) -> SyncingProcess {
    let syncing_process = tokio::spawn(sync_kafka_to_clickhouse(
        kafka_config,
        clickhouse_config,
        source_topic_name.clone(),
        source_topic_columns,
        target_table_name.clone(),
        target_table_columns,
    ));

    SyncingProcess {
        process: syncing_process,
        topic: source_topic_name,
        table: target_table_name,
    }
}

async fn sync_kafka_to_clickhouse(
    kafka_config: RedpandaConfig,
    clickhouse_config: ClickHouseConfig,
    source_topic_name: String,
    source_topic_columns: Vec<Column>,
    target_table_name: String,
    target_table_columns: Vec<ClickHouseColumn>,
) -> anyhow::Result<()> {
    let subscriber = create_subscriber(&kafka_config, SYNC_GROUP_ID, &source_topic_name);

    let clikhouse_columns = target_table_columns
        .iter()
        .map(|column| column.name.clone())
        .collect();

    let inserter = Inserter::new(clickhouse_config, &target_table_name, clikhouse_columns);

    // WARNING: the code below is very performance sensitive
    // it is run for every message that needs to be written to clickhouse. As such we should
    // optimize as much as possible

    // In that context we would rather not interupt the syncing process if an error occurs.
    // The user doesn't currently get feedback on the error since the process is burried behind the scenes and
    // asynchronous.

    // This should also not be broken, otherwise, the subscriber will stop receiving messages

    loop {
        match subscriber.recv().await {
            Err(e) => {
                debug!("Error receiving message from {}: {}", source_topic_name, e);
            }

            Ok(message) => match message.payload() {
                Some(payload) => match std::str::from_utf8(payload) {
                    Ok(payload_str) => {
                        debug!(
                            "Received message from {}: {}",
                            source_topic_name, payload_str
                        );

                        if let Ok(json_value) = serde_json::from_str(payload_str) {
                            if let Ok(clickhouse_record) =
                                mapper_json_to_clickhouse_record(&source_topic_columns, json_value)
                            {
                                let res = inserter.insert(clickhouse_record).await;

                                if let Err(e) = res {
                                    error!(
                                        "Error adding records to the queue to be inserted: {}",
                                        e
                                    );
                                }
                            }
                        }
                    }
                    Err(_) => {
                        error!(
                            "Received message from {} with invalid UTF-8",
                            source_topic_name
                        );
                    }
                },
                None => {
                    debug!(
                        "Received message from {} with no payload",
                        source_topic_name
                    );
                }
            },
        }
    }
}

fn mapper_json_to_clickhouse_record(
    schema_columns: &[Column],
    json_value: Value,
) -> anyhow::Result<ClickHouseRecord> {
    match json_value {
        Value::Object(map) => {
            let mut record = ClickHouseRecord::new();

            for column in schema_columns.iter() {
                let key = column.name.clone();
                let value = map.get(&key);

                log::debug!(
                    "Looking to map column {:?} to values in map: {:?}",
                    column,
                    map
                );
                log::debug!("Value found for key {}: {:?}", key, value);

                match value {
                    Some(value) => {
                        match map_json_value_to_clickhouse_value(&column.data_type, value) {
                            Ok(clickhouse_value) => {
                                record.insert(key, clickhouse_value);
                            }
                            Err(e) => {
                                log::debug!("Error mapping JSON value to ClickHouse value: {}", e)
                            }
                        };
                    }
                    None => {
                        // Clickhouse doesn't like NULLABLE arrays so we are inserting an empty array instead.
                        if let ColumnType::Array(inner_type) = &column.data_type {
                            let clickhouse_inner_type =
                                std_field_type_to_clickhouse_type_mapper(*inner_type.clone())?;
                            record.insert(
                                key,
                                ClickHouseValue::new_array(Vec::new(), clickhouse_inner_type),
                            );
                        }
                        // Other values are ignored and the client will insert NULL instead
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
    TypeMismatch {
        column_type: ColumnType,
        value: Value,
    },
    #[error("The Column Type {column_type:?} is not supported")]
    UnsupportedColumnType { column_type: ColumnType },
    #[error("Mapping missing in the `std_field_type_to_clickhouse_type_mapper` method")]
    ClickHouseModule(#[from] ClickhouseError),
}

fn map_json_value_to_clickhouse_value(
    column_type: &ColumnType,
    value: &Value,
) -> Result<ClickHouseValue, MappingError> {
    match column_type {
        ColumnType::String => {
            if let Some(value_str) = value.as_str() {
                Ok(ClickHouseValue::new_string(value_str.to_string()))
            } else {
                Err(MappingError::TypeMismatch {
                    column_type: column_type.clone(),
                    value: value.clone(),
                })
            }
        }
        ColumnType::Boolean => {
            if let Some(value_bool) = value.as_bool() {
                Ok(ClickHouseValue::new_boolean(value_bool))
            } else {
                Err(MappingError::TypeMismatch {
                    column_type: column_type.clone(),
                    value: value.clone(),
                })
            }
        }
        ColumnType::Int => {
            if let Some(value_int) = value.as_i64() {
                Ok(ClickHouseValue::new_int_64(value_int))
            } else {
                Err(MappingError::TypeMismatch {
                    column_type: column_type.clone(),
                    value: value.clone(),
                })
            }
        }
        ColumnType::Float => {
            if let Some(value_float) = value.as_f64() {
                Ok(ClickHouseValue::new_float_64(value_float))
            } else {
                Err(MappingError::TypeMismatch {
                    column_type: column_type.clone(),
                    value: value.clone(),
                })
            }
        }
        ColumnType::Decimal => Err(MappingError::UnsupportedColumnType {
            column_type: column_type.clone(),
        }),
        ColumnType::DateTime => {
            if let Some(value_str) = value.as_str() {
                if let Ok(date_time) = chrono::DateTime::parse_from_rfc3339(value_str) {
                    Ok(ClickHouseValue::new_date_time(date_time))
                } else {
                    Err(MappingError::TypeMismatch {
                        column_type: column_type.clone(),
                        value: value.clone(),
                    })
                }
            } else {
                Err(MappingError::TypeMismatch {
                    column_type: column_type.clone(),
                    value: value.clone(),
                })
            }
        }
        ColumnType::Enum(x) => {
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
                Err(MappingError::TypeMismatch {
                    column_type: column_type.clone(),
                    value: value.clone(),
                })
            }
        }
        ColumnType::Array(inner_column_type) => {
            if let Some(arr) = value.as_array() {
                let array_type =
                    std_field_type_to_clickhouse_type_mapper(*inner_column_type.clone())?;

                let mut array_values = Vec::new();
                for value in arr.iter() {
                    let clickhouse_value =
                        map_json_value_to_clickhouse_value(inner_column_type, value)?;
                    array_values.push(clickhouse_value);
                }

                Ok(ClickHouseValue::new_array(array_values, array_type))
            } else {
                Err(MappingError::TypeMismatch {
                    column_type: column_type.clone(),
                    value: value.clone(),
                })
            }
        }
        ColumnType::Nested(inner_nested) => {
            // Under default settings, to insert a nested object clickhouse requires that the object be passed as an array
            // A, B, C.a, C.b, C.c  where there are multiple Cs (in this case three) would be passed as
            // (A', B', [C.a', C.a', C.a'], [C.b', C.b', C.b'], [C.c', C.c', C.c'])
            // where A', B' are the values of A and B and C.a', C.b', C.c' are the values of C.a, C.b, C.c
            //
            // if there are two levels of nesting such as A, B, C.a, C.b.a, C.b.b, C.b.c, C.c inserts would be
            // (A', B', [C.a', C.a', C.a'], [([C.b.a', C.b.a', C.b.a'], [C.b.b', C.b.b', C.b.b'], [C.b.c', C.b.c', C.b.c']), [C.c', C.c', C.c'])
            //
            // Under flatten_nested=0, nested objects are passed in as arrays of tuples
            // If A, B and C are columnd and C is a nested object with columns a, b, c you'd insert data as follows
            // (A', B', [(C.a', C.b', C.c')])
            // If C.c is a nested object with columns d, e, f you'd insert data as follows
            // (A', B', [(C.a', C.b', [(C.c.d', C.c.e', C.c.f')])])
            // For now, we'll assume that flatten_nested=0 is set in clickhouse

            if let Some(obj) = value.as_object() {
                // Needs a null if the column isn't present
                let column_values = inner_nested
                    .columns
                    .iter()
                    .map(|col| {
                        let col_name = &col.name;
                        let val = obj.get(col_name);
                        match val {
                            Some(val) => {
                                map_json_value_to_clickhouse_value(&col.data_type, val).unwrap()
                            }
                            None => ClickHouseValue::new_null(
                                std_field_type_to_clickhouse_type_mapper(col.data_type.clone())
                                    .unwrap(),
                            ),
                        }
                    })
                    .collect();

                return Ok(ClickHouseValue::new_tuple(column_values));
            } else {
                Err(MappingError::TypeMismatch {
                    column_type: column_type.clone(),
                    value: value.clone(),
                })
            }
        }
        ColumnType::Json => Err(MappingError::UnsupportedColumnType {
            column_type: column_type.clone(),
        }),
        ColumnType::Bytes => Err(MappingError::UnsupportedColumnType {
            column_type: column_type.clone(),
        }),
        ColumnType::BigInt => Err(MappingError::UnsupportedColumnType {
            column_type: column_type.clone(),
        }),
    }
}

#[cfg(test)]
mod tests {

    use crate::framework::data_model::schema::Nested;

    use super::*;

    #[test]
    fn test_map_json_value_to_clickhouse_value_for_nested() {
        let example_json = r#"
        {
            "A": "A",
            "B": "B",
            "C": {
                "a": "a",
                "b": {
                    "d": "d",
                    "e": "e",
                    "f": "f"
                },
                "c": "c"
            }
        }
        "#;

        let nested_column_type: Nested = Nested {
            name: "my_nested_column".to_string(),
            // constructed from the example json
            columns: vec![
                Column {
                    name: "A".to_string(),
                    data_type: ColumnType::String,
                    required: true,
                    unique: false,
                    primary_key: false,
                    default: None,
                },
                Column {
                    name: "B".to_string(),
                    data_type: ColumnType::String,
                    required: true,
                    unique: false,
                    primary_key: false,
                    default: None,
                },
                Column {
                    name: "C".to_string(),
                    data_type: ColumnType::Nested(Nested {
                        name: "C".to_string(),
                        columns: vec![
                            Column {
                                name: "a".to_string(),
                                data_type: ColumnType::String,
                                required: true,
                                unique: false,
                                primary_key: false,
                                default: None,
                            },
                            Column {
                                name: "b".to_string(),
                                data_type: ColumnType::Nested(Nested {
                                    name: "b".to_string(),
                                    columns: vec![
                                        Column {
                                            name: "d".to_string(),
                                            data_type: ColumnType::String,
                                            required: true,
                                            unique: false,
                                            primary_key: false,
                                            default: None,
                                        },
                                        Column {
                                            name: "e".to_string(),
                                            data_type: ColumnType::String,
                                            required: true,
                                            unique: false,
                                            primary_key: false,
                                            default: None,
                                        },
                                        Column {
                                            name: "f".to_string(),
                                            data_type: ColumnType::String,
                                            required: true,
                                            unique: false,
                                            primary_key: false,
                                            default: None,
                                        },
                                    ],
                                }),
                                required: true,
                                unique: false,
                                primary_key: false,
                                default: None,
                            },
                            Column {
                                name: "c".to_string(),
                                data_type: ColumnType::String,
                                required: true,
                                unique: false,
                                primary_key: false,
                                default: None,
                            },
                        ],
                    }),
                    required: true,
                    unique: false,
                    primary_key: false,
                    default: None,
                },
                Column {
                    name: "D".to_string(),
                    data_type: ColumnType::Int,
                    required: false,
                    unique: false,
                    primary_key: false,
                    default: None,
                },
            ],
        };

        let example_json_value: Value = serde_json::from_str(example_json).unwrap();

        let values = map_json_value_to_clickhouse_value(
            &ColumnType::Nested(nested_column_type),
            &example_json_value,
        );

        println!("{:?}", values);
    }
}

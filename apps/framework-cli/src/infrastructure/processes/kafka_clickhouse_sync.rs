use std::collections::hash_map::RandomState;
use std::collections::HashSet;
use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use log::debug;
use log::error;
use log::info;
use rdkafka::consumer::StreamConsumer;
use rdkafka::producer::{DeliveryFuture, FutureProducer};
use rdkafka::Message;
use serde_json::Value;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::framework::core::code_loader::FrameworkObjectVersions;
use crate::framework::core::infrastructure::table::Column;
use crate::framework::core::infrastructure::table::ColumnType;
use crate::infrastructure::olap::clickhouse::config::ClickHouseConfig;
use crate::infrastructure::olap::clickhouse::errors::ClickhouseError;
use crate::infrastructure::olap::clickhouse::inserter::Inserter;
use crate::infrastructure::olap::clickhouse::model::{
    ClickHouseColumn, ClickHouseRecord, ClickHouseRuntimeEnum, ClickHouseValue,
};
use crate::infrastructure::olap::clickhouse::version_sync::{VersionSync, VersionSyncType};
use crate::infrastructure::stream::redpanda::create_subscriber;
use crate::infrastructure::stream::redpanda::fetch_topics;
use crate::infrastructure::stream::redpanda::RedpandaConfig;
use crate::infrastructure::stream::redpanda::{create_producer, send_with_back_pressure};
use crate::metrics::{MetricEvent, Metrics};

const TABLE_SYNC_GROUP_ID: &str = "clickhouse_sync";
const VERSION_SYNC_GROUP_ID: &str = "version_sync_flow_sync";

struct TableSyncingProcess {
    process: JoinHandle<anyhow::Result<()>>,
    topic: String,
    table: String,
}

struct TopicToTopicSyncingProcess {
    process: JoinHandle<()>,
    #[allow(dead_code)]
    from_topic: String,
    to_topic: String,
}

pub struct SyncingProcessesRegistry {
    to_table_registry: HashMap<String, JoinHandle<anyhow::Result<()>>>,
    to_topic_registry: HashMap<String, JoinHandle<()>>,
    kafka_config: RedpandaConfig,
    clickhouse_config: ClickHouseConfig,
}

impl SyncingProcessesRegistry {
    pub fn new(kafka_config: RedpandaConfig, clickhouse_config: ClickHouseConfig) -> Self {
        Self {
            to_table_registry: HashMap::new(),
            to_topic_registry: HashMap::new(),
            kafka_config,
            clickhouse_config,
        }
    }

    fn format_key(syncing_process: &TableSyncingProcess) -> String {
        Self::format_key_str(&syncing_process.topic, &syncing_process.table)
    }

    fn format_key_str(topic: &str, table: &str) -> String {
        format!("{}-{}", topic, table)
    }

    fn insert_table_sync(&mut self, syncing_process: TableSyncingProcess) {
        let key = Self::format_key(&syncing_process);
        self.to_table_registry.insert(key, syncing_process.process);
    }
    fn insert_topic_sync(&mut self, syncing_process: TopicToTopicSyncingProcess) {
        let key = syncing_process.to_topic; // the source input topic
        self.to_topic_registry.insert(key, syncing_process.process);
    }

    pub async fn start_all(
        &mut self,
        framework_object_versions: &FrameworkObjectVersions,
        version_syncs: &[VersionSync],
        metrics: Arc<Metrics>,
    ) -> Result<(), rdkafka::error::KafkaError> {
        info!("<DCM> Starting all syncing processes");

        let kafka_config = self.kafka_config.clone();
        let clickhouse_config = self.clickhouse_config.clone();

        let available_topics: HashSet<String, RandomState> =
            HashSet::from_iter(fetch_topics(&kafka_config).await?);

        // Spawn sync for the current and old models
        let versions_iterator =
            framework_object_versions
                .all_versions()
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
                            metrics.clone(),
                        ))
                });

        let version_syncs_iterator = version_syncs.iter().filter_map(|vs| match vs.sync_type {
            VersionSyncType::Sql(_) => None,
            _ => {
                let output_topic = vs.topic_name("output");
                if available_topics.contains(&output_topic) {
                    Some(spawn_sync_process_core(
                        kafka_config.clone(),
                        clickhouse_config.clone(),
                        output_topic,
                        vs.dest_data_model.columns.clone(),
                        vs.dest_table.name.clone(),
                        vs.dest_table.columns.clone(),
                        metrics.clone(),
                    ))
                } else {
                    None
                }
            }
        });

        for syncing_process in versions_iterator.chain(version_syncs_iterator) {
            self.insert_table_sync(syncing_process);
        }

        version_syncs.iter().for_each(|vs| {
            let input_topic = vs.topic_name("input");
            let source_topic = vs.source_topic_name();

            if available_topics.contains(&input_topic) && available_topics.contains(&source_topic) {
                self.insert_topic_sync(spawn_kafka_to_kafka_process(
                    kafka_config.clone(),
                    source_topic,
                    input_topic,
                    metrics.clone(),
                ));
            }
        });

        Ok(())
    }

    pub fn start_topic_to_table(
        &mut self,
        source_topic_name: String,
        source_topic_columns: Vec<Column>,
        target_table_name: String,
        target_table_columns: Vec<ClickHouseColumn>,
        metrics: Arc<Metrics>,
    ) {
        info!(
            "<DCM> Starting syncing process for topic: {} and table: {}",
            source_topic_name, target_table_name
        );
        let key = Self::format_key_str(&source_topic_name, &target_table_name);

        // the schema of the currently running process is outdated
        if let Some(process) = self.to_table_registry.remove(&key) {
            process.abort();
        }

        let syncing_process = spawn_sync_process_core(
            self.kafka_config.clone(),
            self.clickhouse_config.clone(),
            source_topic_name,
            source_topic_columns,
            target_table_name,
            target_table_columns,
            metrics,
        );

        self.insert_table_sync(syncing_process);
    }

    pub fn stop_topic_to_table(&mut self, topic_name: &str, table_name: &str) {
        let key = Self::format_key_str(topic_name, table_name);
        if let Some(process) = self.to_table_registry.remove(&key) {
            process.abort();
        }
    }

    pub fn start_topic_to_topic(
        &mut self,
        source_topic_name: String,
        target_topic_name: String,
        metrics: Arc<Metrics>,
    ) {
        info!(
            "<DCM> Starting syncing process from topic: {} to topic: {}",
            source_topic_name, target_topic_name
        );
        let key = target_topic_name.clone();

        if let Some(process) = self.to_topic_registry.remove(&key) {
            process.abort();
        }

        self.insert_topic_sync(spawn_kafka_to_kafka_process(
            self.kafka_config.clone(),
            source_topic_name,
            target_topic_name,
            metrics.clone(),
        ));
    }

    pub fn stop_topic_to_topic(&mut self, target_topic_name: &str) {
        if let Some(process) = self.to_table_registry.remove(target_topic_name) {
            process.abort();
        }
    }
}

type FnSyncProcess =
    Box<dyn Fn((String, Vec<Column>, String, Vec<ClickHouseColumn>)) -> TableSyncingProcess>;

fn spawn_sync_process(
    kafka_config: RedpandaConfig,
    clickhouse_config: ClickHouseConfig,
    metrics: Arc<Metrics>,
) -> FnSyncProcess {
    Box::new(
        move |(
            source_topic_name,
            source_topic_columns,
            target_table_name,
            target_table_columns,
        )| {
            info!(
                "Starting Kafka sync to clickhouse from topic: {} to table: {}",
                source_topic_name, target_table_name
            );
            spawn_sync_process_core(
                kafka_config.clone(),
                clickhouse_config.clone(),
                source_topic_name,
                source_topic_columns,
                target_table_name,
                target_table_columns,
                metrics.clone(),
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
    metrics: Arc<Metrics>,
) -> TableSyncingProcess {
    let syncing_process = tokio::spawn(sync_kafka_to_clickhouse(
        kafka_config,
        clickhouse_config,
        source_topic_name.clone(),
        source_topic_columns,
        target_table_name.clone(),
        target_table_columns,
        metrics,
    ));

    TableSyncingProcess {
        process: syncing_process,
        topic: source_topic_name,
        table: target_table_name,
    }
}

fn spawn_kafka_to_kafka_process(
    kafka_config: RedpandaConfig,
    source_topic_name: String,
    target_topic_name: String,
    metrics: Arc<Metrics>,
) -> TopicToTopicSyncingProcess {
    let syncing_process = tokio::spawn(sync_kafka_to_kafka(
        kafka_config,
        source_topic_name.clone(),
        target_topic_name.clone(),
        metrics,
    ));

    TopicToTopicSyncingProcess {
        process: syncing_process,
        from_topic: source_topic_name,
        to_topic: target_topic_name,
    }
}

async fn sync_kafka_to_kafka(
    kafka_config: RedpandaConfig,
    source_topic_name: String,
    target_topic_name: String,
    metrics: Arc<Metrics>,
) {
    let subscriber = create_subscriber(&kafka_config, VERSION_SYNC_GROUP_ID, &source_topic_name);
    let producer = create_producer(kafka_config.clone());

    // there is no concurrency on this queue, the mutex is to satisfy the borrow checker
    let queue: Mutex<VecDeque<DeliveryFuture>> = Mutex::new(VecDeque::new());
    let target_topic_name = &target_topic_name;

    iterate_subscriber(
        subscriber,
        source_topic_name,
        |payload| {
            let producer: &FutureProducer = &producer.producer;
            let queue = &queue;
            Box::pin(async move {
                send_with_back_pressure(
                    &mut *queue.lock().await,
                    producer,
                    target_topic_name,
                    payload,
                )
                .await
            })
        },
        metrics,
    )
    .await
}

async fn sync_kafka_to_clickhouse(
    kafka_config: RedpandaConfig,
    clickhouse_config: ClickHouseConfig,
    source_topic_name: String,
    source_topic_columns: Vec<Column>,
    target_table_name: String,
    target_table_columns: Vec<ClickHouseColumn>,
    metrics: Arc<Metrics>,
) -> anyhow::Result<()> {
    let subscriber: StreamConsumer =
        create_subscriber(&kafka_config, TABLE_SYNC_GROUP_ID, &source_topic_name);

    let clickhouse_columns = target_table_columns
        .iter()
        .map(|column| column.name.clone())
        .collect();

    let inserter = Inserter::new(clickhouse_config, &target_table_name, clickhouse_columns);

    // WARNING: the code below is very performance sensitive
    // it is run for every message that needs to be written to clickhouse. As such we should
    // optimize as much as possible

    // In that context we would rather not interrupt the syncing process if an error occurs.
    // The user doesn't currently get feedback on the error since the process is buried behind the scenes and
    // asynchronous.

    // This should also not be broken, otherwise, the subscriber will stop receiving messages

    iterate_subscriber(
        subscriber,
        source_topic_name,
        |payload_str| {
            // allow the async block to move the borrows
            let inserter = &inserter;
            let source_topic_columns = &source_topic_columns;

            Box::pin(async move {
                if let Ok(json_value) = serde_json::from_str(payload_str.as_str()) {
                    if let Ok(clickhouse_record) =
                        mapper_json_to_clickhouse_record(source_topic_columns, json_value)
                    {
                        let res = inserter.insert(clickhouse_record).await;

                        if let Err(e) = res {
                            error!("Error adding records to the queue to be inserted: {}", e);
                        }
                    }
                }
            })
        },
        metrics,
    )
    .await;
    Ok(())
}

async fn iterate_subscriber<'a, F>(
    subscriber: StreamConsumer,
    source_topic_name: String,
    action: F,
    metrics: Arc<Metrics>,
) where
    // we shouldn't need the boxing, but i can't make the borrow checker happy
    F: Fn(String) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>,
{
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
                        metrics
                            .send_metric_event(MetricEvent::TopicToOLAPEvent {
                                timestamp: chrono::Utc::now(),
                                count: 1,
                                bytes: payload.len() as u64,
                                consumer_group: "clickhouse sync".to_string(),
                                topic_name: source_topic_name.clone(),
                            })
                            .await;

                        action(payload_str.to_string()).await;
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
                    Some(Value::Null) => {
                        if column.required {
                            log::error!("Required column {} has a null value", key);
                        } else {
                            record.insert(key, ClickHouseValue::new_null());
                        }
                    }
                    Some(value) => {
                        match map_json_value_to_clickhouse_value(&column.data_type, value) {
                            Ok(clickhouse_value) => {
                                record.insert(key, clickhouse_value);
                            }
                            Err(e) => {
                                log::debug!("For column {} with type {}, Error mapping JSON value to ClickHouse value: {}", column.name, &column.data_type, e)
                            }
                        };
                    }
                    None => {
                        // Clickhouse doesn't like NULLABLE arrays so we are inserting an empty array instead.
                        if let ColumnType::Array(_inner_type) = &column.data_type {
                            record.insert(key, ClickHouseValue::new_array(Vec::new()));
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
        ColumnType::Enum(_) => {
            // In this context what could be coming in could be a string or a number depending on
            // how the enum was defined at the source.
            if let Some(value_str) = value.as_str() {
                Ok(ClickHouseValue::new_enum(
                    ClickHouseRuntimeEnum::ClickHouseString(value_str.to_string()),
                ))
            } else if let Some(value_int) = value.as_i64() {
                Ok(ClickHouseValue::new_enum(
                    ClickHouseRuntimeEnum::ClickHouseInt(value_int as u8),
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
                let mut array_values = Vec::new();
                for value in arr.iter() {
                    let clickhouse_value =
                        map_json_value_to_clickhouse_value(inner_column_type, value)?;
                    array_values.push(clickhouse_value);
                }

                Ok(ClickHouseValue::new_array(array_values))
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
            //
            // For now, we'll assume that flatten_nested=0 is set in clickhouse

            if let Some(obj) = value.as_object() {
                // Needs a null if the column isn't present
                let mut values: Vec<ClickHouseValue> = Vec::new();

                for col in inner_nested.columns.iter() {
                    let col_name = &col.name;
                    let val = obj.get(col_name);

                    match val {
                        Some(Value::Null) => {
                            if col.required {
                                log::error!("Required column {} has a null value", col_name);
                            }
                            // We are adding the value anyway to match the number of arguments that clickhouse expects
                            values.push(ClickHouseValue::new_null());
                        }
                        Some(val) => {
                            values.push(map_json_value_to_clickhouse_value(&col.data_type, val)?);
                            map_json_value_to_clickhouse_value(&col.data_type, val)?;
                        }
                        None => {
                            values.push(ClickHouseValue::new_null());
                        }
                    };
                }

                Ok(ClickHouseValue::new_tuple(values))
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
    use crate::framework::core::infrastructure::table::Nested;

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
                    jwt: false,
                    default: None,
                },
                Column {
                    name: "B".to_string(),
                    data_type: ColumnType::String,
                    required: true,
                    unique: false,
                    primary_key: false,
                    jwt: false,
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
                                jwt: false,
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
                                            jwt: false,
                                            default: None,
                                        },
                                        Column {
                                            name: "e".to_string(),
                                            data_type: ColumnType::String,
                                            required: true,
                                            unique: false,
                                            primary_key: false,
                                            jwt: false,
                                            default: None,
                                        },
                                        Column {
                                            name: "f".to_string(),
                                            data_type: ColumnType::String,
                                            required: true,
                                            unique: false,
                                            primary_key: false,
                                            jwt: false,
                                            default: None,
                                        },
                                    ],
                                }),
                                required: true,
                                unique: false,
                                primary_key: false,
                                jwt: false,
                                default: None,
                            },
                            Column {
                                name: "c".to_string(),
                                data_type: ColumnType::String,
                                required: true,
                                unique: false,
                                primary_key: false,
                                jwt: false,
                                default: None,
                            },
                        ],
                    }),
                    required: true,
                    unique: false,
                    primary_key: false,
                    jwt: false,
                    default: None,
                },
                Column {
                    name: "D".to_string(),
                    data_type: ColumnType::Int,
                    required: false,
                    unique: false,
                    primary_key: false,
                    jwt: false,
                    default: None,
                },
            ],
        };

        let example_json_value: Value = serde_json::from_str(example_json).unwrap();

        let values = map_json_value_to_clickhouse_value(
            &ColumnType::Nested(nested_column_type),
            &example_json_value,
        );

        let values_string = "[('A','B',[('a',[('d','e','f')],'c')],NULL)]".to_string();
        // Note the corresponding insert statement would be
        // INSERT INTO TimLiveTest VALUES ('T', [('A','B',[('a',[('d','e','f')],'c')],NULL)])
        // where TimLiveTest is the table name and contains our nested object and a order by Key

        assert_eq!(values.unwrap().clickhouse_to_string(), values_string);
    }
}

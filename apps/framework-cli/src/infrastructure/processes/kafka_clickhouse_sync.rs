//! Data synchronization module for Kafka to ClickHouse integration.
//!
//! This module provides functionality to:
//! - Synchronize data from Kafka topics to ClickHouse tables
//! - Forward messages between Kafka topics
//! - Transform JSON data to ClickHouse compatible formats
//!
//! The implementation focuses on performance and reliability with support for
//! batching, back pressure, and error handling mechanisms.

use futures::TryFutureExt;
use log::error;
use log::info;
use log::{debug, warn};
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::producer::DeliveryFuture;
use rdkafka::Message;
use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, LazyLock};
use tokio::task::JoinHandle;

use crate::framework::core::infrastructure::table::Column;
use crate::framework::core::infrastructure::table::ColumnType;
use crate::infrastructure::olap::clickhouse::client::ClickHouseClient;
use crate::infrastructure::olap::clickhouse::config::ClickHouseConfig;
use crate::infrastructure::olap::clickhouse::errors::ClickhouseError;
use crate::infrastructure::olap::clickhouse::inserter::Inserter;
use crate::infrastructure::olap::clickhouse::model::{
    ClickHouseColumn, ClickHouseRecord, ClickHouseRuntimeEnum, ClickHouseValue,
};
use crate::infrastructure::stream::kafka::client::create_subscriber;
use crate::infrastructure::stream::kafka::client::{create_producer, send_with_back_pressure};
use crate::infrastructure::stream::kafka::models::KafkaConfig;
use crate::metrics::{MetricEvent, Metrics};
use crate::utilities::validate_passthrough::DECIMAL_REGEX;
use regex;
use regex::Regex;
use std::str::FromStr;
use tokio::select;
use uuid::Uuid;

const IPV4_REGEX: &str =
    r"^((25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$";

pub static IPV4_PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(IPV4_REGEX).unwrap());

/// Consumer group ID for table synchronization
const TABLE_SYNC_GROUP_ID: &str = "clickhouse_sync";
/// Consumer group ID for version synchronization
const VERSION_SYNC_GROUP_ID: &str = "version_sync_flow_sync";
/// Maximum interval in seconds between flushes to ClickHouse
const MAX_FLUSH_INTERVAL_SECONDS: u64 = 1;
/// Maximum batch size for ClickHouse inserts
const MAX_BATCH_SIZE: usize = 100000;

/// Represents a Kafka to ClickHouse synchronization process
struct TableSyncingProcess {
    /// Async task handle for the sync process
    process: JoinHandle<anyhow::Result<()>>,
}

/// Represents a Kafka to Kafka synchronization process
struct TopicToTopicSyncingProcess {
    /// Async task handle for the sync process
    process: JoinHandle<()>,
    /// Source Kafka topic name
    #[allow(dead_code)]
    from_topic: String,
    /// Target Kafka topic name
    to_topic: String,
}

/// Registry that manages all synchronization processes
///
/// TODO make this more type safe, ie the topic names should not be strings but rather a struct
/// so that a name from Topic could not be passed in
pub struct SyncingProcessesRegistry {
    /// Map of topic-table processes by their combined key
    to_table_registry: HashMap<String, JoinHandle<anyhow::Result<()>>>,
    /// Map of topic-to-topic processes by their target topic name
    to_topic_registry: HashMap<String, JoinHandle<()>>,
    /// Kafka configuration
    kafka_config: KafkaConfig,
    /// ClickHouse configuration
    clickhouse_config: ClickHouseConfig,
}

impl SyncingProcessesRegistry {
    /// Creates a new synchronization processes registry
    ///
    /// # Arguments
    /// * `kafka_config` - Configuration for Kafka/Redpanda connection
    /// * `clickhouse_config` - Configuration for ClickHouse connection
    pub fn new(kafka_config: KafkaConfig, clickhouse_config: ClickHouseConfig) -> Self {
        Self {
            to_table_registry: HashMap::new(),
            to_topic_registry: HashMap::new(),
            kafka_config,
            clickhouse_config,
        }
    }

    /// Registers a topic-to-table synchronization process
    ///
    /// # Arguments
    /// * `syncing_process` - The synchronization process to register
    fn insert_table_sync(&mut self, sync_id: String, syncing_process: TableSyncingProcess) {
        self.to_table_registry
            .insert(sync_id, syncing_process.process);
    }

    /// Registers a topic-to-topic synchronization process
    ///
    /// # Arguments
    /// * `syncing_process` - The synchronization process to register
    fn insert_topic_sync(&mut self, syncing_process: TopicToTopicSyncingProcess) {
        let key = syncing_process.to_topic; // the source input topic
        self.to_topic_registry.insert(key, syncing_process.process);
    }

    /// Starts a new synchronization process from a Kafka topic to a ClickHouse table
    ///
    /// # Arguments
    /// * `source_topic_name` - Source Kafka topic name
    /// * `source_topic_columns` - Schema definition of the source topic
    /// * `target_table_name` - Target ClickHouse table name
    /// * `target_table_columns` - Schema definition of the target table
    /// * `metrics` - Metrics collection service
    pub fn start_topic_to_table(
        &mut self,
        sync_id: String,
        source_topic_name: String,
        source_topic_columns: Vec<Column>,
        target_table_name: String,
        target_table_columns: Vec<ClickHouseColumn>,
        metrics: Arc<Metrics>,
    ) {
        info!(
            "Starting syncing process for topic: {} and table: {}",
            source_topic_name, target_table_name
        );
        // the schema of the currently running process is outdated
        if let Some(process) = self.to_table_registry.remove(&sync_id) {
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

        self.insert_table_sync(sync_id, syncing_process);
    }

    /// Stops a topic-to-table synchronization process
    ///
    /// # Arguments
    /// * `topic_name` - Source Kafka topic name
    /// * `table_name` - Target ClickHouse table name
    pub fn stop_topic_to_table(&mut self, sync_id: &str) {
        if let Some(process) = self.to_table_registry.remove(sync_id) {
            process.abort();
        } else {
            warn!("Sync ID {} not found in to_table_registry", sync_id);
        }
    }

    /// Starts a new synchronization process from one Kafka topic to another
    ///
    /// # Arguments
    /// * `source_topic_name` - Source Kafka topic name
    /// * `target_topic_name` - Target Kafka topic name
    /// * `metrics` - Metrics collection service
    pub fn start_topic_to_topic(
        &mut self,
        source_topic_name: String,
        target_topic_name: String,
        metrics: Arc<Metrics>,
    ) {
        info!(
            "Starting syncing process from topic: {} to topic: {}",
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

    /// Stops a topic-to-topic synchronization process
    ///
    /// # Arguments
    /// * `target_topic_name` - Target Kafka topic name
    pub fn stop_topic_to_topic(&mut self, target_topic_name: &str) {
        if let Some(process) = self.to_topic_registry.remove(target_topic_name) {
            process.abort();
        }
    }
}

/// Spawns a new Kafka to ClickHouse sync process
///
/// # Arguments
/// * `kafka_config` - Kafka/Redpanda configuration
/// * `clickhouse_config` - ClickHouse configuration
/// * `source_topic_name` - Source Kafka topic name
/// * `source_topic_columns` - Schema definition of the source topic
/// * `target_table_name` - Target ClickHouse table name
/// * `target_table_columns` - Schema definition of the target table
/// * `metrics` - Metrics collection service
///
/// # Returns
/// A TableSyncingProcess struct encapsulating the async task
fn spawn_sync_process_core(
    kafka_config: KafkaConfig,
    clickhouse_config: ClickHouseConfig,
    source_topic_name: String,
    source_topic_columns: Vec<Column>,
    target_table_name: String,
    target_table_columns: Vec<ClickHouseColumn>,
    metrics: Arc<Metrics>,
) -> TableSyncingProcess {
    let target_table_name_clone = target_table_name.clone();

    let syncing_process = tokio::spawn(
        sync_kafka_to_clickhouse(
            kafka_config,
            clickhouse_config,
            source_topic_name.clone(),
            source_topic_columns,
            target_table_name.clone(),
            target_table_columns,
            metrics,
        )
        .inspect_err(move |e| {
            error!(
                "Sync process to table {} failed with error {:?}",
                target_table_name_clone, e
            )
        }),
    );

    TableSyncingProcess {
        process: syncing_process,
    }
}

/// Spawns a new topic to topic sync process
///
/// # Arguments
/// * `kafka_config` - Kafka/Redpanda configuration
/// * `source_topic_name` - Source Kafka topic name
/// * `target_topic_name` - Target Kafka topic name
/// * `metrics` - Metrics collection service
///
/// # Returns
/// A TopicToTopicSyncingProcess struct encapsulating the async task
fn spawn_kafka_to_kafka_process(
    kafka_config: KafkaConfig,
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

/// Continuously forwards messages from one Kafka topic to another
///
/// # Arguments
/// * `kafka_config` - Kafka/Redpanda configuration
/// * `source_topic_name` - Source Kafka topic name
/// * `target_topic_name` - Target Kafka topic name
/// * `metrics` - Metrics collection service
async fn sync_kafka_to_kafka(
    kafka_config: KafkaConfig,
    source_topic_name: String,
    target_topic_name: String,
    metrics: Arc<Metrics>,
) {
    let subscriber: Arc<StreamConsumer> = Arc::new(create_subscriber(
        &kafka_config,
        VERSION_SYNC_GROUP_ID,
        &source_topic_name,
    ));
    let producer = create_producer(kafka_config.clone());

    let mut queue: VecDeque<DeliveryFuture> = VecDeque::new();
    let target_topic_name = &target_topic_name;

    loop {
        match subscriber.recv().await {
            Err(e) => {
                // Escalated from `debug!` to `warn!` so operators notice message-receive errors that
                // cause potential data loss or stalled sync loops.
                warn!("Error receiving message from {}: {}", source_topic_name, e);
            }

            Ok(message) => match message.payload() {
                Some(payload) => match std::str::from_utf8(payload) {
                    Ok(payload_str) => {
                        log::trace!(
                            "Received message from {}: {}",
                            source_topic_name,
                            payload_str
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

                        send_with_back_pressure(
                            &mut queue,
                            &producer.producer,
                            target_topic_name,
                            payload_str.to_string(),
                        )
                        .await;
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

/// Continuously synchronizes data from a Kafka topic to a ClickHouse table
///
/// # Arguments
/// * `kafka_config` - Kafka/Redpanda configuration
/// * `clickhouse_config` - ClickHouse configuration
/// * `source_topic_name` - Source Kafka topic name
/// * `source_topic_columns` - Schema definition of the source topic
/// * `target_table_name` - Target ClickHouse table name
/// * `target_table_columns` - Schema definition of the target table
/// * `metrics` - Metrics collection service
///
/// # Returns
/// Result indicating success or failure
async fn sync_kafka_to_clickhouse(
    kafka_config: KafkaConfig,
    clickhouse_config: ClickHouseConfig,
    source_topic_name: String,
    source_topic_columns: Vec<Column>,
    target_table_name: String,
    target_table_columns: Vec<ClickHouseColumn>,
    metrics: Arc<Metrics>,
) -> anyhow::Result<()> {
    info!(
        "Starting/resuming kafka-clickhouse sync for {}.",
        source_topic_name
    );

    let subscriber: Arc<StreamConsumer> = Arc::new(create_subscriber(
        &kafka_config,
        TABLE_SYNC_GROUP_ID,
        &source_topic_name,
    ));

    let clickhouse_columns = target_table_columns
        .iter()
        .map(|column| column.name.clone())
        .collect();

    let topic_clone = source_topic_name.clone();
    let subscriber_clone = subscriber.clone();

    let client = ClickHouseClient::new(&clickhouse_config).unwrap();
    let mut inserter = Inserter::<ClickHouseClient>::new(
        client,
        MAX_BATCH_SIZE,
        Box::new(move |partition, offset| {
            subscriber_clone.store_offset(&topic_clone, partition, offset)
        }),
        target_table_name,
        clickhouse_columns,
    );

    let flush_interval = std::time::Duration::from_secs(MAX_FLUSH_INTERVAL_SECONDS);
    let mut interval_clock = tokio::time::interval(flush_interval);

    // WARNING: the code below is very performance sensitive
    // it is run for every message that needs to be written to clickhouse. As such we should
    // optimize as much as possible

    // In that context we would rather not interrupt the syncing process if an error occurs.

    // This should also not be broken, otherwise, the subscriber will stop receiving messages

    loop {
        // if we have a full batch, we flush it
        if !inserter.is_empty() && inserter.len() > 1 {
            inserter.flush().await;
        }

        select! {
            // This is here to ensure that if we don't have new messages to process, we still flush
            // the inserter at the end of the interval.
            _ = interval_clock.tick() => {
                inserter.flush().await;
                continue;
            }
            // Since this is triggered for every message, if a batch gets too big, we will
            // trigger a flush at the next message that comes in.
            message = subscriber.recv() => {
                match message {
                    Err(e) => {
                        // Same escalation here for the table-sync path: failed consume means data
                        // never reaches ClickHouse, so it deserves a `warn!`.
                        warn!("Error receiving message from {}: {}", source_topic_name, e);
                    }

                    Ok(message) => match message.payload() {
                        Some(payload) => match std::str::from_utf8(payload) {
                            Ok(payload_str) => {
                                log::trace!(
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

                                if let Ok(json_value) = serde_json::from_str(payload_str) {
                                    if let Ok(clickhouse_record) =
                                        mapper_json_to_clickhouse_record(&source_topic_columns, json_value)
                                    {
                                        inserter.insert(
                                            clickhouse_record,
                                            message.partition(),
                                            message.offset(),
                                        );
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
    }
}

/// Maps JSON values to ClickHouse records based on column schema
///
/// # Arguments
/// * `schema_columns` - Column schema defining the data structure
/// * `json_value` - JSON value to convert to ClickHouse record
///
/// # Returns
/// * `Ok(ClickHouseRecord)` - Successfully mapped record
/// * `Err` - Error if mapping fails
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

                log::trace!(
                    "Looking to map column {:?} to values in map: {:?}",
                    column,
                    map
                );
                log::trace!("Value found for key {}: {:?}", key, value);

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
                                // Promote mapping failures to `warn!` so we don't silently skip
                                // individual records when their schema/value deviates.
                                log::warn!("For column {} with type {}, Error mapping JSON value to ClickHouse value: {}", column.name, &column.data_type, e)
                            }
                        };
                    }
                    None => {
                        // Clickhouse doesn't like NULLABLE arrays so we are inserting an empty array instead.
                        if let ColumnType::Array { .. } = &column.data_type {
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

/// Error types that can occur during value mapping
#[derive(Debug, thiserror::Error)]
enum MappingError {
    /// Error when JSON value doesn't match the expected ClickHouse column type
    #[error("Failed to map the JSON value {value:?} to ClickHouse column typed {column_type:?}")]
    TypeMismatch {
        column_type: Box<ColumnType>,
        value: Value,
    },
    /// Error when trying to map to an unsupported ClickHouse column type
    #[error("The Column Type {column_type:?} is not supported")]
    UnsupportedColumnType { column_type: Box<ColumnType> },
    /// Error propagated from the ClickHouse client
    #[error("Mapping missing in the `std_field_type_to_clickhouse_type_mapper` method")]
    ClickHouseModule(#[from] ClickhouseError),
}

/// Maps a JSON value to a ClickHouse value based on the column type
///
/// # Arguments
/// * `column_type` - Target ClickHouse column type
/// * `value` - JSON value to convert
///
/// # Returns
/// * `Ok(ClickHouseValue)` - Successfully mapped value
/// * `Err(MappingError)` - Error describing why mapping failed
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
                    column_type: Box::new(column_type.clone()),
                    value: value.clone(),
                })
            }
        }
        ColumnType::Boolean => {
            if let Some(value_bool) = value.as_bool() {
                Ok(ClickHouseValue::new_boolean(value_bool))
            } else {
                Err(MappingError::TypeMismatch {
                    column_type: Box::new(column_type.clone()),
                    value: value.clone(),
                })
            }
        }
        ColumnType::Int(_) => {
            if let Some(value_int) = value.as_i64() {
                Ok(ClickHouseValue::new_int_64(value_int))
            } else {
                Err(MappingError::TypeMismatch {
                    column_type: Box::new(column_type.clone()),
                    value: value.clone(),
                })
            }
        }
        ColumnType::Float(_) => {
            if let Some(value_float) = value.as_f64() {
                Ok(ClickHouseValue::new_float_64(value_float))
            } else {
                Err(MappingError::TypeMismatch {
                    column_type: Box::new(column_type.clone()),
                    value: value.clone(),
                })
            }
        }
        ColumnType::Decimal { .. } => {
            if let Some(number) = value.as_i64() {
                Ok(ClickHouseValue::new_int_64(number))
            } else if let Some(number) = value.as_f64() {
                Ok(ClickHouseValue::new_float_64(number))
            } else if let Some(s) = value.as_str() {
                if DECIMAL_REGEX.is_match(s) {
                    Ok(ClickHouseValue::new_string(s.to_string()))
                } else {
                    Err(MappingError::TypeMismatch {
                        column_type: Box::new(column_type.clone()),
                        value: value.clone(),
                    })
                }
            } else {
                Err(MappingError::TypeMismatch {
                    column_type: Box::new(column_type.clone()),
                    value: value.clone(),
                })
            }
        }
        ColumnType::DateTime { .. } => {
            if let Some(value_str) = value.as_str() {
                if let Ok(date_time) = chrono::DateTime::parse_from_rfc3339(value_str) {
                    Ok(ClickHouseValue::new_date_time(date_time))
                } else {
                    Err(MappingError::TypeMismatch {
                        column_type: Box::new(column_type.clone()),
                        value: value.clone(),
                    })
                }
            } else {
                Err(MappingError::TypeMismatch {
                    column_type: Box::new(column_type.clone()),
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
                    column_type: Box::new(column_type.clone()),
                    value: value.clone(),
                })
            }
        }
        ColumnType::Array {
            element_type,
            element_nullable,
        } => {
            if let Some(arr) = value.as_array() {
                let mut array_values = Vec::new();
                for value in arr.iter() {
                    if *value == Value::Null {
                        if !element_nullable {
                            log::error!("Array of non nullable elements has a null value");
                        }
                        // We are adding the value anyway to match the number of arguments that clickhouse expects
                        array_values.push(ClickHouseValue::new_null());
                    } else {
                        let clickhouse_value =
                            map_json_value_to_clickhouse_value(element_type, value)?;
                        array_values.push(clickhouse_value);
                    }
                }

                Ok(ClickHouseValue::new_array(array_values))
            } else {
                Err(MappingError::TypeMismatch {
                    column_type: Box::new(column_type.clone()),
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

                Ok(ClickHouseValue::new_array(vec![
                    ClickHouseValue::new_tuple(values),
                ]))
            } else {
                Err(MappingError::TypeMismatch {
                    column_type: Box::new(column_type.clone()),
                    value: value.clone(),
                })
            }
        }
        ColumnType::Json => {
            if let Some(obj) = value.as_object() {
                Ok(ClickHouseValue::new_json(obj.clone()))
            } else {
                Err(MappingError::TypeMismatch {
                    column_type: Box::new(ColumnType::Json),
                    value: value.clone(),
                })
            }
        }
        ColumnType::Bytes => Err(MappingError::UnsupportedColumnType {
            column_type: Box::new(column_type.clone()),
        }),
        ColumnType::BigInt => Err(MappingError::UnsupportedColumnType {
            column_type: Box::new(column_type.clone()),
        }),
        ColumnType::Uuid => match value.as_str().filter(|s| Uuid::try_parse(s).is_ok()) {
            None => Err(MappingError::TypeMismatch {
                column_type: Box::new(ColumnType::Uuid),
                value: value.clone(),
            }),
            Some(uuid_str) => Ok(ClickHouseValue::new_string(uuid_str.to_string())),
        },
        ColumnType::Date | ColumnType::Date16 => {
            if let Some(value_str) = value
                .as_str()
                .filter(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").is_ok())
            {
                Ok(ClickHouseValue::new_string(value_str.to_string()))
            } else {
                Err(MappingError::TypeMismatch {
                    column_type: Box::new(column_type.clone()),
                    value: value.clone(),
                })
            }
        }
        ColumnType::IpV4 => {
            if let Some(value_str) = value.as_str().filter(|s| IPV4_PATTERN.is_match(s)) {
                Ok(ClickHouseValue::new_string(value_str.to_string()))
            } else {
                Err(MappingError::TypeMismatch {
                    column_type: Box::new(ColumnType::IpV4),
                    value: value.clone(),
                })
            }
        }
        ColumnType::IpV6 => {
            if let Some(value_str) = value
                .as_str()
                .filter(|s| std::net::Ipv6Addr::from_str(s).is_ok())
            {
                Ok(ClickHouseValue::new_string(value_str.to_string()))
            } else {
                Err(MappingError::TypeMismatch {
                    column_type: Box::new(ColumnType::IpV6),
                    value: value.clone(),
                })
            }
        }
        ColumnType::Nullable(inner) => {
            if value.is_null() {
                Ok(ClickHouseValue::new_null())
            } else {
                map_json_value_to_clickhouse_value(inner, value)
            }
        }
        ColumnType::NamedTuple(fields) => {
            if let Some(obj) = value.as_object() {
                let mut values = Vec::new();
                for (name, field_type) in fields {
                    let val = obj.get(name);
                    match val {
                        Some(Value::Null) => {
                            values.push(ClickHouseValue::new_null());
                        }
                        Some(val) => {
                            values.push(map_json_value_to_clickhouse_value(field_type, val)?);
                        }
                        None => {
                            values.push(ClickHouseValue::new_null());
                        }
                    }
                }
                Ok(ClickHouseValue::new_tuple(values))
            } else {
                Err(MappingError::TypeMismatch {
                    column_type: Box::new(column_type.clone()),
                    value: value.clone(),
                })
            }
        }
        ColumnType::Map {
            key_type,
            value_type,
        } => {
            if let Some(obj) = value.as_object() {
                let mut map_pairs = Vec::new();
                for (key_str, value_json) in obj {
                    // Convert the key string to the appropriate ClickHouse value
                    // JSON object keys are always strings, but ClickHouse Map keys can be numeric
                    let key_clickhouse_value =
                        match key_type.as_ref() {
                            ColumnType::Int(_) => {
                                let key_int = key_str.parse::<i64>().map_err(|_| {
                                    MappingError::TypeMismatch {
                                        column_type: Box::new(key_type.as_ref().clone()),
                                        value: serde_json::Value::String(key_str.clone()),
                                    }
                                })?;
                                ClickHouseValue::new_int_64(key_int)
                            }
                            ColumnType::Float(_) => {
                                let key_float = key_str.parse::<f64>().map_err(|_| {
                                    MappingError::TypeMismatch {
                                        column_type: Box::new(key_type.as_ref().clone()),
                                        value: serde_json::Value::String(key_str.clone()),
                                    }
                                })?;
                                ClickHouseValue::new_float_64(key_float)
                            }
                            // For string types, use the key as-is
                            ColumnType::String => ClickHouseValue::new_string(key_str.clone()),
                            // For other types, convert via JSON value
                            _ => {
                                let key_json_value = serde_json::Value::String(key_str.clone());
                                map_json_value_to_clickhouse_value(key_type, &key_json_value)?
                            }
                        };

                    // Convert the value to the appropriate ClickHouse value
                    let value_clickhouse_value =
                        map_json_value_to_clickhouse_value(value_type, value_json)?;

                    map_pairs.push((key_clickhouse_value, value_clickhouse_value));
                }
                Ok(ClickHouseValue::new_map(map_pairs))
            } else {
                Err(MappingError::TypeMismatch {
                    column_type: Box::new(column_type.clone()),
                    value: value.clone(),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::framework::core::infrastructure::table::{IntType, Nested};

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
            jwt: false,
            // constructed from the example json
            columns: vec![
                Column {
                    name: "A".to_string(),
                    data_type: ColumnType::String,
                    required: true,
                    unique: false,
                    primary_key: false,
                    default: None,
                    annotations: vec![],
                },
                Column {
                    name: "B".to_string(),
                    data_type: ColumnType::String,
                    required: true,
                    unique: false,
                    primary_key: false,
                    default: None,
                    annotations: vec![],
                },
                Column {
                    name: "C".to_string(),
                    data_type: ColumnType::Nested(Nested {
                        name: "C".to_string(),
                        jwt: false,
                        columns: vec![
                            Column {
                                name: "a".to_string(),
                                data_type: ColumnType::String,
                                required: true,
                                unique: false,
                                primary_key: false,
                                default: None,
                                annotations: vec![],
                            },
                            Column {
                                name: "b".to_string(),
                                data_type: ColumnType::Nested(Nested {
                                    name: "b".to_string(),
                                    jwt: false,
                                    columns: vec![
                                        Column {
                                            name: "d".to_string(),
                                            data_type: ColumnType::String,
                                            required: true,
                                            unique: false,
                                            primary_key: false,
                                            default: None,
                                            annotations: vec![],
                                        },
                                        Column {
                                            name: "e".to_string(),
                                            data_type: ColumnType::String,
                                            required: true,
                                            unique: false,
                                            primary_key: false,
                                            default: None,
                                            annotations: vec![],
                                        },
                                        Column {
                                            name: "f".to_string(),
                                            data_type: ColumnType::String,
                                            required: true,
                                            unique: false,
                                            primary_key: false,
                                            default: None,
                                            annotations: vec![],
                                        },
                                    ],
                                }),
                                required: true,
                                unique: false,
                                primary_key: false,
                                default: None,
                                annotations: vec![],
                            },
                            Column {
                                name: "c".to_string(),
                                data_type: ColumnType::String,
                                required: true,
                                unique: false,
                                primary_key: false,
                                default: None,
                                annotations: vec![],
                            },
                        ],
                    }),
                    required: true,
                    unique: false,
                    primary_key: false,
                    default: None,
                    annotations: vec![],
                },
                Column {
                    name: "D".to_string(),
                    data_type: ColumnType::Int(IntType::Int64),
                    required: false,
                    unique: false,
                    primary_key: false,
                    default: None,
                    annotations: vec![],
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

    #[test]
    fn test_map_json_value_to_clickhouse_value_for_map_with_string_keys() {
        // Test Map with string keys
        let map_column_type = ColumnType::Map {
            key_type: Box::new(ColumnType::String),
            value_type: Box::new(ColumnType::Int(IntType::Int32)),
        };

        let json_value = serde_json::json!({
            "key1": 10,
            "key2": 20
        });

        let result = map_json_value_to_clickhouse_value(&map_column_type, &json_value);
        assert!(result.is_ok());

        match result.unwrap() {
            ClickHouseValue::Map(pairs) => {
                assert_eq!(pairs.len(), 2);
                // Check that we have the expected keys (order might vary)
                let mut found_keys = std::collections::HashSet::new();
                for (key, _value) in pairs {
                    match key {
                        ClickHouseValue::String(s) => {
                            found_keys.insert(s);
                        }
                        _ => panic!("Expected string key"),
                    }
                }
                assert!(found_keys.contains("key1"));
                assert!(found_keys.contains("key2"));
            }
            _ => panic!("Expected map value"),
        }
    }

    #[test]
    fn test_map_json_value_to_clickhouse_value_for_map_with_numeric_keys() {
        // Test Map with numeric keys (Int32)
        let map_column_type = ColumnType::Map {
            key_type: Box::new(ColumnType::Int(IntType::Int32)),
            value_type: Box::new(ColumnType::String),
        };

        let json_value = serde_json::json!({
            "123": "value1",
            "456": "value2"
        });

        let result = map_json_value_to_clickhouse_value(&map_column_type, &json_value);
        assert!(result.is_ok());

        match result.unwrap() {
            ClickHouseValue::Map(pairs) => {
                assert_eq!(pairs.len(), 2);
                // Check that numeric keys were parsed correctly
                for (key, value) in pairs {
                    match key {
                        ClickHouseValue::ClickhouseInt(_) => {
                            // This is expected for numeric keys
                        }
                        _ => panic!("Expected numeric key, got {key:?}"),
                    }
                    match value {
                        ClickHouseValue::String(_) => {
                            // This is expected for string values
                        }
                        _ => panic!("Expected string value"),
                    }
                }
            }
            _ => panic!("Expected map value"),
        }
    }

    #[test]
    fn test_map_json_value_to_clickhouse_value_for_map_with_invalid_numeric_keys() {
        // Test Map with invalid numeric keys
        let map_column_type = ColumnType::Map {
            key_type: Box::new(ColumnType::Int(IntType::Int32)),
            value_type: Box::new(ColumnType::String),
        };

        let json_value = serde_json::json!({
            "not_a_number": "value1",
            "456": "value2"
        });

        let result = map_json_value_to_clickhouse_value(&map_column_type, &json_value);
        assert!(result.is_err());

        match result.unwrap_err() {
            MappingError::TypeMismatch { .. } => {
                // This is expected for invalid numeric keys
            }
            _ => panic!("Expected TypeMismatch error"),
        }
    }
}

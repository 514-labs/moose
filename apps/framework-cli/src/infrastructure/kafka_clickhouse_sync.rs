use std::collections::HashMap;

use crate::framework::controller::FrameworkObject;
use crate::framework::controller::FrameworkObjectVersions;
use crate::infrastructure::olap::clickhouse::client::ClickHouseClient;
use crate::infrastructure::olap::clickhouse::config::ClickHouseConfig;
use crate::infrastructure::olap::clickhouse::model::ClickHouseValue;
use crate::infrastructure::stream::redpanda::create_subscriber;
use crate::infrastructure::stream::redpanda::RedpandaConfig;
use log::error;
use rdkafka::consumer::CommitMode;
use rdkafka::consumer::Consumer;
use rdkafka::Message;

use log::debug;
use serde_json::Value;
use tokio::task::JoinHandle;

use super::olap::clickhouse::model::ClickHouseRecord;

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
        let key = Self::format_key_framework_obj(framework_object);

        if self.registry.contains_key(&key) {
            return;
        }

        let syncing_process = spawn_sync_process_core(
            self.kafka_config.clone(),
            self.clickhouse_config.clone(),
            framework_object.topic.to_string(),
            framework_object.table.name.to_string(),
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
        let topic = schema.topic;
        let table = schema.table.name;
        spawn_sync_process_core(
            kafka_config.clone(),
            clickhouse_config.clone(),
            topic,
            table,
        )
    })
}

#[tracing::instrument]
fn spawn_sync_process_core(
    kafka_config: RedpandaConfig,
    clickhouse_config: ClickHouseConfig,
    topic: String,
    table: String,
) -> SyncingProcess {
    let syncing_process = tokio::spawn(sync_kafka_to_clickhouse(
        kafka_config,
        clickhouse_config,
        topic.clone(),
        table.clone(),
    ));

    SyncingProcess {
        process: syncing_process,
        topic,
        table,
    }
}

async fn sync_kafka_to_clickhouse(
    kafka_config: RedpandaConfig,
    clickhouse_config: ClickHouseConfig,
    streaming_topic: String,
    clickhouse_table: String,
) -> anyhow::Result<()> {
    let subscriber = create_subscriber(&kafka_config, SYNC_GROUP_ID, &streaming_topic);
    let clickhouse_client = ClickHouseClient::new(&clickhouse_config).await?;

    loop {
        match subscriber.recv().await {
            Err(e) => {
                debug!("Error receiving message: {}", e);
            }

            Ok(message) => match message.payload() {
                Some(payload) => match std::str::from_utf8(payload) {
                    Ok(payload_str) => {
                        debug!("Received message: {}", payload_str);

                        let parsed_json: Value = serde_json::from_str(payload_str)?;
                        let clickhouse_record = mapper_json_to_clickhouse_record(parsed_json)?;

                        clickhouse_client
                            .insert(&clickhouse_table, clickhouse_record)
                            .await?;

                        subscriber.commit_message(&message, CommitMode::Sync)?;
                    }
                    Err(_) => {
                        error!("Received message with invalid UTF-8");
                    }
                },
                None => {
                    debug!("Received message with no payload");
                }
            },
        }
    }
}

fn mapper_json_to_clickhouse_record(json_value: Value) -> anyhow::Result<ClickHouseRecord> {
    match json_value {
        Value::Object(map) => {
            let mut record = ClickHouseRecord::new();

            for (key, value) in map {
                match value {
                    Value::String(value_str) => {
                        if let Ok(date_time) = chrono::DateTime::parse_from_rfc3339(&value_str) {
                            record.insert(key, ClickHouseValue::new_date_time(date_time));
                        } else {
                            record.insert(key, ClickHouseValue::new_string(value_str));
                        }
                    }
                    Value::Number(value_num) => {
                        if let Some(int_val) = value_num.as_i64() {
                            record.insert(key, ClickHouseValue::new_int_64(int_val));
                        } else if let Some(float_val) = value_num.as_f64() {
                            record.insert(key, ClickHouseValue::new_float_64(float_val));
                        } else {
                            log::error!("Unsupported JSON number type: {}, skipping", value_num);
                        }
                    }
                    Value::Bool(value_bool) => {
                        record.insert(key, ClickHouseValue::new_boolean(value_bool));
                    }

                    Value::Null => {
                        record.insert(key, ClickHouseValue::new_string("NULL".to_string()));
                    }

                    _ => log::error!("Unsupported JSON type: {}, skipping", value),
                }
            }

            Ok(record)
        }
        _ => Err(anyhow::anyhow!("Invalid JSON")),
    }
}

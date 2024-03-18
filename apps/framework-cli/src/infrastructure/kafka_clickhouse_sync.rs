use crate::infrastructure::olap::clickhouse::client::ClickHouseClient;
use crate::infrastructure::olap::clickhouse::config::ClickHouseConfig;
use crate::infrastructure::olap::clickhouse::model::ClickHouseValue;
use crate::infrastructure::stream::redpanda::create_subscriber;
use crate::infrastructure::stream::redpanda::RedpandaConfig;
use rdkafka::consumer::CommitMode;
use rdkafka::consumer::Consumer;
use rdkafka::Message;

use log::debug;
use serde_json::Value;

use super::olap::clickhouse::model::ClickHouseRecord;

const SYNC_GROUP_ID: &str = "clickhouse_sync";

// TODO - add the sync start to dev mode and prod mode

pub async fn sync_kafka_to_clickhouse(
    kafka_config: &RedpandaConfig,
    clickhouse_config: &ClickHouseConfig,
    streaming_topic: &str,
    clickhouse_table: &str,
) -> anyhow::Result<()> {
    let subscriber = create_subscriber(kafka_config, SYNC_GROUP_ID, streaming_topic);
    let clickhouse_client = ClickHouseClient::new(clickhouse_config).await?;

    loop {
        match subscriber.recv().await {
            Err(e) => {
                debug!("Error receiving message: {}", e);
            }

            Ok(message) => match message.payload() {
                Some(payload) => {
                    let payload_str = std::str::from_utf8(payload).unwrap();

                    debug!("Received message: {}", payload_str);

                    let parsed_json: Value = serde_json::from_str(payload_str)?;
                    let clickhouse_record = mapper_json_to_clickhouse_record(parsed_json)?;

                    clickhouse_client
                        .insert(clickhouse_table, clickhouse_record)
                        .await?;

                    subscriber.commit_message(&message, CommitMode::Sync)?;
                }
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

// #[tokio::test]
// async fn test_sync() {
//     let kafka_config = RedpandaConfig {
//         broker: "localhost:19092".to_string(),
//         sasl_username: None,
//         sasl_password: None,
//         sasl_mechanism: None,
//         security_protocol: None,
//         message_timeout_ms: 5000,
//     };

//     let clickhouse_config = ClickHouseConfig {
//         user: "panda".to_string(),
//         password: "pandapass".to_string(),
//         host: "localhost".to_string(),
//         use_ssl: false,
//         postgres_port: 5432,
//         kafka_port: 9092,
//         host_port: 18123,
//         db_name: "local".to_string(),
//     };

//     let streaming_topic = "UserActivity_0_0";
//     let clickhouse_table = "UserActivity_0_0";

//     sync_kafka_to_clickhouse(
//         &kafka_config,
//         &clickhouse_config,
//         streaming_topic,
//         clickhouse_table,
//     )
//     .await
//     .unwrap();
// }

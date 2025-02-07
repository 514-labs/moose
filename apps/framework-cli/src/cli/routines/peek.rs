use crate::cli::display::Message;
use crate::infrastructure::olap::clickhouse::mapper::std_table_to_clickhouse_table;
use crate::infrastructure::olap::clickhouse_alt_client::{
    get_pool, retrieve_infrastructure_map, select_some_as_json,
};
use crate::project::Project;

use super::{RoutineFailure, RoutineSuccess};

use crate::infrastructure::olap::clickhouse::model::ClickHouseTable;
use crate::infrastructure::stream::redpanda::create_consumer;
use futures::stream::BoxStream;
use futures::StreamExt;
use log::info;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::{Message as KafkaMessage, Offset, TopicPartitionList};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub async fn peek(
    project: Arc<Project>,
    data_model_name: &str,
    limit: u8,
    file: Option<PathBuf>,
    topic: bool,
) -> Result<RoutineSuccess, RoutineFailure> {
    let pool = get_pool(&project.clickhouse_config);
    let mut client = pool.get_handle().await.map_err(|_| {
        RoutineFailure::error(Message::new(
            "Failed".to_string(),
            "Error connecting to storage".to_string(),
        ))
    })?;

    let infra = retrieve_infrastructure_map(&mut client, &project.clickhouse_config)
        .await
        .map_err(|_| {
            RoutineFailure::error(Message::new(
                "Failed".to_string(),
                "Error retrieving current state".to_string(),
            ))
        })?
        .ok_or_else(|| {
            RoutineFailure::error(Message::new(
                "Failed".to_string(),
                "No state found".to_string(),
            ))
        })?;

    let version_suffix = project.cur_version().as_suffix();

    let consumer_ref: StreamConsumer;
    let table_ref: ClickHouseTable;

    let mut stream: BoxStream<anyhow::Result<Value>> = if topic {
        let group_id = project.redpanda_config.prefix_with_namespace("peek");

        consumer_ref = create_consumer(&project.redpanda_config, &[("group.id", &group_id)]);
        let consumer = &consumer_ref;
        let topic_versioned = format!("{}_{}", data_model_name, version_suffix);

        let topic = infra
            .topics
            .iter()
            .find_map(|(key, topic)| {
                if key.to_lowercase() == topic_versioned.to_lowercase() {
                    Some(topic)
                } else {
                    None
                }
            })
            .ok_or_else(|| {
                RoutineFailure::error(Message::new(
                    "Failed".to_string(),
                    "No matching topic found".to_string(),
                ))
            })?;
        let topic_partition_map = (0..topic.partition_count)
            .map(|partition| {
                (
                    (topic.id().clone(), partition as i32),
                    Offset::OffsetTail(limit as i64),
                )
            })
            .collect();

        info!("Peek topic_partition_map {:?}", topic_partition_map);
        TopicPartitionList::from_topic_map(&topic_partition_map)
            .and_then(|tpl| consumer.assign(&tpl))
            .map_err(|e| {
                RoutineFailure::new(
                    Message::new("Failed".to_string(), "Topic metadata fetch".to_string()),
                    e,
                )
            })?;

        Box::pin(
            consumer
                .stream()
                .map(|message| {
                    Ok(serde_json::from_slice::<Value>(
                        message?.payload().unwrap_or(&[]),
                    )?)
                })
                .take(limit.into()),
        )
    } else {
        // ¯\_(ツ)_/¯
        let dm_with_version = format!("{}_{}_{}", data_model_name, version_suffix, version_suffix);

        let table = infra
            .tables
            .iter()
            .find_map(|(key, table)| {
                if key.to_lowercase() == dm_with_version.to_lowercase() {
                    Some(table)
                } else {
                    None
                }
            })
            .ok_or_else(|| {
                RoutineFailure::error(Message::new(
                    "Failed".to_string(),
                    "No matching table found".to_string(),
                ))
            })?;

        table_ref = std_table_to_clickhouse_table(table).map_err(|_| {
            RoutineFailure::error(Message::new(
                "Failed".to_string(),
                "Error fetching table".to_string(),
            ))
        })?;

        Box::pin(
            select_some_as_json(
                &project.clickhouse_config.db_name,
                &table_ref,
                &mut client,
                limit as i64,
            )
            .await
            .map_err(|_| {
                RoutineFailure::error(Message::new(
                    "Failed".to_string(),
                    "Error selecting data".to_string(),
                ))
            })?
            .map(|result| anyhow::Ok(result?)),
        )
    };

    let mut success_count = 0;

    if let Some(file_path) = file {
        let mut file = File::create(&file_path).await.map_err(|_| {
            RoutineFailure::error(Message::new(
                "Failed".to_string(),
                "Error creating file".to_string(),
            ))
        })?;

        while let Some(result) = stream.next().await {
            match result {
                Ok(value) => {
                    let json = serde_json::to_string(&value).unwrap();
                    file.write_all(format!("{}\n", json).as_bytes())
                        .await
                        .map_err(|_| {
                            RoutineFailure::error(Message::new(
                                "Failed".to_string(),
                                "Error writing to file".to_string(),
                            ))
                        })?;
                    success_count += 1;
                }
                Err(e) => {
                    log::error!("Failed to read row {}", e);
                }
            }
        }

        Ok(RoutineSuccess::success(Message::new(
            "Peeked".to_string(),
            format!("{} rows written to {:?}", success_count, file_path),
        )))
    } else {
        while let Some(result) = stream.next().await {
            match result {
                Ok(value) => {
                    let message = serde_json::to_string(&value).unwrap();
                    println!("{}", message);
                    info!("{}", message);
                    success_count += 1;
                }
                Err(e) => {
                    log::error!("Failed to read row {}", e);
                }
            }
        }

        // Just a newline for output cleanliness
        println!();

        Ok(RoutineSuccess::success(Message::new(
            "Peeked".to_string(),
            format!("{} rows", success_count),
        )))
    }
}

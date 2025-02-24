use super::data_model::config::EndpointIngestionFormat;
use crate::framework::core::infrastructure::table::Table;
use crate::framework::core::plan::PlanningError;
use crate::framework::data_model::model::DataModel;
use crate::infrastructure::migration::InitialDataLoadError;
use crate::infrastructure::olap;
use crate::infrastructure::olap::clickhouse::config::ClickHouseConfig;
use crate::infrastructure::olap::clickhouse::model::ClickHouseTable;
use crate::infrastructure::stream::redpanda;
use crate::infrastructure::stream::redpanda::{
    send_with_back_pressure, wait_for_delivery, RedpandaConfig,
};
use crate::proto::infrastructure_map::InitialDataLoad as ProtoInitialDataLoad;
use clickhouse_rs::errors::codes::UNKNOWN_TABLE;
use clickhouse_rs::ClientHandle;
use futures::StreamExt;
use protobuf::MessageField;
use rdkafka::producer::DeliveryFuture;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct RouteMeta {
    pub topic_name: String,
    pub data_model: DataModel,
    pub format: EndpointIngestionFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitialDataLoad {
    pub table: Table,
    pub topic: String,
    pub status: InitialDataLoadStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InitialDataLoadStatus {
    InProgress(i64),
    Completed,
}

impl InitialDataLoad {
    pub(crate) fn expanded_display(&self) -> String {
        format!(
            "Initial data load: from table {} to topic {}",
            self.table.name, self.topic
        )
    }

    pub fn to_proto(&self) -> ProtoInitialDataLoad {
        ProtoInitialDataLoad {
            table: MessageField::some(self.table.to_proto()),
            topic: self.topic.clone(),
            progress: match &self.status {
                InitialDataLoadStatus::InProgress(i) => Some(*i as u64),
                InitialDataLoadStatus::Completed => None,
            },
            special_fields: Default::default(),
        }
    }

    pub fn from_proto(proto: ProtoInitialDataLoad) -> Self {
        InitialDataLoad {
            table: Table::from_proto(proto.table.unwrap()),
            topic: proto.topic,
            status: match proto.progress {
                Some(i) if i >= 100 => InitialDataLoadStatus::Completed,
                Some(i) => InitialDataLoadStatus::InProgress(i as i64),
                None => InitialDataLoadStatus::Completed,
            },
        }
    }
}

pub async fn resume_initial_data_load(
    table: &ClickHouseTable,
    click_house_config: &ClickHouseConfig,
    clickhouse_client: &mut ClientHandle,
    topic: &str,
    config: &RedpandaConfig,
    resume_from: i64,
) -> Result<(), InitialDataLoadError> {
    // TODO: we should probably have a lazy_static pool, after we actually use the global project instance
    let mut stream = olap::clickhouse_alt_client::select_all_as_json(
        &click_house_config.db_name,
        table,
        clickhouse_client,
        resume_from,
    )
    .await?;

    let producer = redpanda::create_idempotent_producer(config);

    let mut queue: VecDeque<DeliveryFuture> = VecDeque::new();

    while let Some(s) = stream.next().await {
        match s {
            Ok(value) => {
                let payload = value.to_string();
                send_with_back_pressure(&mut queue, &producer, topic, payload).await;
            }
            Err(e) => log::error!("Failure in row {:?}", e),
        }
    }
    for future in queue {
        wait_for_delivery(topic, future).await;
    }

    Ok(())
}

/// returns None if fully populated, Some(topic_record_count) if not
pub async fn check_topic_fully_populated(
    table_name: &str,
    clickhouse_config: &ClickHouseConfig,
    clickhouse: &mut ClientHandle,
    topic: &str,
    config: &RedpandaConfig,
) -> Result<Option<i64>, PlanningError> {
    let topic_size = redpanda::check_topic_size(topic, config).await?;
    let table_size = olap::clickhouse::check_table_size(table_name, clickhouse_config, clickhouse)
        .await
        .or_else(|err| match err {
            clickhouse_rs::errors::Error::Server(err) if err.code == UNKNOWN_TABLE => Ok(0),
            _ => Err(err),
        })?;

    Ok(if topic_size >= table_size {
        None
    } else {
        Some(topic_size)
    })
}

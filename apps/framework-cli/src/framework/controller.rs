use super::core::code_loader::FrameworkObject;
use super::data_model::config::EndpointIngestionFormat;
use crate::framework::core::infrastructure::table::Table;
use crate::framework::core::plan::PlanningError;
use crate::framework::data_model::model::DataModel;
use crate::infrastructure::migration::InitialDataLoadError;
use crate::infrastructure::olap;
use crate::infrastructure::olap::clickhouse::config::ClickHouseConfig;
use crate::infrastructure::olap::clickhouse::model::ClickHouseTable;
use crate::infrastructure::olap::clickhouse::queries::create_alias_for_table;
use crate::infrastructure::olap::clickhouse::queries::create_alias_query_from_table;
use crate::infrastructure::olap::clickhouse::version_sync::VersionSync;
use crate::infrastructure::olap::clickhouse::ConfiguredDBClient;
use crate::infrastructure::stream::redpanda;
use crate::infrastructure::stream::redpanda::{
    send_with_back_pressure, wait_for_delivery, RedpandaConfig,
};
use crate::proto::infrastructure_map::InitialDataLoad as ProtoInitialDataLoad;
use clickhouse_rs::errors::codes::UNKNOWN_TABLE;
use clickhouse_rs::ClientHandle;
use futures::StreamExt;
use log::{debug, error, info};
use protobuf::MessageField;
use rdkafka::producer::DeliveryFuture;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::Path;
use std::path::PathBuf;

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

pub async fn initial_data_load(
    table: &ClickHouseTable,
    configured_client: &ConfiguredDBClient,
    topic: &str,
    config: &RedpandaConfig,
) -> anyhow::Result<()> {
    let pool = olap::clickhouse_alt_client::get_pool(&configured_client.config);
    let mut client = pool.get_handle().await?;

    match check_topic_fully_populated(
        &table.name,
        &configured_client.config,
        &mut client,
        topic,
        config,
    )
    .await?
    {
        None => {
            debug!("Topic {} is already fully populated.", topic)
        }
        Some(offset) => {
            resume_initial_data_load(
                table,
                &configured_client.config,
                &mut client,
                topic,
                config,
                offset,
            )
            .await?;
        }
    };
    Ok(())
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
            Err(e) => log::error!("<DCM> Failure in row {:?}", e),
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

pub async fn create_sql_version_sync(
    version_sync: &VersionSync,
    configured_client: &ConfiguredDBClient,
) -> anyhow::Result<()> {
    let create_function_query = version_sync.create_function_query();
    olap::clickhouse::run_query(&create_function_query, configured_client).await?;

    let is_table_new =
        olap::clickhouse::check_is_table_new(&version_sync.dest_table, configured_client).await?;

    if is_table_new {
        debug!(
            "Performing initial load for table: {:?}",
            version_sync.dest_table.name
        );
        let initial_load_query = version_sync.clone().initial_load_query()?;
        olap::clickhouse::run_query(&initial_load_query, configured_client).await?;
    }

    let create_trigger_query = version_sync.create_trigger_query()?;
    olap::clickhouse::run_query(&create_trigger_query, configured_client).await?;

    Ok(())
}

pub(crate) async fn drop_table(
    db_name: &str,
    table: &ClickHouseTable,
    configured_client: &ConfiguredDBClient,
) -> anyhow::Result<()> {
    info!("<DCM> Dropping tables for: {:?}", table.name);
    let drop_data_table_query = table.drop_data_table_query(db_name)?;
    olap::clickhouse::run_query(&drop_data_table_query, configured_client).await?;

    Ok(())
}

// This is in the case of the new table doesn't have any changes in the new version
// of the schema, so we just point the new table to the old table.
pub async fn create_or_replace_table_alias(
    table: &ClickHouseTable,
    previous_version: &ClickHouseTable,
    configured_client: &ConfiguredDBClient,
    is_production: bool,
) -> anyhow::Result<()> {
    if !is_production {
        drop_table(&configured_client.config.db_name, table, configured_client).await?;
    }

    let query =
        create_alias_query_from_table(&configured_client.config.db_name, previous_version, table)?;
    olap::clickhouse::run_query(&query, configured_client).await?;

    Ok(())
}

pub async fn create_or_replace_latest_table_alias(
    fo: &FrameworkObject,
    configured_client: &ConfiguredDBClient,
) -> anyhow::Result<()> {
    if let Some(table) = &fo.table {
        info!(
            "<DCM> Creating table alias: {:?} -> {:?}",
            fo.data_model.name, table.name
        );

        match olap::clickhouse::get_engine(
            &configured_client.config.db_name,
            &fo.data_model.name,
            configured_client,
        )
        .await?
        {
            None => {}
            Some(v) if v == "View" => {
                olap::clickhouse::delete_table_or_view(&fo.data_model.name, configured_client)
                    .await?;
            }
            Some(engine) => {
                error!(
                    "Will not replace table {} alias, as it has engine {}.",
                    fo.data_model.name, engine
                );
                return Ok(());
            }
        };

        let query = create_alias_for_table(
            &configured_client.config.db_name,
            &fo.data_model.name,
            table,
        )?;
        olap::clickhouse::run_query(&query, configured_client).await?;
    }

    Ok(())
}

pub fn schema_file_path_to_ingest_route(
    base_path: &Path,
    path: &Path,
    data_model_name: String,
    version: &str,
) -> PathBuf {
    debug!("got data model path: {:?}", base_path);
    debug!("processing schema file into route: {:?}", path);

    // E.g. `model Foo` in `app/datamodels/inner/bar.prisma will have route
    // `ingest/inner/Foo/latest`
    let mut route = path.strip_prefix(base_path).unwrap().to_path_buf();
    route.set_file_name(data_model_name);

    debug!("route: {:?}", route);

    PathBuf::from("ingest").join(route).join(version)
}

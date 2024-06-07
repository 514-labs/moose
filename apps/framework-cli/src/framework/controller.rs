use std::collections::{HashMap, VecDeque};
use std::io::Error;
use std::io::ErrorKind;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use futures::StreamExt;
use log::{debug, error, info, warn};
use rdkafka::producer::DeliveryFuture;

use super::core::code_loader::FrameworkObject;
use super::data_model::config::EndpointIngestionFormat;
use crate::infrastructure::olap;
use crate::infrastructure::olap::clickhouse::model::ClickHouseTable;
use crate::infrastructure::olap::clickhouse::queries::create_alias_for_table;
use crate::infrastructure::olap::clickhouse::queries::create_alias_query_from_table;
use crate::infrastructure::olap::clickhouse::version_sync::{VersionSync, VersionSyncType};
use crate::infrastructure::olap::clickhouse::ConfiguredDBClient;
use crate::infrastructure::stream::redpanda;
use crate::infrastructure::stream::redpanda::{
    send_with_back_pressure, wait_for_delivery, RedpandaConfig,
};
use crate::project::Project;

#[derive(Debug, Clone)]
pub struct RouteMeta {
    pub original_file_path: PathBuf,
    pub topic_name: String,
    pub format: EndpointIngestionFormat,
}

pub async fn create_or_replace_version_sync(
    project: &Project,
    version_sync: &VersionSync,
    configured_client: &ConfiguredDBClient,
) -> anyhow::Result<()> {
    let drop_function_query = version_sync.drop_function_query();
    olap::clickhouse::run_query(&drop_function_query, configured_client).await?;

    if !project.is_production {
        let drop_trigger_query = version_sync.drop_trigger_query();
        olap::clickhouse::run_query(&drop_trigger_query, configured_client).await?;
    }

    match version_sync.sync_type {
        VersionSyncType::Sql(_) => create_sql_version_sync(version_sync, configured_client).await?,
        VersionSyncType::Ts(_) => {
            create_ts_version_sync_topics(project, version_sync).await?;
            initial_data_load(
                &version_sync.source_table,
                configured_client,
                &version_sync.topic_name("input"),
                &project.redpanda_config,
            )
            .await?;
        }
    };

    Ok(())
}

pub async fn initial_data_load(
    table: &ClickHouseTable,
    configured_client: &ConfiguredDBClient,
    topic: &str,
    config: &RedpandaConfig,
) -> anyhow::Result<()> {
    match check_topic_fully_populated(table, configured_client, topic, config).await? {
        None => {
            debug!("Topic {} is already fully populated.", topic)
        }
        Some(offset) => {
            // TODO: we should probably have a lazy_static pool, after we actually use the global project instance
            let pool = olap::clickhouse_alt_client::get_pool(&configured_client.config);
            let mut client = pool.get_handle().await?;
            let mut stream = olap::clickhouse_alt_client::select_all_as_json(
                &configured_client.config.db_name,
                table,
                &mut client,
                offset,
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
                    Err(e) => println!("<DCM> Failure in row {:?}", e),
                }
            }
            for future in queue {
                wait_for_delivery(topic, future).await;
            }
        }
    };
    Ok(())
}

/// returns None if fully populated, Some(topic_record_count) if not
async fn check_topic_fully_populated(
    table: &ClickHouseTable,
    configured_client: &ConfiguredDBClient,
    topic: &str,
    config: &RedpandaConfig,
) -> anyhow::Result<Option<i64>> {
    let topic_size = redpanda::check_topic_size(topic, config).await?;
    let table_size = olap::clickhouse::check_table_size(table, configured_client).await?;

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

pub async fn create_ts_version_sync_topics(
    project: &Project,
    version_sync: &VersionSync,
) -> anyhow::Result<()> {
    let input_topic = version_sync.topic_name("input");
    let output_topic = version_sync.topic_name("output");

    redpanda::create_topics(
        &project.redpanda_config.clone(),
        vec![input_topic, output_topic],
    )
    .await?;
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

pub(crate) async fn create_or_replace_tables(
    table: &ClickHouseTable,
    configured_client: &ConfiguredDBClient,
    is_production: bool,
) -> anyhow::Result<()> {
    info!("<DCM> Creating table: {:?}", table.name);
    let create_data_table_query =
        table.create_data_table_query(&configured_client.config.db_name)?;

    olap::clickhouse::check_ready(configured_client)
        .await
        .map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("Failed to connect to clickhouse: {}", e),
            )
        })?;

    // Clickhouse doesn't support dropping a view if it doesn't exist, so we need to drop it first in case the schema has changed
    if !is_production {
        drop_table(&configured_client.config.db_name, table, configured_client).await?;
    }

    olap::clickhouse::run_query(&create_data_table_query, configured_client).await?;

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

pub async fn set_up_topic_and_tables_and_route(
    project: &Project,
    fo: &FrameworkObject,
    previous_version: &Option<(String, HashMap<String, FrameworkObject>)>,
    configured_client: &ConfiguredDBClient,
    route_table: &mut HashMap<PathBuf, RouteMeta>,
    ingest_route: PathBuf,
    is_latest: bool,
) -> anyhow::Result<()> {
    let topic = fo.topic.clone();

    let previous_fo_opt = match previous_version {
        None => None,
        Some((_, previous_models)) => previous_models.get(&fo.data_model.name),
    };

    let topic_name = match (
        previous_fo_opt,
        &fo.table,
        &previous_fo_opt.and_then(|previous_fo| previous_fo.table.clone()),
    ) {
        // In the case where the previous version of the data model and the new version of the data model are the same
        // we just need to use pointers to the old table and topic
        (Some(previous_fo), Some(current_table), Some(previous_table))
            if previous_fo.data_model == fo.data_model =>
        {
            info!(
                "Data model {} has not changed, using previous version table {} and topic {}",
                fo.data_model.name,
                previous_fo
                    .table
                    .clone()
                    .map(|table| { table.name })
                    .unwrap_or("None".to_string()),
                previous_fo.topic
            );
            // In this case no need for a topic on the current table since it is all the same as the previous table.
            match redpanda::delete_topics(&project.redpanda_config, vec![topic]).await {
                Ok(_) => info!("Topics deleted successfully"),
                Err(e) => warn!("Failed to delete topics: {}", e),
            }

            if fo.data_model.config.storage.enabled {
                create_or_replace_table_alias(
                    current_table,
                    previous_table,
                    configured_client,
                    project.is_production,
                )
                .await?;
            }

            // In the case where we use a view here, we need to use the previous topic that was set
            // That topic may be the topic of n - 2 version of the data model. Right now the Route object
            // is where this is kept.
            // this is a gross way to find the previous ingest route
            let previous_version = previous_version.as_ref().unwrap().0.as_str();
            let old_base_path = project.old_version_location(previous_version)?;
            let old_ingest_route = schema_file_path_to_ingest_route(
                &old_base_path,
                &previous_fo.original_file_path,
                fo.data_model.name.clone(),
                previous_version,
            );

            route_table
                .get(&old_ingest_route)
                .unwrap()
                // this might be chained multiple times,
                // so we cannot just use the previous.table.name
                .topic_name
                .clone()
        }
        _ => {
            match redpanda::create_topics(&project.redpanda_config, vec![topic]).await {
                Ok(_) => info!("Topics created successfully"),
                Err(e) => warn!("Failed to create topics: {}", e),
            }

            if let Some(table) = &fo.table {
                debug!("Creating table: {:?}", table.name);

                create_or_replace_tables(table, configured_client, project.is_production).await?;

                debug!("Table created: {:?}", table.name);
            }

            fo.topic.clone()
        }
    };

    if is_latest && fo.data_model.config.storage.enabled {
        create_or_replace_latest_table_alias(fo, configured_client).await?;
    }

    route_table.insert(
        ingest_route.clone(),
        RouteMeta {
            original_file_path: fo.original_file_path.clone(),
            topic_name,
            format: fo.data_model.config.ingestion.format.clone(),
        },
    );

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn process_objects(
    framework_objects: &HashMap<String, FrameworkObject>,
    previous_version: &Option<(String, HashMap<String, FrameworkObject>)>,
    project: Arc<Project>,
    schema_dir: &Path,
    configured_client: &ConfiguredDBClient,
    route_table: &mut HashMap<PathBuf, RouteMeta>,
    version: &str,
) -> anyhow::Result<()> {
    let is_latest = version == project.version();

    for (_, fo) in framework_objects.iter() {
        let ingest_route = schema_file_path_to_ingest_route(
            schema_dir,
            &fo.original_file_path,
            fo.data_model.name.clone(),
            version,
        );

        set_up_topic_and_tables_and_route(
            &project,
            fo,
            previous_version,
            configured_client,
            route_table,
            ingest_route.clone(),
            is_latest,
        )
        .await?;
    }
    Ok(())
}

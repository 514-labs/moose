use std::collections::{HashMap, VecDeque};
use std::io::Error;
use std::io::ErrorKind;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use futures::StreamExt;
use log::{debug, error, info, warn};
use rdkafka::producer::DeliveryFuture;

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
use crate::project::PROJECT;
use crate::project::{AggregationSet, Project};
#[cfg(test)]
use crate::utilities::constants::SCHEMAS_DIR;

use super::data_model;
use super::data_model::config::EndpointIngestionFormat;
use super::data_model::config::ModelConfigurationError;
use super::data_model::is_schema_file;
use super::data_model::parser::{parse_data_model_file, DataModelParsingError};
use super::data_model::schema::ColumnType;
use super::data_model::schema::DataEnum;
use super::data_model::schema::DataModel;
use super::data_model::DuplicateModelError;

#[derive(Debug, Clone)]
pub struct FrameworkObject {
    pub data_model: DataModel,
    pub table: Option<ClickHouseTable>,
    pub topic: String,
    pub original_file_path: PathBuf,
}

#[derive(Debug, thiserror::Error)]
#[error("Failed to build internal framework object representation")]
#[non_exhaustive]
pub enum MappingError {
    ClickhouseError(#[from] crate::infrastructure::olap::clickhouse::errors::ClickhouseError),
    TypescriptError(#[from] super::typescript::generator::TypescriptGeneratorError),
}

pub fn framework_object_mapper(
    s: DataModel,
    original_file_path: &Path,
    version: &str,
) -> Result<FrameworkObject, MappingError> {
    let clickhouse_table = if s.config.storage.enabled {
        Some(olap::clickhouse::mapper::std_table_to_clickhouse_table(
            s.to_table(version),
        )?)
    } else {
        None
    };

    let topic = format!("{}_{}", s.name.clone(), version.replace('.', "_"));

    Ok(FrameworkObject {
        data_model: s.clone(),
        table: clickhouse_table,
        topic,
        original_file_path: original_file_path.to_path_buf(),
    })
}

#[derive(Debug, Clone)]
pub struct SchemaVersion {
    pub base_path: PathBuf,
    pub models: HashMap<String, FrameworkObject>,
}

impl SchemaVersion {
    pub fn get_all_enums(&self) -> Vec<DataEnum> {
        let mut enums = Vec::new();
        for model in self.models.values() {
            for column in &model.data_model.columns {
                if let ColumnType::Enum(data_enum) = &column.data_type {
                    enums.push(data_enum.clone());
                }
            }
        }
        enums
    }

    pub fn get_all_models(&self) -> Vec<DataModel> {
        self.models
            .values()
            .map(|model| model.data_model.clone())
            .collect()
    }
}

// TODO: save this object somewhere so that we can clean up removed models
// TODO abstract this with an iterator and some helper functions to make the internal state hidden.
// That would enable us to do things like .next() and .peek() on the iterator that are now not possible
#[derive(Debug, Clone)]
pub struct FrameworkObjectVersions {
    pub current_version: String,
    pub current_models: SchemaVersion,
    pub previous_version_models: HashMap<String, SchemaVersion>,
}

impl FrameworkObjectVersions {
    pub fn new(current_version: String, current_schema_directory: PathBuf) -> Self {
        FrameworkObjectVersions {
            current_version,
            current_models: SchemaVersion {
                base_path: current_schema_directory,
                models: HashMap::new(),
            },
            previous_version_models: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RouteMeta {
    pub original_file_path: PathBuf,
    pub topic_name: String,
    pub format: EndpointIngestionFormat,
}

pub fn get_all_framework_objects(
    framework_objects: &mut HashMap<String, FrameworkObject>,
    schema_dir: &Path,
    version: &str,
    aggregations: &AggregationSet,
) -> anyhow::Result<()> {
    if schema_dir.is_dir() {
        for entry in std::fs::read_dir(schema_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                debug!("<DCM> Processing directory: {:?}", path);
                get_all_framework_objects(framework_objects, &path, version, aggregations)?;
            } else if is_schema_file(&path) {
                debug!("<DCM> Processing file: {:?}", path);
                let objects = get_framework_objects_from_schema_file(&path, version, aggregations)?;
                for fo in objects {
                    DuplicateModelError::try_insert(framework_objects, fo, &path)?;
                }
            }
        }
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
#[error("Failed to get the Data Model configuration")]
#[non_exhaustive]
pub enum DataModelError {
    Configuration(#[from] ModelConfigurationError),
    Parsing(#[from] DataModelParsingError),
    Mapping(#[from] MappingError),

    #[error("{message}")]
    Other {
        message: String,
    },
}

pub fn get_framework_objects_from_schema_file(
    path: &Path,
    version: &str,
    aggregations: &AggregationSet,
) -> Result<Vec<FrameworkObject>, DataModelError> {
    let framework_objects = parse_data_model_file(path)?;
    let mut indexed_models = HashMap::new();

    for model in framework_objects.models {
        if aggregations.current_version == version
            && aggregations.names.contains(model.name.clone().trim())
        {
            return Err(DataModelError::Other {
                message: format!(
                    "Model & aggregation {} cannot have the same name",
                    model.name
                ),
            });
        }

        indexed_models.insert(model.name.clone().trim().to_lowercase(), model);
    }

    let data_models_configs = data_model::config::get(path)?;
    for (config_variable_name, config) in data_models_configs.iter() {
        let sanitized_config_name = config_variable_name.trim().to_lowercase();
        match sanitized_config_name.strip_suffix("config") {
            Some(config_name_without_suffix) => {
                let data_model_opt = indexed_models.get_mut(config_name_without_suffix);
                if let Some(data_model) = data_model_opt {
                    data_model.config = config.clone();
                }
            }
            None => {
                return Err(DataModelError::Other { message: format!("Config name exports have to be of the format <dataModelName>Config so that they can be correlated to the proper datamodel. \n {} is not respecting this pattern", config_variable_name) })
            }
        }
    }

    let mut to_return = Vec::new();
    for model in indexed_models.into_values() {
        to_return.push(framework_object_mapper(model, path, version)?);
    }

    Ok(to_return)
}

pub async fn create_or_replace_version_sync(
    project: &Project,
    version_sync: &VersionSync,
    configured_client: &ConfiguredDBClient,
) -> anyhow::Result<()> {
    let drop_function_query = version_sync.drop_function_query();
    olap::clickhouse::run_query(&drop_function_query, configured_client).await?;

    if !PROJECT.lock().unwrap().is_production {
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
            let mut stream =
                olap::clickhouse_alt_client::select_all_as_json(table, &mut client, offset).await?;

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
    table: &ClickHouseTable,
    configured_client: &ConfiguredDBClient,
) -> anyhow::Result<()> {
    info!("<DCM> Dropping tables for: {:?}", table.name);
    let drop_data_table_query = table.drop_data_table_query()?;
    olap::clickhouse::run_query(&drop_data_table_query, configured_client).await?;

    Ok(())
}

// This is in the case of the new table doesn't have any changes in the new version
// of the schema, so we just point the new table to the old table.
pub async fn create_or_replace_table_alias(
    table: &ClickHouseTable,
    previous_version: &ClickHouseTable,
    configured_client: &ConfiguredDBClient,
) -> anyhow::Result<()> {
    if !PROJECT.lock().unwrap().is_production {
        drop_table(table, configured_client).await?;
    }

    let query = create_alias_query_from_table(previous_version, table)?;
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

        match olap::clickhouse::get_engine(&table.db_name, &fo.data_model.name, configured_client)
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

        let query = create_alias_for_table(&fo.data_model.name, table)?;
        olap::clickhouse::run_query(&query, configured_client).await?;
    }

    Ok(())
}

pub(crate) async fn create_or_replace_tables(
    table: &ClickHouseTable,
    configured_client: &ConfiguredDBClient,
) -> anyhow::Result<()> {
    info!("<DCM> Creating table: {:?}", table.name);
    let create_data_table_query = table.create_data_table_query()?;

    olap::clickhouse::check_ready(configured_client)
        .await
        .map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("Failed to connect to clickhouse: {}", e),
            )
        })?;

    // Clickhouse doesn't support dropping a view if it doesn't exist, so we need to drop it first in case the schema has changed
    if !PROJECT.lock().unwrap().is_production {
        drop_table(table, configured_client).await?;
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
                create_or_replace_table_alias(current_table, previous_table, configured_client)
                    .await?;
            }

            previous_fo.topic.clone()
        }
        _ => {
            match redpanda::create_topics(&project.redpanda_config, vec![topic]).await {
                Ok(_) => info!("Topics created successfully"),
                Err(e) => warn!("Failed to create topics: {}", e),
            }

            if let Some(table) = &fo.table {
                debug!("Creating table: {:?}", table.name);

                create_or_replace_tables(table, configured_client).await?;

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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    #[test]
    fn test_get_all_framework_objects() {
        use super::*;
        let manifest_location = env!("CARGO_MANIFEST_DIR");
        let schema_dir = PathBuf::from(manifest_location)
            .join("tests/test_project")
            .join(SCHEMAS_DIR);

        let mut framework_objects = HashMap::new();
        let aggregations = AggregationSet {
            current_version: "0.0".to_string(),
            names: HashSet::new(),
        };
        let result =
            get_all_framework_objects(&mut framework_objects, &schema_dir, "0.0", &aggregations);
        assert!(result.is_ok());
        assert_eq!(framework_objects.len(), 2);
    }
}

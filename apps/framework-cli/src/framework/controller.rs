use std::collections::HashMap;
use std::io::Error;
use std::io::ErrorKind;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use log::debug;
use log::info;

use crate::framework;
use crate::framework::languages::CodeGenerator;
use crate::framework::languages::SupportedLanguages;
use crate::framework::sdks::TypescriptObjects;
use crate::framework::typescript::get_typescript_models_dir;
use crate::framework::typescript::SendFunction;
use crate::infrastructure::olap;
use crate::infrastructure::olap::clickhouse::ConfiguredDBClient;
use crate::infrastructure::olap::clickhouse::{ClickhouseKafkaTrigger, VERSION_SYNC_REGEX};
use crate::infrastructure::olap::clickhouse::{ClickhouseTable, VersionSync};
use crate::infrastructure::stream;
use crate::project::Project;
#[cfg(test)]
use crate::utilities::constants::SCHEMAS_DIR;

use super::schema::{is_prisma_file, DataModel};
use super::schema::{parse_schema_file, DuplicateModelError};
use super::typescript::TypescriptInterface;

#[derive(Debug, Clone)]
pub struct FrameworkObject {
    pub data_model: DataModel,
    pub table: ClickhouseTable,
    pub topic: String,
    pub ts_interface: TypescriptInterface,
    pub original_file_path: PathBuf,
}

pub fn framework_object_mapper(
    s: DataModel,
    original_file_path: &Path,
    version: &str,
) -> FrameworkObject {
    let clickhouse_table =
        olap::clickhouse::mapper::std_table_to_clickhouse_table(s.to_table(version));
    FrameworkObject {
        data_model: s.clone(),
        table: clickhouse_table,
        topic: s.name.clone(),
        ts_interface: framework::typescript::mapper::std_table_to_typescript_interface(
            s.to_table(version),
            s.name.as_str(),
        ),
        original_file_path: original_file_path.to_path_buf(),
    }
}

#[derive(Debug, Clone)]
pub struct SchemaVersion {
    pub base_path: PathBuf,
    pub models: HashMap<String, FrameworkObject>,
    pub typescript_objects: HashMap<String, TypescriptObjects>,
}

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
                typescript_objects: HashMap::new(),
            },
            previous_version_models: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RouteMeta {
    pub original_file_path: PathBuf,
    pub table_name: String,
    pub view_name: Option<String>,
}

pub fn get_all_version_syncs(
    project: &Project,
    framework_object_versions: &FrameworkObjectVersions,
) -> anyhow::Result<Vec<VersionSync>> {
    let flows_dir = project.flows_dir();
    let mut version_syncs = vec![];
    for entry in std::fs::read_dir(flows_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            if let Some(captures) =
                VERSION_SYNC_REGEX.captures(entry.file_name().to_string_lossy().as_ref())
            {
                let from_table_name = captures.get(1).unwrap().as_str();
                let to_table_name = captures.get(4).map_or(from_table_name, |m| m.as_str());

                let from_version = captures.get(2).unwrap().as_str().replace('_', ".");
                let to_version = captures.get(5).unwrap().as_str().replace('_', ".");

                let from_version_models = framework_object_versions
                    .previous_version_models
                    .get(&from_version);
                let to_version_models = if to_version == framework_object_versions.current_version {
                    Some(&framework_object_versions.current_models)
                } else {
                    framework_object_versions
                        .previous_version_models
                        .get(&to_version)
                };

                match (from_version_models, to_version_models) {
                    (Some(from_version_models), Some(to_version_models)) => {
                        let from_table = from_version_models.models.get(from_table_name);
                        let to_table = to_version_models.models.get(to_table_name);
                        match (from_table, to_table) {
                            (Some(from_table), Some(to_table)) => {
                                let version_sync = VersionSync {
                                    db_name: from_table.table.db_name.clone(),
                                    model_name: from_table.data_model.name.clone(),
                                    source_version: from_version.clone(),
                                    source_table: from_table.table.clone(),
                                    dest_version: to_version.clone(),
                                    dest_table: to_table.table.clone(),
                                    migration_function: std::fs::read_to_string(&path)?,
                                };
                                version_syncs.push(version_sync);
                            }
                            _ => {
                                return Err(anyhow::anyhow!(
                                    "Failed to find tables in versions {:?}",
                                    path.file_name()
                                ));
                            }
                        }
                    }
                    _ => {
                        debug!(
                            "Version unavailable for version sync {:?}. from: {:?} to: {:?}",
                            path.file_name(),
                            from_version_models,
                            to_version_models,
                        );
                    }
                }
            };

            debug!("Processing version sync: {:?}.", path);
        }
    }
    Ok(version_syncs)
}

pub fn get_all_framework_objects(
    framework_objects: &mut HashMap<String, FrameworkObject>,
    schema_dir: &Path,
    version: &str,
) -> anyhow::Result<()> {
    if schema_dir.is_dir() {
        for entry in std::fs::read_dir(schema_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                debug!("Processing directory: {:?}", path);
                get_all_framework_objects(framework_objects, &path, version)?;
            } else if is_prisma_file(&path) {
                debug!("Processing file: {:?}", path);
                let objects = get_framework_objects_from_schema_file(&path, version)?;
                for fo in objects {
                    DuplicateModelError::try_insert(framework_objects, fo, &path)?;
                }
            }
        }
    }
    Ok(())
}

pub fn get_framework_objects_from_schema_file(
    path: &Path,
    version: &str,
) -> Result<Vec<FrameworkObject>, Error> {
    let framework_objects =
        parse_schema_file::<FrameworkObject>(path, version, framework_object_mapper).map_err(
            |e| {
                Error::new(
                    ErrorKind::Other,
                    format!("Failed to parse schema file. Error {}", e),
                )
            },
        )?;
    Ok(framework_objects)
}

pub async fn drop_kafka_trigger(
    view: &ClickhouseKafkaTrigger,
    configured_client: &ConfiguredDBClient,
) -> anyhow::Result<()> {
    let drop_view_query = view.drop_materialized_view_query()?;
    olap::clickhouse::run_query(&drop_view_query, configured_client).await?;
    Ok(())
}

pub(crate) async fn create_or_replace_kafka_trigger(
    view: &ClickhouseKafkaTrigger,
    configured_client: &ConfiguredDBClient,
) -> anyhow::Result<()> {
    let create_view_query = view.create_materialized_view_query().map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("Failed to get clickhouse query: {:?}", e),
        )
    })?;

    // Clickhouse doesn't support dropping a view if it doesn't exist, so we need to drop it first in case the schema has changed
    drop_kafka_trigger(view, configured_client).await?;
    olap::clickhouse::run_query(&create_view_query, configured_client)
        .await
        .map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("Failed to create view in clickhouse: {}", e),
            )
        })?;
    Ok(())
}

pub async fn create_or_replace_version_sync(
    version_sync: VersionSync,
    configured_client: &ConfiguredDBClient,
) -> anyhow::Result<()> {
    let drop_function_query = version_sync.drop_function_query();
    let drop_trigger_query = version_sync.drop_trigger_query();
    let create_function_query = version_sync.create_function_query();
    let create_trigger_query = version_sync.create_trigger_query();

    olap::clickhouse::run_query(&drop_function_query, configured_client).await?;
    olap::clickhouse::run_query(&drop_trigger_query, configured_client).await?;
    olap::clickhouse::run_query(&create_function_query, configured_client).await?;
    olap::clickhouse::run_query(&create_trigger_query, configured_client).await?;
    Ok(())
}

pub(crate) async fn drop_tables(
    fo: &FrameworkObject,
    configured_client: &ConfiguredDBClient,
) -> anyhow::Result<()> {
    info!("Dropping tables for: {:?}", fo.table.name);

    let drop_data_table_query = fo.table.drop_kafka_table_query()?;
    let drop_kafka_table_query = fo.table.drop_data_table_query()?;

    olap::clickhouse::run_query(&drop_data_table_query, configured_client).await?;
    olap::clickhouse::run_query(&drop_kafka_table_query, configured_client).await?;
    Ok(())
}

pub(crate) async fn create_or_replace_tables(
    project_name: &str,
    fo: &FrameworkObject,
    configured_client: &ConfiguredDBClient,
) -> anyhow::Result<()> {
    info!("Creating table: {:?}", fo.table.name);

    let create_data_table_query = fo.table.create_data_table_query()?;
    let create_kafka_table_query = fo.table.create_kafka_table_query(project_name)?;

    olap::clickhouse::check_ready(configured_client)
        .await
        .map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("Failed to connect to clickhouse: {}", e),
            )
        })?;

    // Clickhouse doesn't support dropping a view if it doesn't exist, so we need to drop it first in case the schema has changed
    drop_tables(fo, configured_client).await?;

    olap::clickhouse::run_query(&create_data_table_query, configured_client).await?;
    olap::clickhouse::run_query(&create_kafka_table_query, configured_client).await?;
    Ok(())
}

pub(crate) fn create_language_objects(
    fo: &FrameworkObject,
    ingest_route: &Path,
    project: Arc<Project>,
) -> Result<TypescriptObjects, Error> {
    info!("Creating typescript interface: {:?}", fo.ts_interface);
    let ts_interface_code = fo.ts_interface.create_code().map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("Failed to get typescript interface: {:?}", e),
        )
    })?;

    let send_func = SendFunction::new(
        fo.ts_interface.clone(),
        project.http_server_config.url(),
        ingest_route.to_str().unwrap().to_string(),
    );
    let send_func_code = send_func.create_code().map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("Failed to generate send function: {:?}", e),
        )
    })?;
    let typescript_dir = get_typescript_models_dir(project)?;
    let interface_file_path = typescript_dir.join(format!("{}.ts", fo.ts_interface.file_name()));
    let send_func_file_path = typescript_dir.join(send_func.interface.send_function_file_name());

    debug!(
        "Writing typescript interface to file: {:?}",
        interface_file_path
    );

    framework::languages::write_code_to_file(
        SupportedLanguages::Typescript,
        interface_file_path,
        ts_interface_code,
    )
    .map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("Failed to write typescript interface to file: {:?}", e),
        )
    })?;
    framework::languages::write_code_to_file(
        SupportedLanguages::Typescript,
        send_func_file_path,
        send_func_code,
    )
    .map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("Failed to write typescript function to file: {:?}", e),
        )
    })?;
    Ok(TypescriptObjects::new(fo.ts_interface.clone(), send_func))
}

pub async fn remove_table_and_topics_from_schema_file_path(
    project: &Project,
    schema_file_path: &Path,
    route_table: &mut HashMap<PathBuf, RouteMeta>,
    configured_client: &ConfiguredDBClient,
) -> anyhow::Result<()> {
    //need to get the path of the file, scan the route table and remove all the files that need to be deleted.
    // This doesn't have to be as fast as the scanning for routes in the web server so we're ok with the scan here.

    for (k, meta) in route_table.clone().into_iter() {
        if meta.original_file_path == schema_file_path {
            let topics = vec![meta.table_name.clone()];
            match stream::redpanda::delete_topics(&project.redpanda_config, topics).await {
                Ok(_) => println!("Topics deleted successfully"),
                Err(e) => eprintln!("Failed to delete topics: {}", e),
            }

            olap::clickhouse::delete_table_or_view(meta.table_name, configured_client)
                .await
                .map_err(|e| {
                    Error::new(
                        ErrorKind::Other,
                        format!("Failed to create table in clickhouse: {}", e),
                    )
                })?;

            if let Some(view_name) = meta.view_name {
                olap::clickhouse::delete_table_or_view(view_name, configured_client)
                    .await
                    .map_err(|e| {
                        Error::new(
                            ErrorKind::Other,
                            format!("Failed to create table in clickhouse: {}", e),
                        )
                    })?;
            }

            (*route_table).remove(&k);
        }
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

pub async fn process_objects(
    framework_objects: &HashMap<String, FrameworkObject>,
    project: Arc<Project>,
    schema_dir: &Path,
    configured_client: &ConfiguredDBClient,
    compilable_objects: &mut HashMap<String, TypescriptObjects>, // Objects that require compilation after processing
    route_table: &mut HashMap<PathBuf, RouteMeta>,
    version: &str,
) -> anyhow::Result<()> {
    for (name, fo) in framework_objects.iter() {
        let ingest_route = schema_file_path_to_ingest_route(
            schema_dir,
            &fo.original_file_path,
            fo.data_model.name.clone(),
            version,
        );
        let topics = vec![fo.topic.clone()];
        match stream::redpanda::create_topics(&project.redpanda_config, topics).await {
            Ok(_) => println!("Topics created successfully"),
            Err(e) => eprintln!("Failed to create topics: {}", e),
        }

        debug!("Creating table & view: {:?}", fo.table.name);

        create_or_replace_tables(&project.name(), fo, configured_client).await?;

        let view = ClickhouseKafkaTrigger::from_clickhouse_table(&fo.table);
        create_or_replace_kafka_trigger(&view, configured_client).await?;

        debug!("Table created: {:?}", fo.table.name);

        let typescript_objects = create_language_objects(fo, &ingest_route, project.clone())?;
        compilable_objects.insert(name.clone(), typescript_objects);

        route_table.insert(
            ingest_route,
            RouteMeta {
                original_file_path: fo.original_file_path.clone(),
                table_name: fo.table.name.clone(),
                view_name: Some(fo.table.view_name()),
            },
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_get_all_framework_objects() {
        use super::*;
        let manifest_location = env!("CARGO_MANIFEST_DIR");
        let schema_dir = PathBuf::from(manifest_location)
            .join("tests/test_project")
            .join(SCHEMAS_DIR);
        let mut framework_objects = HashMap::new();
        let result = get_all_framework_objects(&mut framework_objects, &schema_dir, "0.0");
        assert!(result.is_ok());
        assert_eq!(framework_objects.len(), 2);
    }
}

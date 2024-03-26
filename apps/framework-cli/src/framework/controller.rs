use std::collections::HashMap;
use std::io::Error;
use std::io::ErrorKind;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use log::{debug, info, warn};

use crate::framework;
use crate::framework::languages::CodeGenerator;
use crate::framework::languages::SupportedLanguages;
use crate::framework::sdks::TypescriptObjects;
use crate::framework::typescript::get_typescript_models_dir;
use crate::framework::typescript::SendFunction;
use crate::infrastructure::olap;
use crate::infrastructure::olap::clickhouse::model::ClickHouseTable;
use crate::infrastructure::olap::clickhouse::queries::CreateAliasQuery;
use crate::infrastructure::olap::clickhouse::version_sync::VersionSync;
use crate::infrastructure::olap::clickhouse::ConfiguredDBClient;
use crate::infrastructure::stream::redpanda;
use crate::project::Project;
use crate::project::PROJECT;
#[cfg(test)]
use crate::utilities::constants::SCHEMAS_DIR;

use super::schema::{is_prisma_file, DataModel};
use super::schema::{parse_schema_file, DuplicateModelError};
use super::typescript::TypescriptInterface;

#[derive(Debug, Clone)]
pub struct FrameworkObject {
    pub data_model: DataModel,
    pub table: ClickHouseTable,
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

    let topic = format!("{}_{}", s.name.clone(), version.replace('.', "_"));

    FrameworkObject {
        data_model: s.clone(),
        table: clickhouse_table,
        topic,
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

pub async fn create_or_replace_version_sync(
    version_sync: VersionSync,
    configured_client: &ConfiguredDBClient,
) -> anyhow::Result<()> {
    let drop_function_query = version_sync.drop_function_query();
    let create_function_query = version_sync.create_function_query();

    olap::clickhouse::run_query(&drop_function_query, configured_client).await?;
    olap::clickhouse::run_query(&create_function_query, configured_client).await?;

    if !PROJECT.lock().unwrap().is_production {
        let drop_trigger_query = version_sync.drop_trigger_query();
        let create_trigger_query = version_sync.create_trigger_query();
        olap::clickhouse::run_query(&drop_trigger_query, configured_client).await?;
        olap::clickhouse::run_query(&create_trigger_query, configured_client).await?;
    }

    Ok(())
}

pub(crate) async fn drop_tables(
    fo: &FrameworkObject,
    configured_client: &ConfiguredDBClient,
) -> anyhow::Result<()> {
    info!("Dropping tables for: {:?}", fo.table.name);

    let drop_data_table_query = fo.table.drop_data_table_query()?;
    olap::clickhouse::run_query(&drop_data_table_query, configured_client).await?;

    Ok(())
}

pub async fn create_or_replace_table_alias(
    fo: &FrameworkObject,
    previous_version: &FrameworkObject,
    configured_client: &ConfiguredDBClient,
) -> anyhow::Result<()> {
    drop_tables(fo, configured_client).await?;

    let query = CreateAliasQuery::build(&previous_version.table, &fo.table);
    olap::clickhouse::run_query(&query, configured_client).await?;

    Ok(())
}

pub(crate) async fn create_or_replace_tables(
    fo: &FrameworkObject,
    configured_client: &ConfiguredDBClient,
) -> anyhow::Result<()> {
    info!("Creating table: {:?}", fo.table.name);

    let create_data_table_query = fo.table.create_data_table_query()?;

    olap::clickhouse::check_ready(configured_client)
        .await
        .map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("Failed to connect to clickhouse: {}", e),
            )
        })?;

    olap::clickhouse::run_query(&create_data_table_query, configured_client).await?;

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
            match redpanda::delete_topics(&project.redpanda_config, topics).await {
                Ok(_) => info!("Topics deleted successfully"),
                Err(e) => warn!("Failed to delete topics: {}", e),
            }

            olap::clickhouse::delete_table_or_view(meta.table_name, configured_client)
                .await
                .map_err(|e| {
                    Error::new(
                        ErrorKind::Other,
                        format!("Failed to create table in clickhouse: {}", e),
                    )
                })?;

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

pub async fn set_up_topic_and_tables_and_route(
    project: &Project,
    fo: &FrameworkObject,
    previous_version: &Option<(String, HashMap<String, FrameworkObject>)>,
    configured_client: &ConfiguredDBClient,
    route_table: &mut HashMap<PathBuf, RouteMeta>,
    ingest_route: PathBuf,
) -> anyhow::Result<()> {
    let topic = fo.topic.clone();

    let previous_fo_opt = match previous_version {
        None => None,
        Some((_, previous_models)) => previous_models.get(&fo.data_model.name),
    };

    let ingest_topic_name;
    match previous_fo_opt {
        Some(previous_fo) if previous_fo.data_model == fo.data_model => {
            match redpanda::delete_topics(&project.redpanda_config, vec![topic]).await {
                Ok(_) => info!("Topics deleted successfully"),
                Err(e) => warn!("Failed to delete topics: {}", e),
            }

            create_or_replace_table_alias(fo, previous_fo, configured_client).await?;

            // this is a gross way to find the previous ingest route
            let previous_version = previous_version.as_ref().unwrap().0.as_str();
            let old_base_path = project.old_version_location(previous_version)?;
            let old_ingest_route = schema_file_path_to_ingest_route(
                &old_base_path,
                &previous_fo.original_file_path,
                fo.data_model.name.clone(),
                previous_version,
            );
            ingest_topic_name = route_table
                .get(&old_ingest_route)
                .unwrap()
                // this might be chained multiple times,
                // so we cannot just use the previous.table.name
                .table_name
                .clone();
        }
        _ => {
            match redpanda::create_topics(&project.redpanda_config, vec![topic]).await {
                Ok(_) => info!("Topics created successfully"),
                Err(e) => warn!("Failed to create topics: {}", e),
            }

            debug!("Creating table: {:?}", fo.table.name);

            create_or_replace_tables(fo, configured_client).await?;

            debug!("Table created: {:?}", fo.table.name);

            ingest_topic_name = fo.table.name.clone();
        }
    };

    route_table.insert(
        ingest_route.clone(),
        RouteMeta {
            original_file_path: fo.original_file_path.clone(),
            table_name: ingest_topic_name,
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

        set_up_topic_and_tables_and_route(
            &project,
            fo,
            previous_version,
            configured_client,
            route_table,
            ingest_route.clone(),
        )
        .await?;

        let typescript_objects = create_language_objects(fo, &ingest_route, project.clone())?;
        compilable_objects.insert(name.clone(), typescript_objects);
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

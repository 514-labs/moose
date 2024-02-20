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
use crate::infrastructure::olap::clickhouse::ClickhouseKafkaTrigger;
use crate::infrastructure::olap::clickhouse::ClickhouseTable;
use crate::infrastructure::olap::clickhouse::ConfiguredDBClient;
use crate::infrastructure::stream;
use crate::project::Project;
use crate::utilities::constants::SCHEMAS_DIR;

use super::schema::parse_schema_file;
use super::schema::DataModel;
use super::typescript::TypescriptInterface;

#[derive(Debug, Clone)]
pub struct FrameworkObject {
    pub data_model: DataModel,
    pub table: ClickhouseTable,
    pub topic: String,
    pub ts_interface: TypescriptInterface,
}

pub fn framework_object_mapper(s: DataModel) -> FrameworkObject {
    let clickhouse_table = olap::clickhouse::mapper::std_table_to_clickhouse_table(s.to_table());
    FrameworkObject {
        data_model: s.clone(),
        table: clickhouse_table,
        topic: s.name.clone(),
        ts_interface: framework::typescript::mapper::std_table_to_typescript_interface(
            s.to_table(),
        ),
    }
}

#[derive(Debug, Clone)]
pub struct RouteMeta {
    pub original_file_path: PathBuf,
    pub table_name: String,
    pub view_name: Option<String>,
}

pub fn get_all_framework_objects(
    framework_objects: &mut Vec<FrameworkObject>,
    schema_dir: &Path,
) -> Result<Vec<FrameworkObject>, Error> {
    if schema_dir.is_dir() {
        for entry in std::fs::read_dir(schema_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                debug!("Processing directory: {:?}", path);
                get_all_framework_objects(framework_objects, &path)?;
            } else {
                debug!("Processing file: {:?}", path);
                let mut objects = get_framework_objects_from_schema_file(&path)?;
                framework_objects.append(&mut objects)
            }
        }
    }
    Ok(framework_objects.to_vec())
}

pub fn get_framework_objects_from_schema_file(path: &Path) -> Result<Vec<FrameworkObject>, Error> {
    let framework_objects = parse_schema_file::<FrameworkObject>(path, framework_object_mapper)
        .map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("Failed to parse schema file. Error {}", e),
            )
        })?;
    Ok(framework_objects)
}

pub(crate) async fn create_or_replace_kafka_trigger(
    fo: &FrameworkObject,
    view_name: String,
    configured_client: &ConfiguredDBClient,
) -> Result<(), Error> {
    let view = ClickhouseKafkaTrigger::new(
        fo.table.db_name.clone(),
        view_name,
        fo.table.kafka_table_name(),
        fo.table.name.clone(),
    );
    let create_view_query = view.create_materialized_view_query().map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("Failed to get clickhouse query: {:?}", e),
        )
    })?;

    // Clickhouse doesn't support dropping a view if it doesn't exist so we need to drop it first in case the schema has changed
    let drop_view_query = view.drop_materialized_view_query().map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("Failed to get clickhouse query: {:?}", e),
        )
    })?;
    olap::clickhouse::run_query(&drop_view_query, configured_client)
        .await
        .map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("Failed to drop view in clickhouse: {}", e),
            )
        })?;
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

pub(crate) async fn create_or_replace_tables(
    project_name: &str,
    fo: &FrameworkObject,
    configured_client: &ConfiguredDBClient,
) -> anyhow::Result<()> {
    info!("Creating table: {:?}", fo.table.name);

    let drop_data_table_query = fo.table.drop_kafka_table_query()?;
    let drop_kafka_table_query = fo.table.drop_data_table_query()?;

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

    // Clickhouse doesn't support dropping a view if it doesn't exist so we need to drop it first in case the schema has changed
    olap::clickhouse::run_query(&drop_data_table_query, configured_client).await?;
    olap::clickhouse::run_query(&drop_kafka_table_query, configured_client).await?;
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
        project.http_server_config().url(),
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
    project_name: &str,
    schema_file_path: &Path,
    route_table: &mut HashMap<PathBuf, RouteMeta>,
    configured_client: &ConfiguredDBClient,
) -> anyhow::Result<()> {
    //need to get the path of the file, scan the route table and remove all the files that need to be deleted.
    // This doesn't have to be as fast as the scanning for routes in the web server so we're ok with the scan here.

    for (k, meta) in route_table.clone().into_iter() {
        if meta.original_file_path == schema_file_path {
            stream::redpanda::delete_topic(project_name, meta.table_name.clone())?;

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

fn schema_file_path_to_ingest_route(app_dir: PathBuf, path: &Path, table_name: String) -> PathBuf {
    let data_model_path = app_dir.join(SCHEMAS_DIR);
    debug!("got data model path: {:?}", data_model_path);
    debug!("processing schema file into route: {:?}", path);
    let mut route = path.strip_prefix(data_model_path).unwrap().to_path_buf();

    route.set_file_name(table_name);

    debug!("route: {:?}", route);

    PathBuf::from("ingest").join(route)
}

pub async fn process_objects(
    framework_objects: Vec<FrameworkObject>,
    project: Arc<Project>,
    schema_file_path: &Path,
    configured_client: &ConfiguredDBClient,
    compilable_objects: &mut Vec<TypescriptObjects>, // Objects that require compilation after processing
    route_table: &mut HashMap<PathBuf, RouteMeta>,
) -> anyhow::Result<()> {
    let app_dir = project.clone().app_dir();
    for fo in framework_objects {
        let ingest_route = schema_file_path_to_ingest_route(
            app_dir.clone(),
            schema_file_path,
            fo.table.name.clone(),
        );
        stream::redpanda::create_topic_from_name(project.name(), fo.topic.clone())?;

        debug!("Creating table & view: {:?}", fo.table.name);

        let view_name = format!("{}_trigger", fo.table.name);

        create_or_replace_tables(project.name(), &fo, configured_client).await?;
        create_or_replace_kafka_trigger(&fo, view_name.clone(), configured_client).await?;

        debug!("Table created: {:?}", fo.table.name);

        let typescript_objects = create_language_objects(&fo, &ingest_route, project.clone())?;
        compilable_objects.push(typescript_objects);

        route_table.insert(
            ingest_route,
            RouteMeta {
                original_file_path: schema_file_path.to_path_buf(),
                table_name: fo.table.name.clone(),
                view_name: Some(view_name),
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
        let mut framework_objects = vec![];
        let result = get_all_framework_objects(&mut framework_objects, &schema_dir);
        assert!(result.is_ok());
        assert!(framework_objects.len() == 2);
    }
}

use crate::framework::languages::CodeGenerator;
use crate::infrastructure::olap::clickhouse::ClickhouseTable;
use crate::infrastructure::stream;

use std::collections::HashMap;
use std::path::Path;

use crate::framework::languages::SupportedLanguages;

use crate::framework;

use log::debug;
use log::info;

use crate::framework::typescript::get_typescript_models_dir;

use crate::framework::typescript::SendFunction;

use crate::framework::sdks::TypescriptObjects;

use crate::project::Project;

use std::path::PathBuf;

use crate::infrastructure::olap;

use std::io::ErrorKind;

use crate::infrastructure::olap::clickhouse::ClickhouseView;

use std::io::Error;

use crate::infrastructure::olap::clickhouse::ConfiguredDBClient;

use super::schema::parse_schema_file;
use super::schema::MatViewOps;
use super::schema::Table;
use super::schema::TableOps;
use super::typescript::TypescriptInterface;

pub struct FrameworkObject {
    pub table: ClickhouseTable,
    pub topic: String,
    pub ts_interface: TypescriptInterface,
}

pub fn framework_object_mapper(t: Table) -> FrameworkObject {
    let clickhouse_table = olap::clickhouse::mapper::std_table_to_clickhouse_table(t.clone());
    FrameworkObject {
        table: clickhouse_table.clone(),
        topic: t.name.clone(),
        ts_interface: framework::typescript::mapper::std_table_to_typescript_interface(t),
    }
}

#[derive(Debug, Clone)]
pub struct RouteMeta {
    pub original_file_path: PathBuf,
    pub table_name: String,
    pub view_name: Option<String>,
}

pub fn get_framework_objects(route: &Path) -> Result<Vec<FrameworkObject>, Error> {
    let framework_objects = parse_schema_file::<FrameworkObject>(route, framework_object_mapper)
        .map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("Failed to parse schema file. Error {}", e),
            )
        })?;
    Ok(framework_objects)
}

pub(crate) async fn create_or_replace_view(
    fo: &FrameworkObject,
    view_name: String,
    configured_client: &ConfiguredDBClient,
) -> Result<(), Error> {
    let view = ClickhouseView::new(fo.table.db_name.clone(), view_name, fo.table.clone());
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

pub(crate) async fn create_or_replace_table(
    fo: &FrameworkObject,
    configured_client: &ConfiguredDBClient,
) -> Result<(), Error> {
    info!("Creating table: {:?}", fo.table.name);
    let create_table_query = fo.table.create_table_query().map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("Failed to get clickhouse query: {:?}", e),
        )
    })?;
    let drop_table_query = fo.table.drop_table_query().map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("Failed to get clickhouse query: {:?}", e),
        )
    })?;

    olap::clickhouse::check_ready(configured_client)
        .await
        .map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("Failed to connect to clickhouse: {}", e),
            )
        })?;

    // Clickhouse doesn't support dropping a view if it doesn't exist so we need to drop it first in case the schema has changed
    olap::clickhouse::run_query(&drop_table_query, configured_client)
        .await
        .map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("Failed to drop table in clickhouse: {}", e),
            )
        })?;
    olap::clickhouse::run_query(&create_table_query, configured_client)
        .await
        .map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("Failed to create table in clickhouse: {}", e),
            )
        })?;
    Ok(())
}

pub(crate) fn create_language_objects(
    fo: &FrameworkObject,
    ingest_route: &Path,
    project: &Project,
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
        project.local_webserver_config.url(),
        ingest_route.to_str().unwrap().to_string(),
    );
    let send_func_code = send_func.create_code().map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("Failed to generate send function: {:?}", e),
        )
    })?;
    let typescript_dir = get_typescript_models_dir(project.clone())?;
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
    shcema_file_path: &Path,
    route_table: &mut HashMap<PathBuf, RouteMeta>,
    configured_client: &ConfiguredDBClient,
) -> Result<(), Error> {
    //need to get the path of the file, scan the route table and remove all the files that need to be deleted.
    // This doesn't have to be as fast as the scanning for routes in the web server so we're ok with the scan here.

    for (k, meta) in route_table.clone().into_iter() {
        if meta.original_file_path == shcema_file_path {
            stream::redpanda::delete_topic(meta.table_name.clone())?;

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

            (&mut *route_table).remove(&k);
        }
    }
    Ok(())
}

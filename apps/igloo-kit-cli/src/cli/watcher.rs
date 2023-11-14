use std::{
    collections::HashMap,
    io::{Error, ErrorKind},
    path::PathBuf,
    sync::{Arc, RwLock},
};

use notify::{event::ModifyKind, Config, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::Mutex;

use crate::{
    cli::display::show_message,
    framework::{
        self,
        languages::{CodeGenerator, SupportedLanguages},
        schema::{parse_schema_file, MatViewOps, Table, TableOps},
        sdks::{generate_ts_sdk, TypescriptObjects, move_to_npm_global_dir},
        typescript::{get_typescript_models_dir, SendFunction, TypescriptInterface},
    },
    infrastructure::{
        olap::{
            self,
            clickhouse::{
                config::ClickhouseConfig, ClickhouseTable, ClickhouseView, ConfiguredDBClient,
            },
        },
        stream,
    },
    project::Project, utilities::package_managers,
};

use super::{
    display::{Message, MessageType},
    local_webserver::Webserver,
    CommandTerminal,
};

fn dataframe_path_to_ingest_route(app_dir: PathBuf, path: PathBuf, table_name: String) -> PathBuf {
    let dataframe_path = app_dir.join("dataframes");
    println!("dataframe path: {:?}", dataframe_path);
    println!("path: {:?}", path);
    let mut route = path.strip_prefix(dataframe_path).unwrap().to_path_buf();

    route.set_file_name(table_name);

    PathBuf::from("ingest").join(route)
}

async fn process_event(
    web_server: Webserver,
    project: Project,
    event: notify::Event,
    route_table: Arc<Mutex<HashMap<PathBuf, RouteMeta>>>,
    configured_client: &ConfiguredDBClient,
) -> Result<(), Error> {
    let route = event.paths[0].clone();
    let mut route_table = route_table.lock().await;

    match event.kind {
        notify::EventKind::Create(_) => {
            // Only create tables and topics from prisma files in the dataframes directory
            create_framework_objects_from_dataframe_route(
                project,
                web_server,
                &route,
                &mut route_table,
                configured_client,
            )
            .await
        }
        notify::EventKind::Modify(mk) => {
            match mk {
                ModifyKind::Name(_) => {
                    // remove the file from the routes if they don't exist in the file directory
                    if route.exists() {
                        create_framework_objects_from_dataframe_route(
                            project,
                            web_server,
                            &route,
                            &mut route_table,
                            configured_client,
                        )
                        .await
                    } else {
                        remove_table_and_topics_from_dataframe_route(
                            &route,
                            &mut route_table,
                            configured_client,
                        )
                        .await
                    }
                }
                _ => Ok(()),
            }
        }
        notify::EventKind::Remove(_) => Ok(()),
        _ => Ok(()),
    }
}

#[derive(Debug, Clone)]
pub struct RouteMeta {
    pub original_file_path: PathBuf,
    pub table_name: String,
    pub view_name: Option<String>,
}

struct FrameworkObject {
    pub table: ClickhouseTable,
    pub topic: String,
    pub ts_interface: TypescriptInterface,
}

fn framework_object_mapper(t: Table) -> FrameworkObject {
    let clickhouse_table = olap::clickhouse::mapper::std_table_to_clickhouse_table(t.clone());
    FrameworkObject {
        table: clickhouse_table.clone(),
        topic: t.name.clone(),
        ts_interface: framework::typescript::mapper::std_table_to_typescript_interface(t),
    }
}

async fn create_framework_objects_from_dataframe_route(
    project: Project,
    web_server: Webserver,
    route: &PathBuf,
    route_table: &mut tokio::sync::MutexGuard<'_, HashMap<PathBuf, RouteMeta>>,
    configured_client: &ConfiguredDBClient,
) -> Result<(), Error> {
    if let Some(ext) = route.extension() {
        if ext == "prisma" && route.as_path().to_str().unwrap().contains("dataframes") {
            let framework_objects =
                parse_schema_file::<FrameworkObject>(route.clone(), framework_object_mapper)
                    .map_err(|e| {
                        Error::new(
                            ErrorKind::Other,
                            format!("Failed to parse schema file. Error {}", e),
                        )
                    })?;

            let mut process_further: Vec<TypescriptObjects> = Vec::new();

            for fo in framework_objects {
                let ingest_route = dataframe_path_to_ingest_route(
                    project.app_folder.clone(),
                    route.clone(),
                    fo.table.name.clone(),
                );
                stream::redpanda::create_topic_from_name(fo.topic.clone());
                let view_name = format!("{}_view", fo.table.name);
                create_db_objects(&fo, configured_client, view_name.clone()).await?;
                let typescript_objects =
                    create_language_objects(&fo, &web_server, &ingest_route, &project)?;
                process_further.push(typescript_objects);

                route_table.insert(
                    ingest_route,
                    RouteMeta {
                        original_file_path: route.clone(),
                        table_name: fo.table.name.clone(),
                        view_name: Some(view_name),
                    },
                );
            }

            let sdk_location = generate_ts_sdk(&project, process_further)?;
            let package_manager = package_managers::PackageManager::Pnpm;
            package_managers::install_packages(&sdk_location, &package_manager)?;
            package_managers::run_build(&sdk_location, &package_manager)?;
            package_managers::link_sdk(&sdk_location, None, &package_manager)?;
        }
    } else {
        println!("No primsa extension found. Likely created unsupported file type")
    }
    Ok(())
}

async fn create_db_objects(
    fo: &FrameworkObject,
    configured_client: &ConfiguredDBClient,
    view_name: String,
) -> Result<(), Error> {
    let table_query = fo.table.create_table_query().map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("Failed to get clickhouse query: {:?}", e),
        )
    })?;
    olap::clickhouse::run_query(table_query, configured_client)
        .await
        .map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("Failed to create table in clickhouse: {}", e),
            )
        })?;
    let view = ClickhouseView::new(fo.table.db_name.clone(), view_name, fo.table.clone());
    let view_query = view.create_materialized_view_query().map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("Failed to get clickhouse query: {:?}", e),
        )
    })?;
    olap::clickhouse::run_query(view_query, configured_client)
        .await
        .map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("Failed to create view in clickhouse: {}", e),
            )
        })?;
    Ok(())
}

fn create_language_objects(
    fo: &FrameworkObject,
    web_server: &Webserver,
    ingest_route: &PathBuf,
    project: &Project,
) -> Result<TypescriptObjects, Error> {
    let ts_interface_code = fo.ts_interface.create_code().map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("Failed to get typescript interface: {:?}", e),
        )
    })?;
    let send_func = SendFunction::new(
        fo.ts_interface.clone(),
        web_server.url(),
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

async fn remove_table_and_topics_from_dataframe_route(
    route: &PathBuf,
    route_table: &mut tokio::sync::MutexGuard<'_, HashMap<PathBuf, RouteMeta>>,
    configured_client: &ConfiguredDBClient,
) -> Result<(), Error> {
    //need to get the path of the file, scan the route table and remove all the files that need to be deleted.
    // This doesn't have to be as fast as the scanning for routes in the web server so we're ok with the scan here.
    for (k, meta) in route_table.clone().into_iter() {
        if meta.original_file_path == route.clone() {
            stream::redpanda::delete_topic(meta.table_name.clone());

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

            route_table.remove(&k);
        }
    }
    Ok(())
}

async fn watch(
    web_server: Webserver,
    project: Project,
    route_table: Arc<Mutex<HashMap<PathBuf, RouteMeta>>>,
    configured_client: &ConfiguredDBClient,
) -> Result<(), Error> {
    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher = RecommendedWatcher::new(tx, Config::default()).map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("Failed to create file watcher: {}", e),
        )
    })?;

    watcher
        .watch(project.app_folder.as_ref(), RecursiveMode::Recursive)
        .map_err(|e| Error::new(ErrorKind::Other, format!("Failed to watch file: {}", e)))?;

    for res in rx {
        match res {
            Ok(event) => {
                process_event(
                    web_server.clone(),
                    project.clone(),
                    event.clone(),
                    Arc::clone(&route_table),
                    configured_client,
                )
                .await
                .map_err(|e| {
                    Error::new(
                        ErrorKind::Other,
                        format!("clickhouse error has occured: {}", e),
                    )
                })?;
            }
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("File watcher event caused a failure: {}", error),
                ))
            }
        }
        println!("{:?}", route_table)
    }
    Ok(())
}

pub struct FileWatcher;

impl FileWatcher {
    pub fn new() -> Self {
        Self {}
    }

    pub fn start(
        &self,
        project: Project,
        web_server: Webserver,
        term: Arc<RwLock<CommandTerminal>>,
        route_table: Arc<Mutex<HashMap<PathBuf, RouteMeta>>>,
        clickhouse_config: ClickhouseConfig,
    ) -> Result<(), Error> {
        show_message(term, MessageType::Info, {
            Message {
                action: "Watching".to_string(),
                details: format!("{:?}", project.app_folder.display()),
            }
        });

        tokio::spawn(async move {
            // Need to spin up client in thread to ensure it lives long enough
            let db_client = olap::clickhouse::create_client(clickhouse_config.clone());

            if let Err(error) =
                watch(web_server, project, Arc::clone(&route_table), &db_client).await
            {
                println!("Error: {error:?}");
            }
        });

        Ok(())
    }
}

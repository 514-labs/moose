use std::{
    collections::HashMap,
    io::{Error, ErrorKind},
    path::PathBuf,
    sync::{Arc, RwLock},
};

use notify::{event::ModifyKind, Config, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::Mutex;

use crate::{
    constants::SCHEMAS_DIR,
    framework::{
        controller::{
            create_language_objects, create_or_replace_table, create_or_replace_view,
            get_framework_objects, remove_table_and_topics_from_schema_file_path, FrameworkObject,
            RouteMeta,
        },
        sdks::{generate_ts_sdk, TypescriptObjects},
    },
    infrastructure::{
        olap::{self, clickhouse::ConfiguredDBClient},
        stream,
    },
    project::Project,
    utilities::package_managers,
};

use super::{
    display::{Message, MessageType},
    CommandTerminal,
};
use log::{debug, info};

fn schema_file_path_to_ingest_route(
    app_dir: PathBuf,
    path: PathBuf,
    table_name: String,
) -> PathBuf {
    let dataframe_path = app_dir.join(SCHEMAS_DIR);
    println!("dataframe path: {:?}", dataframe_path);
    println!("path: {:?}", path);
    let mut route = path.strip_prefix(dataframe_path).unwrap().to_path_buf();

    route.set_file_name(table_name);

    PathBuf::from("ingest").join(route)
}

async fn process_event(
    project: Project,
    event: notify::Event,
    route_table: Arc<Mutex<HashMap<PathBuf, RouteMeta>>>,
    configured_client: &ConfiguredDBClient,
) -> Result<(), Error> {
    debug!(
        "File Watcher Event Received: {:?}, with Route Table {:?}",
        event, route_table
    );

    let route = event.paths[0].clone();

    match event.kind {
        notify::EventKind::Create(_) => {
            // Only create tables and topics from prisma files in the dataframes directory
            create_framework_objects_from_schema_file_path(
                &project,
                &route,
                route_table,
                configured_client,
            )
            .await
        }
        notify::EventKind::Modify(mk) => {
            match mk {
                ModifyKind::Name(_) => {
                    // remove the file from the routes if they don't exist in the file directory
                    if route.exists() {
                        create_framework_objects_from_schema_file_path(
                            &project,
                            &route,
                            route_table,
                            configured_client,
                        )
                        .await
                    } else {
                        remove_table_and_topics_from_schema_file_path(
                            &route,
                            route_table,
                            configured_client,
                        )
                        .await
                    }
                }

                ModifyKind::Data(_) => {
                    if route.exists() {
                        create_framework_objects_from_schema_file_path(
                            &project,
                            &route,
                            route_table,
                            configured_client,
                        )
                        .await?
                    }
                    Ok(())
                }
                _ => Ok(()),
            }
        }
        notify::EventKind::Remove(_) => Ok(()),
        _ => Ok(()),
    }
}

async fn create_framework_objects_from_schema_file_path(
    project: &Project,
    schema_file_path: &PathBuf,
    route_table: Arc<Mutex<HashMap<PathBuf, RouteMeta>>>,
    configured_client: &ConfiguredDBClient,
) -> Result<(), Error> {
    //! Creates the route, topics and tables from a path to the schema file

    if let Some(ext) = schema_file_path.extension() {
        if ext == "prisma"
            && schema_file_path
                .as_path()
                .to_str()
                .unwrap()
                .contains(SCHEMAS_DIR)
        {
            process_schema_file(schema_file_path, project, configured_client, route_table).await?;
        }
    } else {
        info!("No primsa extension found. Likely created unsupported file type")
    }
    Ok(())
}

pub async fn process_schema_file(
    schema_file_path: &PathBuf,
    project: &Project,
    configured_client: &ConfiguredDBClient,
    route_table: Arc<Mutex<HashMap<PathBuf, RouteMeta>>>,
) -> Result<(), Error> {
    let framework_objects = get_framework_objects(schema_file_path)?;
    let mut compilable_objects: Vec<TypescriptObjects> = Vec::new();
    process_objects(
        framework_objects,
        project,
        schema_file_path,
        configured_client,
        &mut compilable_objects,
        route_table,
    )
    .await?;
    debug!("All objects created, generating sdk...");
    let sdk_location = generate_ts_sdk(project, compilable_objects)?;
    let package_manager = package_managers::PackageManager::Npm;
    package_managers::install_packages(&sdk_location, &package_manager)?;
    package_managers::run_build(&sdk_location, &package_manager)?;
    package_managers::link_sdk(&sdk_location, None, &package_manager)?;
    Ok(())
}

async fn process_objects(
    framework_objects: Vec<FrameworkObject>,
    project: &Project,
    schema_file_path: &PathBuf,
    configured_client: &ConfiguredDBClient,
    compilable_objects: &mut Vec<TypescriptObjects>, // Objects that require compilation after processing
    route_table: Arc<Mutex<HashMap<PathBuf, RouteMeta>>>,
) -> Result<(), Error> {
    let mut route_table = route_table.lock().await;

    for fo in framework_objects {
        let ingest_route = schema_file_path_to_ingest_route(
            project.app_dir().clone(),
            schema_file_path.clone(),
            fo.table.name.clone(),
        );
        stream::redpanda::create_topic_from_name(fo.topic.clone());

        debug!("Creating table & view: {:?}", fo.table.name);

        let view_name = format!("{}_view", fo.table.name);
        create_or_replace_table(&fo, configured_client).await?;
        create_or_replace_view(&fo, view_name.clone(), configured_client).await?;

        debug!("Table created: {:?}", fo.table.name);

        let typescript_objects = create_language_objects(&fo, &ingest_route, project)?;
        compilable_objects.push(typescript_objects);

        route_table.insert(
            ingest_route,
            RouteMeta {
                original_file_path: schema_file_path.clone(),
                table_name: fo.table.name.clone(),
                view_name: Some(view_name),
            },
        );
    }
    Ok(())
}

async fn watch(
    project: &Project,
    route_table: Arc<Mutex<HashMap<PathBuf, RouteMeta>>>,
) -> Result<(), Error> {
    let configured_client = olap::clickhouse::create_client(project.clickhouse_config.clone());

    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher = RecommendedWatcher::new(tx, Config::default()).map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("Failed to create file watcher: {}", e),
        )
    })?;

    watcher
        .watch(project.app_dir().as_ref(), RecursiveMode::Recursive)
        .map_err(|e| Error::new(ErrorKind::Other, format!("Failed to watch file: {}", e)))?;

    for res in rx {
        match res {
            Ok(event) => {
                process_event(
                    project.clone(),
                    event.clone(),
                    Arc::clone(&route_table),
                    &configured_client,
                )
                .await
                .map_err(|e| {
                    Error::new(ErrorKind::Other, format!("Processing error occured: {}", e))
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
        project: &Project,
        term: Arc<RwLock<CommandTerminal>>,
        route_table: Arc<Mutex<HashMap<PathBuf, RouteMeta>>>,
    ) -> Result<(), Error> {
        show_message!(MessageType::Info, {
            Message {
                action: "Watching".to_string(),
                details: format!("{:?}", project.app_dir().display()),
            }
        });
        let project = project.clone();

        tokio::spawn(async move {
            if let Err(error) = watch(&project, Arc::clone(&route_table)).await {
                println!("Error: {error:?}");
            }
        });

        Ok(())
    }
}

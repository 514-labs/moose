use std::{
    collections::HashMap,
    io::{Error, ErrorKind},
    path::{Path, PathBuf},
};

use notify::{event::ModifyKind, Config, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::RwLock;

use super::display::{with_spinner_async, Message, MessageType};
use crate::infrastructure::stream::redpanda;
use crate::{
    framework::controller::{remove_table_and_topics_from_schema_file_path, RouteMeta},
    infrastructure::olap::{self, clickhouse::ConfiguredDBClient},
    project::Project,
    utilities::constants::SCHEMAS_DIR,
};
use crate::{
    framework::schema::process_schema_file, infrastructure::console::post_current_state_to_console,
};
use log::{debug, info};

async fn process_event(
    project: Project,
    event: notify::Event,
    route_table: &RwLock<HashMap<PathBuf, RouteMeta>>,
    configured_client: &ConfiguredDBClient,
) -> Result<(), Error> {
    let mut route_table = route_table.write().await;

    debug!(
        "File Watcher Event Received: {:?}, with Route Table {:?}",
        event, route_table
    );

    let route = event.paths[0].clone();

    match event.kind {
        notify::EventKind::Create(_) => {
            // Only create tables and topics from prisma files in the datamodels directory
            create_framework_objects_from_schema_file_path(
                &project,
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
                        create_framework_objects_from_schema_file_path(
                            &project,
                            &route,
                            &mut route_table,
                            configured_client,
                        )
                        .await
                    } else {
                        remove_table_and_topics_from_schema_file_path(
                            &route,
                            &mut route_table,
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
                            &mut route_table,
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
    schema_file_path: &Path,
    route_table: &mut HashMap<PathBuf, RouteMeta>,
    configured_client: &ConfiguredDBClient,
) -> Result<(), Error> {
    //! Creates the route, topics and tables from a path to the schema file

    if let Some(ext) = schema_file_path.extension() {
        if ext == "prisma" && schema_file_path.to_str().unwrap().contains(SCHEMAS_DIR) {
            with_spinner_async("Processing schema file", async {
                let result =
                    process_schema_file(schema_file_path, project, configured_client, route_table)
                        .await;
                match result {
                    Ok(_) => {
                        show_message!(MessageType::Info, {
                            Message {
                                action: "Schema".to_string(),
                                details: "file processed".to_string(),
                            }
                        });
                    }
                    Err(e) => {
                        show_message!(MessageType::Error, {
                            Message {
                                action: "Schema".to_string(),
                                details: format!("file failed to process: {}", e),
                            }
                        });
                    }
                }
            })
            .await
        }
    } else {
        info!("No primsa extension found. Likely created unsupported file type")
    }
    Ok(())
}

async fn watch(
    project: &Project,
    route_table: &RwLock<HashMap<PathBuf, RouteMeta>>,
) -> Result<(), Error> {
    let configured_client = olap::clickhouse::create_client(project.clickhouse_config.clone());
    let configured_producer = redpanda::create_producer(project.redpanda_config.clone());

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
                    route_table,
                    &configured_client,
                )
                .await
                .map_err(|e| {
                    Error::new(ErrorKind::Other, format!("Processing error occured: {}", e))
                })?;

                let route_table_snapshot = {
                    let read_lock = route_table.read().await;
                    (*read_lock).clone()
                };

                let _ = post_current_state_to_console(
                    project,
                    &configured_client,
                    &configured_producer,
                    route_table_snapshot,
                    project.console_config.clone(),
                )
                .await;
            }
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("File watcher event caused a failure: {}", error),
                ))
            }
        }
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
        route_table: &'static RwLock<HashMap<PathBuf, RouteMeta>>,
    ) -> Result<(), Error> {
        show_message!(MessageType::Info, {
            Message {
                action: "Watching".to_string(),
                details: format!("{:?}", project.app_dir().display()),
            }
        });
        let project = project.clone();

        tokio::spawn(async move {
            if let Err(error) = watch(&project, route_table).await {
                debug!("Error: {error:?}");
            }
        });

        Ok(())
    }
}

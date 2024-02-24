use std::collections::HashSet;
use std::sync::mpsc::TryRecvError;
use std::sync::Arc;
use std::{
    collections::HashMap,
    io::{Error, ErrorKind},
    path::PathBuf,
};

use log::{debug, warn};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::RwLock;

use crate::framework::controller::{
    create_or_replace_kafka_trigger, create_or_replace_tables, drop_kafka_trigger, drop_tables,
    get_framework_objects_from_schema_file, schema_file_path_to_ingest_route,
    FrameworkObjectVersions,
};
use crate::framework::schema::{is_prisma_file, DuplicateModelError};
use crate::infrastructure::console::post_current_state_to_console;
use crate::infrastructure::olap::clickhouse::ClickhouseKafkaTrigger;
use crate::infrastructure::stream::redpanda;
use crate::{
    framework::controller::RouteMeta,
    infrastructure::olap::{self, clickhouse::ConfiguredDBClient},
    project::Project,
};

use super::display::{with_spinner_async, Message, MessageType};

async fn process_events(
    project: Arc<Project>,
    events: Vec<notify::Event>,
    framework_object_versions: &mut FrameworkObjectVersions,
    route_table: &RwLock<HashMap<PathBuf, RouteMeta>>,
    configured_client: &ConfiguredDBClient,
) -> anyhow::Result<()> {
    debug!(
        "File Watcher Event Received: {:?}, with Route Table {:?}",
        events, route_table
    );

    let paths = events
        .into_iter()
        .flat_map(|e| e.paths)
        .filter(|p| is_prisma_file(p))
        .collect::<HashSet<PathBuf>>();

    let mut new_objects = HashMap::new();
    let mut deleted_objects = HashMap::new();
    let mut changed_objects = HashMap::new();
    let mut moved_objects = HashMap::new();

    let mut extra_models = HashMap::new();

    let old_objects = &framework_object_versions.current_models.models;

    for path in paths {
        // This is O(mn) but m and n are both small, so it should be fine
        let mut removed_old_objects_in_file = old_objects
            .iter()
            .filter(|(_, fo)| fo.original_file_path == path)
            .collect::<HashMap<_, _>>();

        if path.exists() {
            let obj_in_new_file =
                get_framework_objects_from_schema_file(&path, &project.version())?;
            for obj in obj_in_new_file {
                removed_old_objects_in_file.remove(&obj.data_model.name);

                match old_objects.get(&obj.data_model.name) {
                    Some(changed) if changed.data_model != obj.data_model => {
                        if changed.original_file_path != obj.original_file_path
                            && deleted_objects.remove(&obj.data_model.name).is_none()
                        {
                            DuplicateModelError::try_insert(&mut extra_models, obj.clone(), &path)?;
                        };
                        DuplicateModelError::try_insert(&mut changed_objects, obj, &path)?;
                    }
                    Some(moved) if moved.original_file_path != obj.original_file_path => {
                        if deleted_objects.remove(&obj.data_model.name).is_none() {
                            DuplicateModelError::try_insert(&mut extra_models, obj.clone(), &path)?;
                        };
                        DuplicateModelError::try_insert(&mut moved_objects, obj, &path)?;
                    }
                    Some(_unchanged) => {
                        debug!("No changes to object: {:?}", obj.data_model.name);
                    }
                    None => {
                        DuplicateModelError::try_insert(&mut new_objects, obj, &path)?;
                    }
                }
            }
        }

        removed_old_objects_in_file
            .into_iter()
            .for_each(|(name, old)| {
                debug!("Found {name:?} removed from file {path:?}");
                if let Some(moved) = new_objects.remove(&old.data_model.name) {
                    changed_objects.insert(old.data_model.name.clone(), moved);
                } else if !changed_objects.contains_key(&old.data_model.name)
                    && !moved_objects.contains_key(&old.data_model.name)
                {
                    deleted_objects.insert(old.data_model.name.clone(), old.clone());
                } else {
                    extra_models.remove(name.as_str());
                }
            });
    }

    if let Some((model_name, extra)) = extra_models.into_iter().next() {
        let other_file_path = old_objects
            .get(&model_name)
            .unwrap()
            .original_file_path
            .clone();
        return Err(DuplicateModelError {
            model_name,
            file_path: extra.original_file_path,
            other_file_path,
        }
        .into());
    }

    debug!("new_objects = {new_objects:?}\ndeleted_objects = {deleted_objects:?}\nchanged_objects = {changed_objects:?}\nmoved_objects = {moved_objects:?}");

    // grab the lock to prevent HTTP requests from being processed while we update the route table
    let mut route_table = route_table.write().await;
    for (_, fo) in deleted_objects {
        drop_tables(&fo, configured_client).await?;
        drop_kafka_trigger(
            &ClickhouseKafkaTrigger::from_clickhouse_table(&fo.table),
            configured_client,
        )
        .await?;
        route_table.remove(&schema_file_path_to_ingest_route(
            &framework_object_versions.current_models.base_path,
            &fo.original_file_path,
            fo.data_model.name.clone(),
            &framework_object_versions.current_version,
        ));
        redpanda::delete_topic(&project.name(), &fo.data_model.name)?;

        framework_object_versions
            .current_models
            .models
            .remove(&fo.data_model.name);
    }
    for (_, fo) in changed_objects.into_iter().chain(new_objects) {
        create_or_replace_tables(&project.name(), &fo, configured_client).await?;
        let view = ClickhouseKafkaTrigger::from_clickhouse_table(&fo.table);
        create_or_replace_kafka_trigger(&view, configured_client).await?;
        route_table.insert(
            schema_file_path_to_ingest_route(
                &framework_object_versions.current_models.base_path,
                &fo.original_file_path,
                fo.data_model.name.clone(),
                &framework_object_versions.current_version,
            ),
            RouteMeta {
                original_file_path: fo.original_file_path.clone(),
                table_name: fo.table.name.clone(),
                view_name: Some(view.name),
            },
        );
        redpanda::create_topic_from_name(&project.name(), fo.topic.clone())?;

        framework_object_versions
            .current_models
            .models
            .insert(fo.data_model.name.clone(), fo);
    }

    for (_, fo) in moved_objects {
        framework_object_versions
            .current_models
            .models
            .insert(fo.data_model.name.clone(), fo);
    }

    Ok(())
}

async fn watch(
    project: Arc<Project>,
    framework_object_versions: &mut FrameworkObjectVersions,
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

    loop {
        let res = rx.recv().unwrap();
        match res {
            Ok(event) => {
                let mut events = vec![event];
                loop {
                    match rx.try_recv() {
                        Ok(Ok(event)) => events.push(event),
                        Ok(Err(e)) => {
                            warn!("File watcher event caused a failure: {}", e);
                            break;
                        }
                        Err(TryRecvError::Empty) => break,
                        Err(TryRecvError::Disconnected) => {
                            warn!("File watcher channel disconnected");
                            break;
                        }
                    }
                }

                with_spinner_async(
                    &format!("Processing {} events from file watcher", events.len()),
                    process_events(
                        project.clone(),
                        events,
                        framework_object_versions,
                        route_table,
                        &configured_client,
                    ),
                )
                .await
                .map_err(|e| {
                    Error::new(
                        ErrorKind::Other,
                        format!("Processing error occurred: {}", e),
                    )
                })?;

                let _ = post_current_state_to_console(
                    project.clone(),
                    &configured_client,
                    &configured_producer,
                    framework_object_versions,
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
}

pub struct FileWatcher;

impl FileWatcher {
    pub fn new() -> Self {
        Self {}
    }

    pub fn start(
        &self,
        project: Arc<Project>,
        framework_object_versions: FrameworkObjectVersions,
        route_table: &'static RwLock<HashMap<PathBuf, RouteMeta>>,
    ) -> Result<(), Error> {
        show_message!(MessageType::Info, {
            Message {
                action: "Watching".to_string(),
                details: format!("{:?}", project.app_dir().display()),
            }
        });

        let mut framework_object_versions = framework_object_versions;

        tokio::spawn(async move {
            if let Err(error) = watch(project, &mut framework_object_versions, route_table).await {
                panic!("Watcher error: {error:?}");
            }
        });

        Ok(())
    }
}

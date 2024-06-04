use std::collections::HashSet;
use std::sync::mpsc::TryRecvError;
use std::sync::Arc;
use std::{
    collections::HashMap,
    io::{Error, ErrorKind},
    path::PathBuf,
};

use anyhow::bail;
use log::{debug, info, warn};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::RwLock;

use crate::framework::consumption::model::Consumption;
use crate::framework::consumption::registry::ConsumptionProcessRegistry;

use crate::framework::aggregations::model::Aggregation;
use crate::framework::aggregations::registry::AggregationProcessRegistry;
use crate::framework::controller::{
    create_or_replace_tables, drop_table, get_framework_objects_from_schema_file,
    schema_file_path_to_ingest_route, FrameworkObjectVersions,
};
use crate::framework::data_model::{is_schema_file, DuplicateModelError};
use crate::framework::flows::loader::get_all_current_flows;
use crate::framework::flows::registry::FlowProcessRegistry;
use crate::framework::typescript;
use crate::infrastructure::console::post_current_state_to_console;
use crate::infrastructure::kafka_clickhouse_sync::SyncingProcessesRegistry;
use crate::infrastructure::stream::redpanda;
use crate::project::AggregationSet;
use crate::utilities::constants::{AGGREGATIONS_DIR, CONSUMPTION_DIR, FLOWS_DIR, SCHEMAS_DIR};
use crate::utilities::package_managers;
use crate::{
    framework::controller::RouteMeta,
    infrastructure::olap::{self, clickhouse::ConfiguredDBClient},
    project::Project,
};

use super::display::{with_spinner_async, Message, MessageType};
use super::routines::flow::verify_flows_against_datamodels;

async fn process_data_models_changes(
    project: Arc<Project>,
    events: Vec<notify::Event>,
    framework_object_versions: &mut FrameworkObjectVersions,
    route_table: &RwLock<HashMap<PathBuf, RouteMeta>>,
    configured_client: &ConfiguredDBClient,
    syncing_process_registry: &mut SyncingProcessesRegistry,
) -> anyhow::Result<()> {
    debug!(
        "File Watcher Event Received: {:?}, with Route Table {:?}",
        events, route_table
    );

    let schema_files = project.schemas_dir();

    let paths = events
        .into_iter()
        .flat_map(|e| e.paths)
        .filter(|p| is_schema_file(p) && p.starts_with(&schema_files))
        .collect::<HashSet<PathBuf>>();

    let mut new_objects = HashMap::new();
    let mut deleted_objects = HashMap::new();
    let mut changed_objects: HashMap<String, crate::framework::controller::FrameworkObject> =
        HashMap::new();
    let mut moved_objects = HashMap::new();

    let mut extra_models = HashMap::new();

    let old_objects = &framework_object_versions.current_models.models;

    let aggregations = AggregationSet {
        current_version: project.version().to_owned(),
        names: project.get_aggregations(),
    };

    for path in paths {
        // This is O(mn) but m and n are both small, so it should be fine
        let mut removed_old_objects_in_file = old_objects
            .iter()
            .filter(|(_, fo)| fo.original_file_path == path)
            .collect::<HashMap<_, _>>();

        if path.exists() {
            let obj_in_new_file =
                get_framework_objects_from_schema_file(&path, project.version(), &aggregations)
                    .await?;

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
                debug!("<DCM> Found {name:?} removed from file {path:?}");
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

    debug!("<DCM> new_objects = {new_objects:?}\ndeleted_objects = {deleted_objects:?}\nchanged_objects = {changed_objects:?}\nmoved_objects = {moved_objects:?}");

    // grab the lock to prevent HTTP requests from being processed while we update the route table
    let mut route_table = route_table.write().await;
    for (_, fo) in deleted_objects {
        if let Some(table) = &fo.table {
            drop_table(&project.clickhouse_config.db_name, table, configured_client).await?;
            syncing_process_registry.stop(&fo.topic, &table.name);
        }

        route_table.remove(&schema_file_path_to_ingest_route(
            &framework_object_versions.current_models.base_path,
            &fo.original_file_path,
            fo.data_model.name.clone(),
            &framework_object_versions.current_version,
        ));

        let topics = vec![fo.data_model.name.clone()];
        match redpanda::delete_topics(&project.redpanda_config, topics).await {
            Ok(_) => info!("<DCM> Topics deleted successfully"),
            Err(e) => warn!("Failed to delete topics: {}", e),
        }

        framework_object_versions
            .current_models
            .models
            .remove(&fo.data_model.name);
    }

    for (_, fo) in changed_objects.iter().chain(new_objects.iter()) {
        if let Some(table) = &fo.table {
            create_or_replace_tables(table, configured_client, project.is_production).await?;
            syncing_process_registry.start(
                fo.topic.clone(),
                fo.data_model.columns.clone(),
                table.name.clone(),
                table.columns.clone(),
            );
        }

        let ingest_route = schema_file_path_to_ingest_route(
            &framework_object_versions.current_models.base_path,
            &fo.original_file_path,
            fo.data_model.name.clone(),
            &framework_object_versions.current_version,
        );

        route_table.insert(
            ingest_route,
            RouteMeta {
                original_file_path: fo.original_file_path.clone(),
                topic_name: fo.topic.clone(),
                format: fo.data_model.config.ingestion.format.clone(),
            },
        );
        let topics = vec![fo.topic.clone()];
        match redpanda::create_topics(&project.redpanda_config, topics).await {
            Ok(_) => info!("<DCM> Topics created successfully"),
            Err(e) => warn!("Failed to create topics: {}", e),
        }

        framework_object_versions
            .current_models
            .models
            .insert(fo.data_model.name.clone(), fo.clone());
    }

    for (_, fo) in moved_objects {
        framework_object_versions
            .current_models
            .models
            .insert(fo.data_model.name.clone(), fo);
    }

    let sdk_location = typescript::generator::generate_sdk(&project, framework_object_versions)?;

    let package_manager = package_managers::PackageManager::Npm;
    package_managers::install_packages(&sdk_location, &package_manager)?;
    package_managers::run_build(&sdk_location, &package_manager)?;
    package_managers::link_sdk(&sdk_location, None, &package_manager)?;

    Ok(())
}

struct EventBuckets {
    flows: Vec<Event>,
    aggregations: Vec<Event>,
    data_models: Vec<Event>,
    consumption: Vec<Event>,
}

impl EventBuckets {
    pub fn new(events: Vec<Event>) -> Self {
        info!("Events: {:?}", events);

        let mut flows = Vec::new();
        let mut aggregations = Vec::new();
        let mut data_models = Vec::new();
        let mut consumption = Vec::new();

        for event in events {
            if event.kind.is_create() || event.kind.is_modify() || event.kind.is_remove() {
                if event.paths.iter().any(|path: &PathBuf| {
                    path.iter()
                        .any(|component| component.eq_ignore_ascii_case(FLOWS_DIR))
                }) {
                    flows.push(event);
                } else if event.paths.iter().any(|path: &PathBuf| {
                    path.iter()
                        .any(|component| component.eq_ignore_ascii_case(AGGREGATIONS_DIR))
                }) {
                    aggregations.push(event);
                } else if event.paths.iter().any(|path: &PathBuf| {
                    path.iter()
                        .any(|component| component.eq_ignore_ascii_case(SCHEMAS_DIR))
                }) {
                    data_models.push(event);
                } else if event.paths.iter().any(|path: &PathBuf| {
                    path.iter()
                        .any(|component| component.eq_ignore_ascii_case(CONSUMPTION_DIR))
                }) {
                    consumption.push(event);
                }
            }
        }

        info!("Flows: {:?}", flows);
        info!("Aggregations: {:?}", aggregations);
        info!("Data Models: {:?}", data_models);
        info!("Consumption: {:?}", consumption);

        Self {
            flows,
            aggregations,
            data_models,
            consumption,
        }
    }

    pub fn non_empty(&self) -> bool {
        !self.flows.is_empty()
            || !self.data_models.is_empty()
            || !self.aggregations.is_empty()
            || !self.consumption.is_empty()
    }
}

async fn watch(
    project: Arc<Project>,
    framework_object_versions: &mut FrameworkObjectVersions,
    route_table: &RwLock<HashMap<PathBuf, RouteMeta>>,
    syncing_process_registry: &mut SyncingProcessesRegistry,
    flows_process_registry: &mut FlowProcessRegistry,
    aggregations_process_registry: &mut AggregationProcessRegistry,
    consumption_process_registry: &mut ConsumptionProcessRegistry,
) -> Result<(), anyhow::Error> {
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
                        // pull all events and process them in one go
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

                let bucketed_events = EventBuckets::new(events);

                if bucketed_events.non_empty() {
                    if !bucketed_events.data_models.is_empty() {
                        with_spinner_async(
                            &format!(
                                "Processing {} Data Model(s) changes from file watcher",
                                bucketed_events.data_models.len()
                            ),
                            process_data_models_changes(
                                project.clone(),
                                bucketed_events.data_models,
                                framework_object_versions,
                                route_table,
                                &configured_client,
                                syncing_process_registry,
                            ),
                            !project.is_production,
                        )
                        .await
                        .map_err(|e| {
                            Error::new(
                                ErrorKind::Other,
                                format!("Processing error occurred: {}", e),
                            )
                        })?;
                    } else if !bucketed_events.flows.is_empty() {
                        with_spinner_async(
                            &format!(
                                "Processing {} Flow(s) changes from file watcher",
                                bucketed_events.flows.len()
                            ),
                            process_flows_changes(&project, flows_process_registry),
                            !project.is_production,
                        )
                        .await?;
                    } else if !bucketed_events.aggregations.is_empty() {
                        with_spinner_async(
                            &format!(
                                "Processing {} Aggregation(s) changes from file watcher",
                                bucketed_events.aggregations.len()
                            ),
                            process_aggregations_changes(&project, aggregations_process_registry),
                            !project.is_production,
                        )
                        .await?;
                    } else if !bucketed_events.consumption.is_empty() {
                        with_spinner_async(
                            &format!(
                                "Processing {} Consumption(s) changes from file watcher",
                                bucketed_events.consumption.len()
                            ),
                            process_consumption_changes(&project, consumption_process_registry),
                            !project.is_production,
                        )
                        .await?;
                    }

                    let _ = verify_flows_against_datamodels(&project, framework_object_versions);

                    let _ = post_current_state_to_console(
                        project.clone(),
                        &configured_client,
                        &configured_producer,
                        framework_object_versions,
                    )
                    .await;
                }
            }
            Err(error) => {
                bail!("File watcher event caused a failure: {}", error);
            }
        }
    }
}

/**
 * Process to start/stop/restart the flows based on changes on local files
 *
 * This does not resolve dependencies between files. It just reloads the flow files.
 *
 * This is currently very dumb and restarts all the flows for all the changes.
 * This can be definitely optimized.
 */
pub async fn process_flows_changes(
    project: &Project,
    flows_process_registry: &mut FlowProcessRegistry,
) -> anyhow::Result<()> {
    let flows = get_all_current_flows(project).await?;
    flows_process_registry.stop_all().await?;
    flows_process_registry.start_all(&flows)?;

    Ok(())
}

pub async fn process_aggregations_changes(
    project: &Project,
    aggregations_process_registry: &mut AggregationProcessRegistry,
) -> anyhow::Result<()> {
    aggregations_process_registry.stop().await?;
    aggregations_process_registry.start(Aggregation {
        dir: project.aggregations_dir(),
    })?;

    Ok(())
}

pub async fn process_consumption_changes(
    project: &Project,
    consumption_process_registry: &mut ConsumptionProcessRegistry,
) -> anyhow::Result<()> {
    consumption_process_registry.stop().await?;
    consumption_process_registry.start(Consumption {
        dir: project.consumption_dir(),
    })?;

    Ok(())
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
        syncing_process_registry: SyncingProcessesRegistry,
        flows_process_registry: FlowProcessRegistry,
        aggregations_process_registry: AggregationProcessRegistry,
        consumption_process_registry: ConsumptionProcessRegistry,
    ) -> Result<(), Error> {
        show_message!(MessageType::Info, {
            Message {
                action: "Watching".to_string(),
                details: format!("{:?}", project.app_dir().display()),
            }
        });

        let mut framework_object_versions = framework_object_versions;
        let mut syncing_process_registry = syncing_process_registry;
        let mut flows_process_registry = flows_process_registry;
        let mut aggregations_process_registry = aggregations_process_registry;
        let mut consumption_process_registry = consumption_process_registry;

        tokio::spawn(async move {
            if let Err(error) = watch(
                project,
                &mut framework_object_versions,
                route_table,
                &mut syncing_process_registry,
                &mut flows_process_registry,
                &mut aggregations_process_registry,
                &mut consumption_process_registry,
            )
            .await
            {
                panic!("Watcher error: {error:?}");
            }
        });

        Ok(())
    }
}

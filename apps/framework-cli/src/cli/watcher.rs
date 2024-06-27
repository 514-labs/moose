use log::{debug, info, warn};
use notify::{Event, RecursiveMode, Watcher};
use notify_debouncer_full::new_debouncer;
use std::collections::HashSet;
use std::ops::DerefMut;
use std::sync::Arc;
use std::time::Duration;
use std::{
    collections::HashMap,
    io::{Error, ErrorKind},
    path::PathBuf,
};
use tokio::sync::RwLock;

use crate::framework;
use crate::framework::consumption::model::Consumption;
use crate::framework::consumption::registry::ConsumptionProcessRegistry;

use crate::framework::aggregations::model::Aggregation;
use crate::framework::aggregations::registry::AggregationProcessRegistry;
use crate::framework::controller::{
    create_or_replace_tables, drop_table, schema_file_path_to_ingest_route,
};
use crate::framework::core::code_loader::{
    get_framework_objects_from_schema_file, FrameworkObjectVersions,
};
use crate::framework::core::infrastructure_map::ApiChange;
use crate::framework::data_model::{is_schema_file, DuplicateModelError};
use crate::framework::flows::loader::get_all_current_flows;
use crate::framework::flows::registry::FlowProcessRegistry;
use crate::framework::registry::model::ProcessRegistries;
use crate::infrastructure::kafka_clickhouse_sync::SyncingProcessesRegistry;
use crate::infrastructure::olap::clickhouse_alt_client::{
    get_pool, store_current_state, store_infrastructure_map,
};
use crate::infrastructure::stream::redpanda;
use crate::project::AggregationSet;
use crate::utilities::constants::{
    AGGREGATIONS_DIR, BLOCKS_DIR, CONSUMPTION_DIR, FLOWS_DIR, SCHEMAS_DIR,
};
use crate::{
    framework::controller::RouteMeta,
    infrastructure::olap::{self, clickhouse::ConfiguredDBClient},
    project::Project,
};

use super::display::{self, with_spinner_async, Message, MessageType};
use super::routines::flow::verify_flows_against_datamodels;
use super::settings::Features;

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

    let schema_files = project.data_models_dir();

    let paths = events
        .into_iter()
        .flat_map(|e| e.paths)
        .filter(|p| is_schema_file(p) && p.starts_with(&schema_files))
        .collect::<HashSet<PathBuf>>();

    let mut new_objects = HashMap::new();
    let mut deleted_objects = HashMap::new();
    let mut changed_objects: HashMap<String, crate::framework::core::code_loader::FrameworkObject> =
        HashMap::new();
    let mut moved_objects = HashMap::new();

    let mut extra_models = HashMap::new();

    let old_objects = &framework_object_versions.current_models.models;

    let aggregations = AggregationSet {
        current_version: project.cur_version().to_owned(),
        names: project.get_aggregations(),
    };

    for path in paths {
        // This is O(mn) but m and n are both small, so it should be fine
        let mut removed_old_objects_in_file = old_objects
            .iter()
            .filter(|(_, fo)| fo.original_file_path == path)
            .collect::<HashMap<_, _>>();

        if path.exists() {
            let obj_in_new_file = get_framework_objects_from_schema_file(
                &project,
                &path,
                project.cur_version(),
                &aggregations,
            )
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
            syncing_process_registry.stop_topic_to_table(&fo.topic, &table.name);
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
            syncing_process_registry.start_topic_to_table(
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
                    path.iter().any(|component| {
                        component.eq_ignore_ascii_case(AGGREGATIONS_DIR)
                            || component.eq_ignore_ascii_case(BLOCKS_DIR)
                    })
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
}

// TODO Remove when route table is removed
#[allow(clippy::too_many_arguments)]
async fn watch(
    project: Arc<Project>,
    features: Features,
    framework_object_versions: &mut FrameworkObjectVersions,
    route_table: &RwLock<HashMap<PathBuf, RouteMeta>>,
    route_update_channel: tokio::sync::mpsc::Sender<ApiChange>,
    consumption_apis: &RwLock<HashSet<String>>,
    syncing_process_registry: &mut SyncingProcessesRegistry,
    project_registries: &mut ProcessRegistries,
) -> Result<(), anyhow::Error> {
    let configured_client = olap::clickhouse::create_client(project.clickhouse_config.clone());

    let mut clickhouse_client_v2 = get_pool(&project.clickhouse_config).get_handle().await?;

    let (tx, mut rx) = tokio::sync::mpsc::channel(32);

    let mut debouncer = new_debouncer(Duration::from_secs(1), None, move |res| {
        let _ = tx
            .blocking_send(res)
            .map_err(|e| log::error!("Failed to watcher send debounced event: {:?}", e));
    })?;

    debouncer
        .watcher()
        .watch(project.app_dir().as_ref(), RecursiveMode::Recursive)
        .map_err(|e| Error::new(ErrorKind::Other, format!("Failed to watch file: {}", e)))?;

    while let Some(res) = rx.recv().await {
        match res {
            Ok(debounced_events) => {
                let events = debounced_events
                    .into_iter()
                    .filter(|event| !event.kind.is_access() && !event.kind.is_other())
                    .map(|debounced_event| debounced_event.event)
                    .collect::<Vec<_>>();

                if events.is_empty() {
                    continue;
                }

                let bucketed_events = EventBuckets::new(events);

                if features.core_v2 {
                    with_spinner_async(
                        "Processing Infrastructure changes from file watcher",
                        async {
                            let plan_result = framework::core::plan::plan_changes(
                                &mut clickhouse_client_v2,
                                &project,
                            )
                            .await?;

                            log::info!("Plan Changes: {:?}", plan_result.changes);

                            display::show_changes(&plan_result);
                            framework::core::execute::execute_online_change(
                                &project,
                                &plan_result,
                                route_update_channel.clone(),
                                syncing_process_registry,
                            )
                            .await?;

                            store_infrastructure_map(
                                &mut clickhouse_client_v2,
                                &project.clickhouse_config,
                                &plan_result.target_infra_map,
                            )
                            .await?;

                            Ok(())
                        },
                        !project.is_production,
                    )
                    .await
                    .map_err(|e: anyhow::Error| {
                        Error::new(
                            ErrorKind::Other,
                            format!("Processing error occurred: {}", e),
                        )
                    })?;
                }

                if !bucketed_events.data_models.is_empty() && !features.core_v2 {
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
                }
                if !bucketed_events.flows.is_empty() {
                    with_spinner_async(
                        &format!(
                            "Processing {} Flow(s) changes from file watcher",
                            bucketed_events.flows.len()
                        ),
                        process_flows_changes(&project, &mut project_registries.flows),
                        !project.is_production,
                    )
                    .await?;
                }
                if !bucketed_events.aggregations.is_empty() {
                    with_spinner_async(
                        &format!(
                            "Processing {} Aggregation(s) changes from file watcher",
                            bucketed_events.aggregations.len()
                        ),
                        process_aggregations_changes(
                            &project,
                            &mut project_registries.aggregations,
                        ),
                        !project.is_production,
                    )
                    .await?;
                }
                if !bucketed_events.consumption.is_empty() {
                    with_spinner_async(
                        &format!(
                            "Processing {} Consumption(s) changes from file watcher",
                            bucketed_events.consumption.len()
                        ),
                        process_consumption_changes(
                            &project,
                            &mut project_registries.consumption,
                            consumption_apis.write().await.deref_mut(),
                        ),
                        !project.is_production,
                    )
                    .await?;
                }

                {
                    let mut client = get_pool(&project.clickhouse_config).get_handle().await?;
                    let aggregations = project.get_aggregations();
                    store_current_state(
                        &mut client,
                        framework_object_versions,
                        &aggregations,
                        &project.clickhouse_config,
                    )
                    .await?
                }

                let _ = verify_flows_against_datamodels(&project, framework_object_versions);
            }
            Err(errors) => {
                for error in errors {
                    log::error!("Watcher Error: {:?}", error);
                }
            }
        }
    }

    Ok(())
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
        dir: if aggregations_process_registry.is_blocks_enabled() {
            project.blocks_dir()
        } else {
            project.aggregations_dir()
        },
    })?;

    Ok(())
}

pub async fn process_consumption_changes(
    project: &Project,
    consumption_process_registry: &mut ConsumptionProcessRegistry,
    paths: &mut HashSet<String>,
) -> anyhow::Result<()> {
    paths.clear();
    walkdir::WalkDir::new(project.consumption_dir())
        .into_iter()
        .for_each(|f| {
            if let Ok(f) = f {
                if f.file_type().is_file() && f.path().extension() == Some("ts".as_ref()) {
                    if let Ok(path) = f.path().strip_prefix(project.consumption_dir()) {
                        let mut path = path.to_path_buf();
                        path.set_extension("");
                        paths.insert(path.to_string_lossy().to_string());
                    }
                }
            }
        });

    debug!("Consumption API paths: {:?}", paths);
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

    // TODO Remove when route table is removed
    #[allow(clippy::too_many_arguments)]
    pub fn start(
        &self,
        project: Arc<Project>,
        features: &Features,
        framework_object_versions: FrameworkObjectVersions,
        route_table: &'static RwLock<HashMap<PathBuf, RouteMeta>>,
        route_update_channel: tokio::sync::mpsc::Sender<ApiChange>,
        consumption_apis: &'static RwLock<HashSet<String>>,
        syncing_process_registry: SyncingProcessesRegistry,
        project_registries: ProcessRegistries,
    ) -> Result<(), Error> {
        show_message!(MessageType::Info, {
            Message {
                action: "Watching".to_string(),
                details: format!("{:?}", project.app_dir().display()),
            }
        });

        let mut framework_object_versions = framework_object_versions;
        let mut syncing_process_registry = syncing_process_registry;
        let mut project_registry = project_registries;
        let features = features.clone();

        tokio::spawn(async move {
            if let Err(error) = watch(
                project,
                features.clone(),
                &mut framework_object_versions,
                route_table,
                route_update_channel,
                consumption_apis,
                &mut syncing_process_registry,
                &mut project_registry,
            )
            .await
            {
                panic!("Watcher error: {error:?}");
            }
        });

        Ok(())
    }
}

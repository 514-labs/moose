use crate::framework;
use log::{debug, info, warn};
use notify::event::ModifyKind;
use notify::{Event, EventHandler, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::ops::DerefMut;
use std::sync::Arc;
use std::time::Duration;
use std::{
    collections::HashMap,
    io::{Error, ErrorKind},
    path::PathBuf,
};
use tokio::sync::{Mutex, RwLock};
use tokio::time::sleep;

use crate::framework::controller::{
    create_or_replace_tables, drop_table, schema_file_path_to_ingest_route,
};
use crate::framework::core::code_loader::{
    get_framework_objects_from_schema_file, FrameworkObjectVersions,
};
use crate::framework::core::infrastructure::olap_process::OlapProcess;
use crate::framework::core::infrastructure_map::{ApiChange, InfrastructureMap};
use crate::framework::data_model::model::DataModelSet;
use crate::framework::data_model::{is_schema_file, DuplicateModelError};
use crate::framework::streaming::loader::get_all_current_streaming_functions;

use super::display::{self, with_spinner_async, Message, MessageType};
use super::routines::streaming::verify_streaming_functions_against_datamodels;
use super::settings::Features;
use crate::infrastructure::olap::clickhouse_alt_client::{
    get_pool, store_current_state, store_infrastructure_map,
};
use crate::infrastructure::processes::aggregations_registry::AggregationProcessRegistry;
use crate::infrastructure::processes::consumption_registry::ConsumptionProcessRegistry;
use crate::infrastructure::processes::functions_registry::FunctionProcessRegistry;
use crate::infrastructure::processes::kafka_clickhouse_sync::SyncingProcessesRegistry;
use crate::infrastructure::processes::process_registry::ProcessRegistries;
use crate::infrastructure::redis::redis_client::RedisClient;
use crate::infrastructure::stream::redpanda::{self, fetch_topics};
use crate::metrics::Metrics;
use crate::project::AggregationSet;
use crate::utilities::constants::{
    AGGREGATIONS_DIR, BLOCKS_DIR, CONSUMPTION_DIR, FUNCTIONS_DIR, SCHEMAS_DIR,
};
use crate::utilities::PathExt;
use crate::{
    framework::controller::RouteMeta,
    infrastructure::olap::{self, clickhouse::ConfiguredDBClient},
    project::Project,
};

async fn process_data_models_changes(
    // old v1 code
    project: Arc<Project>,
    paths: HashSet<PathBuf>,
    framework_object_versions: &mut FrameworkObjectVersions,
    route_table: &RwLock<HashMap<PathBuf, RouteMeta>>,
    configured_client: &ConfiguredDBClient,
    syncing_process_registry: &mut SyncingProcessesRegistry,
    metrics: Arc<Metrics>,
) -> anyhow::Result<()> {
    debug!(
        "File Watcher Event Received: {:?}, with Route Table {:?}",
        paths, route_table
    );

    let schema_files = project.data_models_dir();
    let paths = paths
        .into_iter()
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
                project.cur_version().as_str(),
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
                metrics.clone(),
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
                data_model: fo.data_model.clone(),
                format: fo.data_model.config.ingestion.format,
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
struct EventListener {
    tx: tokio::sync::watch::Sender<EventBuckets>,
}

impl EventHandler for EventListener {
    fn handle_event(&mut self, event: notify::Result<Event>) {
        match event {
            Ok(event) => {
                self.tx.send_if_modified(|events| {
                    events.insert(event);
                    !events.is_empty()
                });
            }
            Err(e) => {
                log::error!("Watcher Error: {:?}", e);
            }
        }
    }
}

#[derive(Default, Debug)]
struct EventBuckets {
    functions: HashSet<PathBuf>,
    aggregations: HashSet<PathBuf>,
    data_models: HashSet<PathBuf>,
    consumption: HashSet<PathBuf>,
}

impl EventBuckets {
    pub fn is_empty(&self) -> bool {
        self.functions.is_empty()
            && self.aggregations.is_empty()
            && self.data_models.is_empty()
            && self.consumption.is_empty()
    }

    pub fn insert(&mut self, event: Event) {
        match event.kind {
            EventKind::Access(_) | EventKind::Modify(ModifyKind::Metadata(_)) => return,
            EventKind::Any
            | EventKind::Create(_)
            | EventKind::Modify(_)
            | EventKind::Remove(_)
            | EventKind::Other => {}
        };
        for path in event.paths {
            if !path.ext_is_supported_lang() {
                continue;
            }

            if path
                .iter()
                .any(|component| component.eq_ignore_ascii_case(FUNCTIONS_DIR))
            {
                self.functions.insert(path);
            } else if path.iter().any(|component| {
                component.eq_ignore_ascii_case(AGGREGATIONS_DIR)
                    || component.eq_ignore_ascii_case(BLOCKS_DIR)
            }) {
                self.aggregations.insert(path);
            } else if path
                .iter()
                .any(|component| component.eq_ignore_ascii_case(SCHEMAS_DIR))
            {
                self.data_models.insert(path);
            } else if path
                .iter()
                .any(|component| component.eq_ignore_ascii_case(CONSUMPTION_DIR))
            {
                self.consumption.insert(path);
            }
        }

        info!("Functions: {:?}", self.functions);
        info!("Aggregations/Blocks: {:?}", self.aggregations);
        info!("Data Models: {:?}", self.data_models);
        info!("Consumption: {:?}", self.consumption);
    }
}

// TODO Remove when route table is removed
#[allow(clippy::too_many_arguments)]
async fn watch(
    project: Arc<Project>,
    features: Features,
    framework_object_versions: &mut Option<FrameworkObjectVersions>,
    route_table: &RwLock<HashMap<PathBuf, RouteMeta>>,
    route_update_channel: tokio::sync::mpsc::Sender<ApiChange>,
    infrastructure_map: &'static RwLock<InfrastructureMap>,
    consumption_apis: &RwLock<HashSet<String>>,
    syncing_process_registry: &mut SyncingProcessesRegistry,
    project_registries: &mut ProcessRegistries,
    metrics: Arc<Metrics>,
    redis_client: Arc<Mutex<RedisClient>>,
) -> Result<(), anyhow::Error> {
    let configured_client = olap::clickhouse::create_client(project.clickhouse_config.clone());

    let mut clickhouse_client_v2 = get_pool(&project.clickhouse_config).get_handle().await?;

    let (tx, mut rx) = tokio::sync::watch::channel(EventBuckets::default());
    let receiver_ack = tx.clone();

    let mut watcher = RecommendedWatcher::new(EventListener { tx }, notify::Config::default())
        .map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("Failed to create file watcher: {}", e),
            )
        })?;

    watcher
        .watch(project.app_dir().as_ref(), RecursiveMode::Recursive)
        .map_err(|e| Error::new(ErrorKind::Other, format!("Failed to watch file: {}", e)))?;

    while let Ok(()) = rx.changed().await {
        sleep(Duration::from_secs(1)).await;
        let bucketed_events = receiver_ack.send_replace(EventBuckets::default());
        rx.mark_unchanged();
        // so that updates done between receiver_ack.send_replace and rx.mark_unchanged
        // can be picked up the next rx.changed call
        if !rx.borrow().is_empty() {
            rx.mark_changed();
        }

        if features.core_v2 {
            with_spinner_async(
                "Processing Infrastructure changes from file watcher",
                async {
                    let plan_result =
                        framework::core::plan::plan_changes(&mut clickhouse_client_v2, &project)
                            .await;

                    match plan_result {
                        Ok(plan_result) => {
                            info!("Plan Changes: {:?}", plan_result.changes);

                            display::show_changes(&plan_result);
                            match framework::core::execute::execute_online_change(
                                &project,
                                &plan_result,
                                route_update_channel.clone(),
                                syncing_process_registry,
                                project_registries,
                                metrics.clone(),
                            )
                            .await
                            {
                                Ok(_) => {
                                    store_infrastructure_map(
                                        &mut clickhouse_client_v2,
                                        &project.clickhouse_config,
                                        &plan_result.target_infra_map,
                                    )
                                    .await?;

                                    plan_result
                                        .target_infra_map
                                        .store_in_redis(&*redis_client.lock().await)
                                        .await?;

                                    let mut infra_ptr = infrastructure_map.write().await;
                                    *infra_ptr = plan_result.target_infra_map
                                }
                                Err(e) => {
                                    let error: anyhow::Error = e.into();
                                    show_message!(MessageType::Error, {
                                        Message {
                                            action: "\nFailed".to_string(),
                                            details: format!("Executing changes to the infrastructure failed:\n{:?}", error) 
                                        }
                                    });
                                }
                            }
                        }
                        Err(e) => {
                            let error: anyhow::Error = e.into();
                            show_message!(MessageType::Error, {
                                Message {
                                    action: "\nFailed".to_string(),
                                    details: format!("Planning changes to the infrastructure failed:\n{:?}", error),
                                }
                            });
                        }
                    }
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
        } else {
            // framework_object_versions is None if using core_v2
            let framework_object_versions = framework_object_versions.as_mut().unwrap();

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
                        metrics.clone(),
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

            if !bucketed_events.functions.is_empty() {
                let topics = fetch_topics(&project.redpanda_config).await?;

                with_spinner_async(
                    &format!(
                        "Processing {} Streaming Function(s) changes from file watcher",
                        bucketed_events.functions.len()
                    ),
                    process_streaming_func_changes(
                        &project,
                        &framework_object_versions.to_data_model_set(),
                        &mut project_registries.functions,
                        &topics,
                    ),
                    !project.is_production,
                )
                .await?;
            }

            if !bucketed_events.aggregations.is_empty() {
                with_spinner_async(
                    &format!(
                        "Processing {} Aggregation(s)/Block(s) changes from file watcher",
                        bucketed_events.aggregations.len()
                    ),
                    process_aggregations_changes(&mut project_registries.aggregations),
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

            let mut client = get_pool(&project.clickhouse_config).get_handle().await?;
            let aggregations = project.get_aggregations();
            store_current_state(
                &mut client,
                framework_object_versions,
                &aggregations,
                &project.clickhouse_config,
            )
            .await?;

            let _ =
                verify_streaming_functions_against_datamodels(&project, framework_object_versions);
        }
    }

    Ok(())
}

/**
 * Process to start/stop/restart the streaming functions based on changes on local files
 *
 * This does not resolve dependencies between files. It just reloads the function files.
 *
 * This is currently very dumb and restarts all the function for all the changes.
 * This can be definitely optimized.
 */
pub async fn process_streaming_func_changes(
    project: &Project,
    data_models_set: &DataModelSet,
    function_process_registry: &mut FunctionProcessRegistry,
    topics: &[String],
) -> anyhow::Result<()> {
    let functions = get_all_current_streaming_functions(project, data_models_set).await?;
    function_process_registry.stop_all().await?;
    function_process_registry.start_all(&functions, topics)?;

    Ok(())
}

pub async fn process_aggregations_changes(
    aggregations_process_registry: &mut AggregationProcessRegistry,
) -> anyhow::Result<()> {
    aggregations_process_registry.stop(&OlapProcess {}).await?;
    aggregations_process_registry.start(&OlapProcess {})?;

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
                if f.file_type().is_file() && f.path().ext_is_supported_lang() {
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
    consumption_process_registry.start()?;

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
        framework_object_versions: Option<FrameworkObjectVersions>,
        route_table: &'static RwLock<HashMap<PathBuf, RouteMeta>>,
        route_update_channel: tokio::sync::mpsc::Sender<ApiChange>,
        infrastructure_map: &'static RwLock<InfrastructureMap>,
        consumption_apis: &'static RwLock<HashSet<String>>,
        syncing_process_registry: SyncingProcessesRegistry,
        project_registries: ProcessRegistries,
        metrics: Arc<Metrics>,
        redis_client: Arc<Mutex<RedisClient>>,
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
        let redis_client = redis_client.clone();

        tokio::spawn(async move {
            watch(
                project,
                features.clone(),
                &mut framework_object_versions,
                route_table,
                route_update_channel,
                infrastructure_map,
                consumption_apis,
                &mut syncing_process_registry,
                &mut project_registry,
                metrics,
                redis_client,
            )
            .await
            .unwrap()
        });

        Ok(())
    }
}

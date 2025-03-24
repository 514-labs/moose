/// # File Watcher Module
///
/// This module provides functionality for watching file changes in the project directory
/// and triggering infrastructure updates based on those changes. It monitors files in specific
/// directories like functions, blocks, data models, consumption, and scripts.
///
/// The watcher uses the `notify` crate to detect file system events and processes them
/// to update the infrastructure map, which is then used to apply changes to the system.
///
/// ## Main Components:
/// - `FileWatcher`: The main struct that initializes and starts the file watching process
/// - `EventListener`: Handles file system events and forwards them to the processing pipeline
/// - `EventBuckets`: Categorizes file changes by their directory type
///
/// ## Process Flow:
/// 1. The watcher monitors the project directory for file changes
/// 2. When changes are detected, they are categorized by directory type
/// 3. After a short delay, changes are processed to update the infrastructure
/// 4. The updated infrastructure is applied to the system
use crate::framework;
use log::info;
use notify::event::ModifyKind;
use notify::{Event, EventHandler, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use std::{
    io::{Error, ErrorKind},
    path::PathBuf,
};
use tokio::sync::{Mutex, RwLock};
use tokio::time::sleep;

use crate::framework::core::infrastructure_map::{ApiChange, InfrastructureMap};

use super::display::{self, with_spinner_async, Message, MessageType};

use crate::cli::routines::openapi::openapi;
use crate::infrastructure::processes::kafka_clickhouse_sync::SyncingProcessesRegistry;
use crate::infrastructure::processes::process_registry::ProcessRegistries;
use crate::infrastructure::redis::redis_client::RedisClient;
use crate::metrics::Metrics;
use crate::project::Project;
use crate::utilities::constants::{
    BLOCKS_DIR, CONSUMPTION_DIR, FUNCTIONS_DIR, SCHEMAS_DIR, SCRIPTS_DIR,
};
use crate::utilities::PathExt;

/// Event listener that receives file system events and forwards them to the event processing pipeline.
/// It uses a watch channel to communicate with the main processing loop.
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

/// Container for categorizing file system events by directory type.
/// This helps organize changes for more efficient processing.
#[derive(Default, Debug)]
struct EventBuckets {
    functions: HashSet<PathBuf>,
    blocks: HashSet<PathBuf>,
    data_models: HashSet<PathBuf>,
    consumption: HashSet<PathBuf>,
    scripts: HashSet<PathBuf>,
}

impl EventBuckets {
    /// Checks if all buckets are empty
    pub fn is_empty(&self) -> bool {
        self.functions.is_empty()
            && self.blocks.is_empty()
            && self.data_models.is_empty()
            && self.consumption.is_empty()
            && self.scripts.is_empty()
    }

    /// Processes a file system event and categorizes it into the appropriate bucket
    /// based on the directory path.
    ///
    /// Only processes events that are relevant (create, modify, remove) and
    /// ignores metadata changes and access events.
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
            if !path.ext_is_supported_lang() && !path.ext_is_script_config() {
                continue;
            }

            if path
                .iter()
                .any(|component| component.eq_ignore_ascii_case(FUNCTIONS_DIR))
            {
                self.functions.insert(path);
            } else if path
                .iter()
                .any(|component| component.eq_ignore_ascii_case(BLOCKS_DIR))
            {
                self.blocks.insert(path);
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
            } else if path
                .iter()
                .any(|component| component.eq_ignore_ascii_case(SCRIPTS_DIR))
            {
                self.scripts.insert(path);
            }
        }

        info!("Functions: {:?}", self.functions);
        info!("Blocks: {:?}", self.blocks);
        info!("Data Models: {:?}", self.data_models);
        info!("Consumption: {:?}", self.consumption);
        info!("Scripts: {:?}", self.scripts);
    }
}

/// Main watching function that monitors the project directory for changes and
/// processes them to update the infrastructure.
///
/// This function runs in a loop, waiting for file system events, then after a short delay
/// processes them to update the infrastructure map and apply changes to the system.
///
/// # Arguments
/// * `project` - The project configuration
/// * `route_update_channel` - Channel for sending API route updates
/// * `infrastructure_map` - The current infrastructure map
/// * `syncing_process_registry` - Registry for syncing processes
/// * `project_registries` - Registry for project processes
/// * `metrics` - Metrics collection
/// * `redis_client` - Redis client for state management
async fn watch(
    project: Arc<Project>,
    route_update_channel: tokio::sync::mpsc::Sender<(InfrastructureMap, ApiChange)>,
    infrastructure_map: &'static RwLock<InfrastructureMap>,
    syncing_process_registry: &mut SyncingProcessesRegistry,
    project_registries: &mut ProcessRegistries,
    metrics: Arc<Metrics>,
    redis_client: Arc<Mutex<RedisClient>>,
) -> Result<(), anyhow::Error> {
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
        let _bucketed_events = receiver_ack.send_replace(EventBuckets::default());
        rx.mark_unchanged();
        // so that updates done between receiver_ack.send_replace and rx.mark_unchanged
        // can be picked up the next rx.changed call
        if !rx.borrow().is_empty() {
            rx.mark_changed();
        }

        let _ = with_spinner_async(
            "Processing Infrastructure changes from file watcher",
            async {
                let plan_result =
                    framework::core::plan::plan_changes(&redis_client, &project).await;

                match plan_result {
                    Ok(plan_result) => {
                        info!("Plan Changes: {:?}", plan_result.changes);

                        framework::core::plan_validator::validate(&project, &plan_result)?;

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
                                let redis_guard = redis_client.lock().await;
                                plan_result
                                    .target_infra_map
                                    .store_in_redis(&redis_guard)
                                    .await?;

                                let _openapi_file =
                                    openapi(&project, &plan_result.target_infra_map).await?;

                                let mut infra_ptr = infrastructure_map.write().await;
                                *infra_ptr = plan_result.target_infra_map
                            }
                            Err(e) => {
                                let error: anyhow::Error = e.into();
                                show_message!(MessageType::Error, {
                                    Message {
                                        action: "\nFailed".to_string(),
                                        details: format!(
                                            "Executing changes to the infrastructure failed:\n{:?}",
                                            error
                                        ),
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
                                details: format!(
                                    "Planning changes to the infrastructure failed:\n{:?}",
                                    error
                                ),
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
            show_message!(MessageType::Error, {
                Message {
                    action: "Failed".to_string(),
                    details: format!("Processing Infrastructure changes failed:\n{:?}", e),
                }
            });
        });
    }

    Ok(())
}

/// File watcher that monitors project files for changes and triggers infrastructure updates.
///
/// This struct provides the main interface for starting the file watching process.
pub struct FileWatcher;

impl FileWatcher {
    /// Creates a new FileWatcher instance
    pub fn new() -> Self {
        Self {}
    }

    /// Starts the file watching process.
    ///
    /// This method initializes the watcher and spawns a background task to monitor
    /// file changes and process them.
    ///
    /// # Arguments
    /// * `project` - The project configuration
    /// * `route_update_channel` - Channel for sending API route updates
    /// * `infrastructure_map` - The current infrastructure map
    /// * `syncing_process_registry` - Registry for syncing processes
    /// * `project_registries` - Registry for project processes
    /// * `metrics` - Metrics collection
    /// * `redis_client` - Redis client for state management
    #[allow(clippy::too_many_arguments)]
    pub fn start(
        &self,
        project: Arc<Project>,
        route_update_channel: tokio::sync::mpsc::Sender<(InfrastructureMap, ApiChange)>,
        infrastructure_map: &'static RwLock<InfrastructureMap>,
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

        let mut syncing_process_registry = syncing_process_registry;
        let mut project_registry = project_registries;

        tokio::spawn(async move {
            watch(
                project,
                route_update_channel,
                infrastructure_map,
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

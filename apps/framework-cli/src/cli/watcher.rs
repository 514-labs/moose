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
use crate::infrastructure::olap::clickhouse_alt_client::{get_pool, store_infrastructure_map};
use crate::infrastructure::processes::kafka_clickhouse_sync::SyncingProcessesRegistry;
use crate::infrastructure::processes::process_registry::ProcessRegistries;
use crate::infrastructure::redis::redis_client::RedisClient;
use crate::metrics::Metrics;
use crate::project::Project;
use crate::utilities::constants::{BLOCKS_DIR, CONSUMPTION_DIR, FUNCTIONS_DIR, SCHEMAS_DIR};
use crate::utilities::PathExt;

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
    blocks: HashSet<PathBuf>,
    data_models: HashSet<PathBuf>,
    consumption: HashSet<PathBuf>,
}

impl EventBuckets {
    pub fn is_empty(&self) -> bool {
        self.functions.is_empty()
            && self.blocks.is_empty()
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
            }
        }

        info!("Functions: {:?}", self.functions);
        info!("Blocks: {:?}", self.blocks);
        info!("Data Models: {:?}", self.data_models);
        info!("Consumption: {:?}", self.consumption);
    }
}

async fn watch(
    project: Arc<Project>,
    route_update_channel: tokio::sync::mpsc::Sender<ApiChange>,
    infrastructure_map: &'static RwLock<InfrastructureMap>,
    syncing_process_registry: &mut SyncingProcessesRegistry,
    project_registries: &mut ProcessRegistries,
    metrics: Arc<Metrics>,
    redis_client: Arc<Mutex<RedisClient>>,
) -> Result<(), anyhow::Error> {
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
                    framework::core::plan::plan_changes(&mut clickhouse_client_v2, &project).await;

                match plan_result {
                    Ok(plan_result) => {
                        info!("Plan Changes: {:?}", plan_result.changes);

                        framework::core::plan_validator::validate(&plan_result)?;

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
                                *infra_ptr = plan_result.target_infra_map;

                                let _openapi_file = openapi(&project).await?;
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

pub struct FileWatcher;

impl FileWatcher {
    pub fn new() -> Self {
        Self {}
    }

    #[allow(clippy::too_many_arguments)]
    pub fn start(
        &self,
        project: Arc<Project>,
        route_update_channel: tokio::sync::mpsc::Sender<ApiChange>,
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

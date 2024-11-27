use crate::infrastructure::redis::redis_client::RedisClient;
use clickhouse_rs::ClientHandle;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

use super::{infrastructure_map::ApiChange, plan::InfraPlan};
use crate::infrastructure::migration;
use crate::infrastructure::migration::InitialDataLoadError;
use crate::{
    infrastructure::{
        api,
        olap::{self, OlapChangesError},
        processes::{
            self, kafka_clickhouse_sync::SyncingProcessesRegistry,
            process_registry::ProcessRegistries,
        },
        stream,
    },
    metrics::Metrics,
    project::Project,
};

#[derive(Debug, thiserror::Error)]
pub enum ExecutionError {
    #[error("Failed to communicate with Olap DB")]
    OlapChange(#[from] OlapChangesError),

    #[error("Failed to communicate with streaming engine")]
    StreamingChange(#[from] stream::StreamingChangesError),

    #[error("Failed to communicate with API")]
    ApiChange(#[from] Box<api::ApiChangeError>),

    #[error("Failed to communicate with Sync Processes")]
    SyncProcessesChange(#[from] processes::SyncProcessChangesError),

    #[error("Failed to load to topic for migration")]
    InitialDataLoad(#[from] InitialDataLoadError), // TODO: refactor to concrete types

    #[error("Leadership check failed")]
    LeadershipCheckFailed(anyhow::Error),

    #[error("Migration error")]
    MigrationError(String),
}

pub async fn execute_initial_infra_change(
    project: &Project,
    plan: &InfraPlan,
    api_changes_channel: Sender<ApiChange>,
    metrics: Arc<Metrics>,
    clickhouse_client: &mut ClientHandle,
    redis_client: &Arc<Mutex<RedisClient>>,
) -> Result<(SyncingProcessesRegistry, ProcessRegistries), ExecutionError> {
    // This probably can be parallelized through Tokio Spawn
    olap::execute_changes(project, &plan.changes.olap_changes).await?;
    stream::execute_changes(project, &plan.changes.streaming_engine_changes).await?;

    // In prod, the webserver is part of the current process that gets spawned. As such
    // it is initialized from 0 and we don't need to apply diffs to it.
    api::execute_changes(
        &plan.target_infra_map.init_api_endpoints(),
        api_changes_channel,
    )
    .await
    .map_err(Box::new)?;

    let mut syncing_processes_registry = SyncingProcessesRegistry::new(
        project.redpanda_config.clone(),
        project.clickhouse_config.clone(),
    );
    let mut process_registries = ProcessRegistries::new(project);

    // Execute changes that are allowed on any instance
    let changes = plan.target_infra_map.init_processes();
    processes::execute_changes(
        &mut syncing_processes_registry,
        &mut process_registries,
        &changes,
        metrics,
    )
    .await?;

    // Check if this process instance has the "leadership" lock
    if redis_client
        .lock()
        .await
        .has_lock("leadership")
        .await
        .map_err(ExecutionError::LeadershipCheckFailed)?
    {
        log::info!("Executing changes for leader instance");

        processes::execute_leader_changes(&mut process_registries, &changes).await?;

        migration::execute_changes(project, &plan.changes.initial_data_loads, clickhouse_client)
            .await
            .map_err(|e| ExecutionError::MigrationError(e.to_string()))?;
    } else {
        log::info!("Skipping migration & olap process changes as this instance does not have the leadership lock");
    }

    Ok((syncing_processes_registry, process_registries))
}

pub async fn execute_online_change(
    project: &Project,
    plan: &InfraPlan,
    api_changes_channel: Sender<ApiChange>,
    sync_processes_registry: &mut SyncingProcessesRegistry,
    process_registries: &mut ProcessRegistries,
    metrics: Arc<Metrics>,
) -> Result<(), ExecutionError> {
    // This probably can be paralelized through Tokio Spawn
    olap::execute_changes(project, &plan.changes.olap_changes).await?;
    stream::execute_changes(project, &plan.changes.streaming_engine_changes).await?;

    // In prod, the webserver is part of the current process that gets spawned. As succh
    // it is initialized from 0 and we don't need to apply diffs to it.
    api::execute_changes(&plan.changes.api_changes, api_changes_channel)
        .await
        .map_err(Box::new)?;

    processes::execute_changes(
        sync_processes_registry,
        process_registries,
        &plan.changes.processes_changes,
        metrics,
    )
    .await?;

    Ok(())
}

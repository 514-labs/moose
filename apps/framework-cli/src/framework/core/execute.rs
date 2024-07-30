use std::sync::Arc;
use tokio::sync::mpsc::Sender;

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

use super::{infrastructure_map::ApiChange, plan::InfraPlan};

#[derive(Debug, thiserror::Error)]
pub enum ExecutionError {
    #[error("Failed to communicate with state storage")]
    OlapChange(#[from] OlapChangesError),

    #[error("Failed to communicate with streaming engine")]
    StreamingChange(#[from] stream::StreamingChangesError),

    #[error("Failed to communicate with API")]
    ApiChange(#[from] api::ApiChangeError),

    #[error("Failed to communicate with Sync Processes")]
    SyncProcessesChange(#[from] processes::SyncProcessChangesError),
}

pub async fn execute_initial_infra_change(
    project: &Project,
    plan: &InfraPlan,
    api_changes_channel: Sender<ApiChange>,
    metrics: Arc<Metrics>,
) -> Result<(SyncingProcessesRegistry, ProcessRegistries), ExecutionError> {
    // This probably can be paralelized through Tokio Spawn
    olap::execute_changes(project, &plan.changes.olap_changes).await?;
    stream::execute_changes(project, &plan.changes.streaming_engine_changes).await?;

    // In prod, the webserver is part of the current process that gets spawned. As succh
    // it is initialized from 0 and we don't need to apply diffs to it.
    api::execute_changes(
        &plan.target_infra_map.init_api_endpoints(),
        api_changes_channel,
    )
    .await?;

    let mut syncing_processes_registry = SyncingProcessesRegistry::new(
        project.redpanda_config.clone(),
        project.clickhouse_config.clone(),
    );
    let mut process_registries = ProcessRegistries::new(project);

    processes::execute_changes(
        &mut syncing_processes_registry,
        &mut process_registries,
        &plan.target_infra_map.init_processes(),
        metrics,
    )
    .await?;

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
    api::execute_changes(&plan.changes.api_changes, api_changes_channel).await?;

    processes::execute_changes(
        sync_processes_registry,
        process_registries,
        &plan.changes.processes_changes,
        metrics,
    )
    .await?;

    Ok(())
}

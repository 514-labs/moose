use tokio::sync::mpsc::Sender;

use crate::{
    infrastructure::{
        api,
        kafka_clickhouse_sync::SyncingProcessesRegistry,
        olap::{self, OlapChangesError},
        stream, sync_processes,
    },
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
    SyncProcessesChange(#[from] sync_processes::SyncProcessChangesError),
}

pub async fn execute_prod(
    project: &Project,
    plan: &InfraPlan,
    api_changes_channel: Sender<ApiChange>,
) -> Result<(), ExecutionError> {
    // This probably can be paralelized through Tokio Spawn
    olap::execute_changes(project, &plan.changes.olap_changes).await?;
    stream::execute_changes(project, &plan.changes.streaming_engine_changes).await?;

    // In prod, the webserver is part of the current process that gets spawned. As succh
    // it is initialized from 0 and we don't need to apply diffs to it.
    api::execute_changes(
        plan.target_infra_map.init_api_endpoints(),
        api_changes_channel,
    )
    .await?;

    let syncing_processes_registry = SyncingProcessesRegistry::new(
        project.redpanda_config.clone(),
        project.clickhouse_config.clone(),
    );
    sync_processes::execute_changes(
        syncing_processes_registry,
        &plan.target_infra_map.init_topic_to_table_sync_processes(),
    )?;

    Ok(())
}

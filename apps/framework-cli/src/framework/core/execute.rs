/// # Infrastructure Execution Module
///
/// This module is responsible for executing infrastructure changes based on the planned changes.
/// It coordinates the execution of changes across different infrastructure components:
/// - OLAP database changes
/// - Streaming engine changes
/// - API endpoint changes
/// - Process changes
///
/// The module provides functions for both initial infrastructure setup and online changes
/// during runtime. It also handles leadership-based execution for certain operations that
/// should only be performed by a single instance.
use crate::cli::settings::Settings;
use crate::infrastructure::redis::redis_client::RedisClient;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

use super::{
    infrastructure_map::{ApiChange, InfrastructureMap},
    plan::InfraPlan,
};
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

/// Errors that can occur during the execution of infrastructure changes.
#[derive(Debug, thiserror::Error)]
pub enum ExecutionError {
    /// Error occurred while applying changes to the OLAP database
    #[error("Failed to communicate with Olap DB")]
    OlapChange(#[from] OlapChangesError),

    /// Error occurred while applying changes to the streaming engine
    #[error("Failed to communicate with streaming engine")]
    StreamingChange(#[from] stream::StreamingChangesError),

    /// Error occurred while applying changes to the API endpoints
    #[error("Failed to communicate with API")]
    ApiChange(#[from] Box<api::ApiChangeError>),

    /// Error occurred while applying changes to synchronization processes
    #[error("Failed to communicate with Sync Processes")]
    SyncProcessesChange(#[from] processes::SyncProcessChangesError),

    /// Error occurred while checking leadership status
    #[error("Leadership check failed")]
    LeadershipCheckFailed(anyhow::Error),
}

/// Executes the initial infrastructure changes when the system starts up.
///
/// This function applies all the changes needed to set up the infrastructure from scratch:
/// - Creates OLAP database tables
/// - Sets up streaming engine topics
/// - Initializes API endpoints
/// - Starts synchronization processes
///
/// It also handles leadership-specific operations that should only be performed by
/// the instance that holds the leadership lock.
///
/// # Arguments
/// * `project` - The project configuration
/// * `settings` - Application settings
/// * `plan` - The infrastructure plan to execute
/// * `api_changes_channel` - Channel for sending API changes
/// * `metrics` - Metrics collection
/// * `redis_client` - Redis client for state management and leadership checks
///
/// # Returns
/// * `Result<(SyncingProcessesRegistry, ProcessRegistries), ExecutionError>` - The initialized process registries or an error
pub async fn execute_initial_infra_change(
    project: &Project,
    settings: &Settings,
    plan: &InfraPlan,
    api_changes_channel: Sender<(InfrastructureMap, ApiChange)>,
    metrics: Arc<Metrics>,
    redis_client: &Arc<Mutex<RedisClient>>,
) -> Result<(SyncingProcessesRegistry, ProcessRegistries), ExecutionError> {
    // This probably can be parallelized through Tokio Spawn
    olap::execute_changes(project, &plan.changes.olap_changes).await?;
    stream::execute_changes(project, &plan.changes.streaming_engine_changes).await?;

    // In prod, the webserver is part of the current process that gets spawned. As such
    // it is initialized from 0 and we don't need to apply diffs to it.
    api::execute_changes(
        &plan.target_infra_map,
        &plan.target_infra_map.init_api_endpoints(),
        api_changes_channel,
    )
    .await
    .map_err(Box::new)?;

    let mut syncing_processes_registry = SyncingProcessesRegistry::new(
        project.kafka_config.clone(),
        project.clickhouse_config.clone(),
    );
    let mut process_registries = ProcessRegistries::new(project, settings);

    // Execute changes that are allowed on any instance
    let changes = plan.target_infra_map.init_processes(project);
    processes::execute_changes(
        &plan.target_infra_map,
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
    } else {
        log::info!("Skipping migration & olap process changes as this instance does not have the leadership lock");
    }

    Ok((syncing_processes_registry, process_registries))
}

/// Executes infrastructure changes during runtime (after initial setup).
///
/// This function applies incremental changes to the infrastructure based on the
/// difference between the current and target infrastructure maps:
/// - Updates OLAP database tables
/// - Updates streaming engine topics
/// - Updates API endpoints
/// - Updates synchronization processes
///
/// # Arguments
/// * `project` - The project configuration
/// * `plan` - The infrastructure plan to execute
/// * `api_changes_channel` - Channel for sending API changes
/// * `sync_processes_registry` - Registry for syncing processes
/// * `process_registries` - Registry for project processes
/// * `metrics` - Metrics collection
///
/// # Returns
/// * `Result<(), ExecutionError>` - Success or an error
pub async fn execute_online_change(
    project: &Project,
    plan: &InfraPlan,
    api_changes_channel: Sender<(InfrastructureMap, ApiChange)>,
    sync_processes_registry: &mut SyncingProcessesRegistry,
    process_registries: &mut ProcessRegistries,
    metrics: Arc<Metrics>,
) -> Result<(), ExecutionError> {
    // This probably can be parallelized through Tokio Spawn
    olap::execute_changes(project, &plan.changes.olap_changes).await?;
    stream::execute_changes(project, &plan.changes.streaming_engine_changes).await?;

    // In prod, the webserver is part of the current process that gets spawned. As such
    // it is initialized from 0 and we don't need to apply diffs to it.
    api::execute_changes(
        &plan.target_infra_map,
        &plan.changes.api_changes,
        api_changes_channel,
    )
    .await
    .map_err(Box::new)?;

    processes::execute_leader_changes(process_registries, &plan.changes.processes_changes).await?;
    processes::execute_changes(
        &plan.target_infra_map,
        sync_processes_registry,
        process_registries,
        &plan.changes.processes_changes,
        metrics,
    )
    .await?;

    Ok(())
}

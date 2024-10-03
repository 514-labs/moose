//! # Execution engine for the core infrastructure
//!
//! This module is responsible for executing and managing infrastructure changes within the framework.
//! It serves as the central orchestrator for applying modifications to various components of the system,
//! ensuring that the actual infrastructure aligns with the desired state as defined in the `InfraPlan`.
//!
//! # Key Components
//!
//! - `ExecutionError`: An enum encapsulating various error types that may occur during the execution process.
//! - `execute_initial_infra_change`: A function that bootstraps the initial infrastructure setup.
//!
//! # Responsibilities
//!
//! 1. OLAP (Online Analytical Processing) Management:
//!    - Applying changes to tables, views, and other OLAP structures.
//!    - Handling data migrations and schema updates.
//!
//! 2. Streaming Infrastructure:
//!    - Managing topics, partitions, and other streaming-related components.
//!    - Coordinating changes between streaming and OLAP systems.
//!
//! 3. API Layer Management:
//!    - Updating API endpoints and structures based on infrastructure changes.
//!    - Ensuring consistency between the API layer and underlying data models.
//!
//! 4. Process Synchronization:
//!    - Managing synchronization processes between different components.
//!    - Handling data consistency across various parts of the infrastructure.
//!
//! 5. Initial Data Loading:
//!    - Coordinating the initial population of data into newly created structures.
//!    - Ensuring data integrity during the bootstrapping process.
//!
//! # Interaction with Other Components
//!
//! This module interacts closely with:
//! - Clickhouse for OLAP operations
//! - Kafka or similar systems for streaming
//! - Custom API servers
//! - Various synchronization processes
//! - Metrics collection for monitoring execution progress and performance
//!
//! # Error Handling
//!
//! The `ExecutionError` enum provides a comprehensive set of error types, allowing for
//! precise error reporting and handling throughout the execution process.
//!
//! # Future Considerations
//!
//! As the system evolves, this module may need to accommodate:
//! - More granular control over execution steps
//! - Enhanced rollback capabilities for failed executions
//! - Integration with external monitoring and alerting systems
//! - Support for blue-green deployments or other advanced deployment strategies

use clickhouse_rs::ClientHandle;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

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
    ApiChange(#[from] api::ApiChangeError),

    #[error("Failed to communicate with Sync Processes")]
    SyncProcessesChange(#[from] processes::SyncProcessChangesError),

    #[error("Failed to load to topic for migration")]
    InitialDataLoad(#[from] InitialDataLoadError), // TODO: refactor to concrete types
}

/// Executes the initial infrastructure changes.
///
/// This initializes the infrastructure from the target state and returns the
/// syncing processes registry and the process registries which are used to
/// execute subsequent changes to the infrastructure plan.
pub async fn execute_initial_infra_change(
    project: &Project,
    plan: &InfraPlan,
    api_changes_channel: Sender<ApiChange>,
    metrics: Arc<Metrics>,
    clickhouse_client: &mut ClientHandle,
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

    migration::execute_changes(project, &plan.changes.initial_data_loads, clickhouse_client)
        .await?;

    Ok((syncing_processes_registry, process_registries))
}

/// Executes the changes to the infrastructure plan.
///
/// Leverages the syncing processes registry and the process registries to
/// execute the changes to the infrastructure plan in the correct order.
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

    // In prod, the webserver is part of the current process that gets spawned. As such
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

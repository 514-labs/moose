use kafka_clickhouse_sync::SyncingProcessesRegistry;
use process_registry::ProcessRegistries;

use crate::framework::core::infrastructure_map::{Change, ProcessChange};

use super::olap::clickhouse::{errors::ClickhouseError, mapper::std_columns_to_clickhouse_columns};

pub mod functions_registry;
pub mod kafka_clickhouse_sync;
pub mod process_registry;

#[derive(Debug, thiserror::Error)]
pub enum SyncProcessChangesError {
    #[error("Failed to map columns to Clickhouse columns")]
    ClickhouseMapping(#[from] ClickhouseError),

    #[error("Failed in the function registry")]
    FunctionRegistry(#[from] functions_registry::FunctionRegistryError),
}

/// This method dispatches the execution of the changes to the right streaming engine.
/// When we have multiple streams (Redpanda, RabbitMQ ...) this is where it goes.
pub async fn execute_changes(
    syncing_registry: &mut SyncingProcessesRegistry,
    process_registry: &mut ProcessRegistries,
    changes: &[ProcessChange],
) -> Result<(), SyncProcessChangesError> {
    for change in changes.iter() {
        match change {
            ProcessChange::TopicToTableSyncProcess(Change::Added(sync)) => {
                log::info!("Starting sync process: {:?}", sync.id());
                let target_table_columns = std_columns_to_clickhouse_columns(&sync.columns)?;
                syncing_registry.start_topic_to_table(
                    sync.source_topic_id.clone(),
                    sync.columns.clone(),
                    sync.target_table_id.clone(),
                    target_table_columns,
                );
            }
            ProcessChange::TopicToTableSyncProcess(Change::Removed(sync)) => {
                log::info!("Stoping sync process: {:?}", sync.id());
                syncing_registry.stop_topic_to_table(&sync.source_topic_id, &sync.target_table_id)
            }
            ProcessChange::TopicToTableSyncProcess(Change::Updated { before, after }) => {
                log::info!("Replacing Sync process: {:?} by {:?}", before, after);
                // Order of operations is important here. We don't want to stop the process if the mapping fails.
                let target_table_columns = std_columns_to_clickhouse_columns(&after.columns)?;
                syncing_registry
                    .stop_topic_to_table(&before.source_topic_id, &before.target_table_id);
                syncing_registry.start_topic_to_table(
                    after.source_topic_id.clone(),
                    after.columns.clone(),
                    after.target_table_id.clone(),
                    target_table_columns,
                );
            }
            ProcessChange::FunctionProcess(Change::Added(function_process)) => {
                log::info!("Starting Function process: {:?}", function_process.id());
                process_registry.flows.start(function_process)?;
            }
            ProcessChange::FunctionProcess(Change::Removed(function_process)) => {
                log::info!("Stoping Function process: {:?}", function_process.id());
                process_registry.flows.stop(function_process).await?;
            }
            ProcessChange::FunctionProcess(Change::Updated { before, after }) => {
                log::info!("Updating Function process: {:?}", before.id());
                process_registry.flows.stop(before).await?;
                process_registry.flows.start(after)?;
            }
        }
    }

    Ok(())
}

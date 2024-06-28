use crate::framework::core::infrastructure_map::{Change, ProcessChange};

use super::{
    kafka_clickhouse_sync::SyncingProcessesRegistry,
    olap::clickhouse::{errors::ClickhouseError, mapper::std_columns_to_clickhouse_columns},
};

#[derive(Debug, thiserror::Error)]
pub enum SyncProcessChangesError {
    #[error("Failed to map columns to Clickhouse columns")]
    ClickhouseMapping(#[from] ClickhouseError),
}

/// This method dispatches the execution of the changes to the right streaming engine.
/// When we have multiple streams (Redpanda, RabbitMQ ...) this is where it goes.
pub fn execute_changes(
    mut registry: SyncingProcessesRegistry,
    changes: &[ProcessChange],
) -> Result<(), SyncProcessChangesError> {
    for change in changes.iter() {
        match change {
            ProcessChange::TopicToTableSyncProcess(Change::Added(sync)) => {
                log::info!("Starting sync process: {:?}", sync.id());
                let target_table_columns = std_columns_to_clickhouse_columns(&sync.columns)?;
                registry.start_topic_to_table(
                    sync.source_topic_id.clone(),
                    sync.columns.clone(),
                    sync.target_table_id.clone(),
                    target_table_columns,
                );
            }
            ProcessChange::TopicToTableSyncProcess(Change::Removed(sync)) => {
                log::info!("Stoping sync process: {:?}", sync.id());
                registry.stop_topic_to_table(&sync.source_topic_id, &sync.target_table_id)
            }
            ProcessChange::TopicToTableSyncProcess(Change::Updated { before, after }) => {
                log::info!("Replacing Sync process: {:?} by {:?}", before, after);
                // Order of operations is important here. We don't want to stop the process if the mapping fails.
                let target_table_columns = std_columns_to_clickhouse_columns(&after.columns)?;
                registry.stop_topic_to_table(&before.source_topic_id, &before.target_table_id);
                registry.start_topic_to_table(
                    after.source_topic_id.clone(),
                    after.columns.clone(),
                    after.target_table_id.clone(),
                    target_table_columns,
                );
            }
        }
    }

    Ok(())
}

use std::sync::Arc;

use consumption_registry::ConsumptionError;
use kafka_clickhouse_sync::SyncingProcessesRegistry;
use process_registry::ProcessRegistries;

use crate::{
    framework::{
        blocks::model::BlocksError,
        core::infrastructure_map::{Change, ProcessChange},
    },
    metrics::Metrics,
};

use super::olap::clickhouse::{errors::ClickhouseError, mapper::std_columns_to_clickhouse_columns};

pub mod blocks_registry;
pub mod consumption_registry;
pub mod cron_registry;
pub mod functions_registry;
pub mod kafka_clickhouse_sync;
pub mod process_registry;

#[derive(Debug, thiserror::Error)]
pub enum SyncProcessChangesError {
    #[error("Failed to map columns to Clickhouse columns")]
    ClickhouseMapping(#[from] ClickhouseError),

    #[error("Failed in the function registry")]
    FunctionRegistry(#[from] functions_registry::FunctionRegistryError),

    #[error("Failed in the blocks registry")]
    OlapProcess(#[from] BlocksError),

    #[error("Failed in the consumption registry")]
    ConsumptionProcess(#[from] ConsumptionError),
}

/// This method dispatches the execution of the changes to the right streaming engine.
/// When we have multiple streams (Redpanda, RabbitMQ ...) this is where it goes.
/// This method executes changes that are allowed on any instance.
pub async fn execute_changes(
    syncing_registry: &mut SyncingProcessesRegistry,
    process_registry: &mut ProcessRegistries,
    changes: &[ProcessChange],
    metrics: Arc<Metrics>,
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
                    metrics.clone(),
                );
            }
            ProcessChange::TopicToTableSyncProcess(Change::Removed(sync)) => {
                log::info!("Stopping sync process: {:?}", sync.id());
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
                    metrics.clone(),
                );
            }
            ProcessChange::TopicToTopicSyncProcess(Change::Added(sync)) => {
                log::info!("Starting sync process: {:?}", sync.id());
                syncing_registry.start_topic_to_topic(
                    sync.source_topic_id.clone(),
                    sync.target_topic_id.clone(),
                    metrics.clone(),
                );
            }
            ProcessChange::TopicToTopicSyncProcess(Change::Removed(sync)) => {
                log::info!("Stopping sync process: {:?}", sync.id());
                syncing_registry.stop_topic_to_topic(&sync.target_topic_id)
            }
            // TopicToTopicSyncProcess Updated seems impossible
            ProcessChange::TopicToTopicSyncProcess(Change::Updated { before, after }) => {
                log::info!("Replacing Sync process: {:?} by {:?}", before, after);
                syncing_registry.stop_topic_to_topic(&before.target_topic_id);
                syncing_registry.start_topic_to_topic(
                    after.source_topic_id.clone(),
                    after.target_topic_id.clone(),
                    metrics.clone(),
                );
            }
            ProcessChange::FunctionProcess(Change::Added(function_process)) => {
                log::info!("Starting Function process: {:?}", function_process.id());
                process_registry.functions.start(function_process)?;
            }
            ProcessChange::FunctionProcess(Change::Removed(function_process)) => {
                log::info!("Stopping Function process: {:?}", function_process.id());
                process_registry.functions.stop(function_process).await?;
            }
            ProcessChange::FunctionProcess(Change::Updated { before, after }) => {
                log::info!("Updating Function process: {:?}", before.id());
                process_registry.functions.stop(before).await?;
                process_registry.functions.start(after)?;
            }
            // Olap process changes are conditional on the leader instance
            ProcessChange::OlapProcess(Change::Added(_)) => {}
            ProcessChange::OlapProcess(Change::Removed(_)) => {}
            ProcessChange::OlapProcess(Change::Updated {
                before: _,
                after: _,
            }) => {}
            ProcessChange::ConsumptionApiWebServer(Change::Added(_)) => {
                log::info!("Starting Consumption webserver process");
                process_registry.consumption.start()?;
            }
            ProcessChange::ConsumptionApiWebServer(Change::Removed(_)) => {
                log::info!("Stoping Consumption webserver process");
                process_registry.consumption.stop().await?;
            }
            ProcessChange::ConsumptionApiWebServer(Change::Updated {
                before: _,
                after: _,
            }) => {
                log::info!("Re-Starting Consumption webserver process");
                process_registry.consumption.stop().await?;
                process_registry.consumption.start()?;
            }
        }
    }

    Ok(())
}

/// This method executes changes that are only allowed on the leader instance.
pub async fn execute_leader_changes(
    process_registry: &mut ProcessRegistries,
    changes: &[ProcessChange],
) -> Result<(), SyncProcessChangesError> {
    for change in changes.iter() {
        match change {
            ProcessChange::OlapProcess(Change::Added(olap_process)) => {
                log::info!("Starting Blocks process: {:?}", olap_process.id());
                process_registry.blocks.start(olap_process)?;
            }
            ProcessChange::OlapProcess(Change::Removed(olap_process)) => {
                log::info!("Stopping Blocks process: {:?}", olap_process.id());
                process_registry.blocks.stop(olap_process).await?;
            }
            ProcessChange::OlapProcess(Change::Updated { before, after }) => {
                log::info!("Updating Blocks process: {:?}", before.id());
                process_registry.blocks.stop(before).await?;
                process_registry.blocks.start(after)?;
            }
            _ => {}
        }
    }

    Ok(())
}

use clickhouse::ClickhouseChangesError;

use crate::{framework::core::infrastructure_map::OlapChange, project::Project};

pub mod clickhouse;
pub mod clickhouse_alt_client;

#[derive(Debug, thiserror::Error)]
pub enum OlapChangesError {
    #[error("Failed to execute the changes on Clickhouse")]
    ClickhouseChanges(#[from] ClickhouseChangesError),
}

/// This method dispatches the execution of the changes to the right olap storage.
/// When we have multiple storages (DuckDB, ...) this is where it goes.
pub async fn execute_changes(
    project: &Project,
    changes: &[OlapChange],
) -> Result<(), OlapChangesError> {
    clickhouse::execute_changes(project, changes).await?;

    Ok(())
}

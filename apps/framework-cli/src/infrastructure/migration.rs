use crate::framework::controller::resume_initial_data_load;
use crate::framework::core::infrastructure_map::InitialDataLoadChange;
use crate::infrastructure::olap::clickhouse::errors::ClickhouseError;
use crate::infrastructure::olap::clickhouse::mapper::std_table_to_clickhouse_table;
use crate::project::Project;
use clickhouse_rs::ClientHandle;

#[derive(Debug, thiserror::Error)]
pub enum InitialDataLoadError {
    #[error("Failed to execute the changes on Clickhouse")]
    ClickhouseMapping(#[from] ClickhouseError),

    #[error("Failed to execute the changes on Clickhouse")]
    ClickhouseExecution(#[from] clickhouse_rs::errors::Error),
}

pub async fn execute_changes(
    project: &Project,
    changes: &[InitialDataLoadChange],
    client: &mut ClientHandle,
) -> Result<(), InitialDataLoadError> {
    for change in changes {
        let (load, resume_from) = match change {
            InitialDataLoadChange::Addition(load) => (load, 0i64),
            InitialDataLoadChange::Resumption { load, resume_from } => (load, *resume_from),
        };

        let table = std_table_to_clickhouse_table(&load.table)?;

        resume_initial_data_load(
            &table,
            &project.clickhouse_config,
            client,
            &load.topic,
            &project.redpanda_config,
            resume_from,
        )
        .await?;
    }

    Ok(())
}

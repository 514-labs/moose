use crate::framework::controller::resume_initial_data_load;
use crate::framework::core::infrastructure_map::InitialDataLoadChange;
use crate::infrastructure::olap::clickhouse::create_client;
use crate::infrastructure::olap::clickhouse::mapper::std_table_to_clickhouse_table;
use crate::project::Project;

pub async fn execute_changes(
    project: &Project,
    changes: &[InitialDataLoadChange],
) -> anyhow::Result<()> {
    for change in changes {
        let (load, resume_from) = match change {
            InitialDataLoadChange::Addition(load) => (load, 0i64),
            InitialDataLoadChange::Resumption { load, resume_from } => (load, *resume_from),
        };

        let table = std_table_to_clickhouse_table(&load.table)?;
        let client = create_client(project.clickhouse_config.clone());

        resume_initial_data_load(
            &table,
            &client,
            &load.topic,
            &project.redpanda_config,
            resume_from,
        )
        .await?;
    }

    Ok(())
}

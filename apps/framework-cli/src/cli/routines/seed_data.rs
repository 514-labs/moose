use crate::cli::routines::RoutineFailure;
use crate::framework::core::infrastructure_map::InfrastructureMap;
use crate::infrastructure::olap::clickhouse::client::ClickHouseClient;
use crate::infrastructure::olap::clickhouse::config::ClickHouseConfig;
use log::{debug, info};

/// Copies data from remote ClickHouse tables into local ClickHouse tables using the remoteSecure() table function.
pub async fn seed_clickhouse_tables(
    infra_map: &InfrastructureMap,
    local_clickhouse: &ClickHouseClient,
    remote_config: &ClickHouseConfig,
    table_name: Option<String>,
    limit: usize,
) -> Result<Vec<String>, RoutineFailure> {
    let remote_host = &remote_config.host;
    let remote_db = &remote_config.db_name;
    let remote_user = &remote_config.user;
    let remote_password = &remote_config.password;
    let local_db = &local_clickhouse.config().db_name;

    let mut summary = Vec::new();
    let tables: Vec<String> = if let Some(ref t) = table_name {
        info!("Seeding single table: {}", t);
        vec![t.clone()]
    } else {
        let table_list: Vec<String> = infra_map
            .tables
            .keys()
            .filter(|table| !table.starts_with("_MOOSE"))
            .cloned()
            .collect();
        info!(
            "Seeding {} tables (excluding internal Moose tables)",
            table_list.len()
        );
        table_list
    };

    for table_name in tables {
        let sql = format!(
            "INSERT INTO `{db}`.`{table}` SELECT * FROM remoteSecure('{}', '{}', '{}', '{}', '{}') LIMIT {limit}",
            remote_host,
            remote_db,
            table_name,
            remote_user,
            remote_password,
            db = local_db,
            table = table_name,
            limit = limit
        );

        debug!("Executing SQL: {}", sql);

        match local_clickhouse.execute_sql(&sql).await {
            Ok(_) => {
                summary.push(format!("✓ {}: copied from remote", table_name));
            }
            Err(e) => {
                summary.push(format!("✗ {}: failed to copy - {}", table_name, e));
            }
        }
    }

    info!("ClickHouse seeding completed");
    Ok(summary)
}

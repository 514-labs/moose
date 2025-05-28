use crate::cli::commands::SeedSubcommands;
use crate::cli::display::Message;
use crate::cli::routines::RoutineFailure;
use crate::cli::routines::RoutineSuccess;
use crate::framework::core::infrastructure_map::InfrastructureMap;
use crate::framework::core::primitive_map::PrimitiveMap;
use crate::infrastructure::olap::clickhouse::client::ClickHouseClient;
use crate::infrastructure::olap::clickhouse::config::ClickHouseConfig;
use crate::project::Project;
use crate::utilities::clickhouse_url::normalize_clickhouse_url;
use log::{debug, info};

fn parse_clickhouse_connection_string(conn_str: &str) -> anyhow::Result<ClickHouseConfig> {
    let url = normalize_clickhouse_url(conn_str)?;

    let user = url.username().to_string();
    let password = url.password().unwrap_or("").to_string();
    let host = url.host_str().unwrap_or("localhost").to_string();
    let use_ssl = url.scheme() == "https";
    let default_port = if use_ssl { 443 } else { 8123 };
    let port = url.port().unwrap_or(default_port) as i32;

    // Try to get db_name from query string first, then from path
    let db_name = url
        .query_pairs()
        .find(|(k, _)| k == "database")
        .map(|(_, v)| v.to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            let path = url.path().trim_start_matches('/').to_string();
            if !path.is_empty() {
                Some(path)
            } else {
                None
            }
        })
        .unwrap_or_else(|| "default".to_string());

    Ok(ClickHouseConfig {
        db_name,
        user,
        password,
        use_ssl,
        host,
        host_port: port,
        native_port: port,
        host_data_path: None,
    })
}

pub async fn handle_seed_command(
    seed_args: &crate::cli::commands::SeedCommands,
    project: &Project,
) -> Result<RoutineSuccess, RoutineFailure> {
    match &seed_args.command {
        Some(SeedSubcommands::Clickhouse {
            connection_string,
            limit,
            table,
        }) => {
            info!(
                "Running seed clickhouse command with connection string: {}",
                connection_string
            );

            let infra_map = if project.features.data_model_v2 {
                InfrastructureMap::load_from_user_code(project)
                    .await
                    .map_err(|e| {
                        RoutineFailure::error(Message {
                            action: "SeedClickhouse".to_string(),
                            details: format!("Failed to load InfrastructureMap: {:?}", e),
                        })
                    })?
            } else {
                let primitive_map = PrimitiveMap::load(project).await.map_err(|e| {
                    RoutineFailure::error(Message {
                        action: "SeedClickhouse".to_string(),
                        details: format!("Failed to load Primitives: {:?}", e),
                    })
                })?;
                InfrastructureMap::new(project, primitive_map)
            };

            // Parse connection string and create remote ClickHouseConfig
            let remote_config =
                parse_clickhouse_connection_string(connection_string).map_err(|e| {
                    RoutineFailure::error(Message::new(
                        "SeedClickhouse".to_string(),
                        format!("Invalid connection string: {}", e),
                    ))
                })?;

            // Create local ClickHouseClient from local config
            let local_clickhouse =
                ClickHouseClient::new(&project.clickhouse_config).map_err(|e| {
                    RoutineFailure::error(Message::new(
                        "SeedClickhouse".to_string(),
                        format!("Failed to create local ClickHouseClient: {}", e),
                    ))
                })?;

            let summary = seed_clickhouse_tables(
                &infra_map,
                &local_clickhouse,
                &remote_config,
                table.clone(),
                *limit,
            )
            .await?;

            Ok(RoutineSuccess::success(Message::new(
                "Seeded ClickHouse".to_string(),
                format!("{}", summary.join("\n")),
            )))
        }
        None => Err(RoutineFailure::error(Message {
            action: "Seed".to_string(),
            details: "No subcommand provided".to_string(),
        })),
    }
}

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

use crate::cli::commands::SeedSubcommands;
use crate::cli::display::Message;
use crate::cli::routines::RoutineFailure;
use crate::cli::routines::RoutineSuccess;
use crate::framework::core::infrastructure_map::InfrastructureMap;
use crate::framework::core::primitive_map::PrimitiveMap;
use crate::infrastructure::olap::clickhouse::client::ClickHouseClient;
use crate::infrastructure::olap::clickhouse::config::ClickHouseConfig;
use crate::project::Project;
use crate::utilities::clickhouse_url::convert_http_to_clickhouse;
use log::{debug, info};

fn parse_clickhouse_connection_string(
    conn_str: &str,
) -> anyhow::Result<(ClickHouseConfig, Option<String>)> {
    let url = convert_http_to_clickhouse(conn_str)?;

    let user = url.username().to_string();
    let password = url.password().unwrap_or("").to_string();
    let host = url.host_str().unwrap_or("localhost").to_string();

    // Determine SSL based on scheme and port
    let use_ssl = match url.scheme() {
        "https" => true,
        "clickhouse" => url.port().unwrap_or(9000) == 9440,
        _ => url.port().unwrap_or(9000) == 9440,
    };

    let port = url.port().unwrap_or(if use_ssl { 9440 } else { 9000 }) as i32;

    // Get database name from path or query parameter
    let db_name = if !url.path().is_empty() && url.path() != "/" {
        Some(url.path().trim_start_matches('/').to_string())
    } else {
        url.query_pairs()
            .find(|(k, _)| k == "database")
            .map(|(_, v)| v.to_string())
            .filter(|s| !s.is_empty())
    };

    let config = ClickHouseConfig {
        db_name: db_name.clone().unwrap_or_default(),
        user,
        password,
        use_ssl,
        host,
        host_port: port,
        native_port: port,
        host_data_path: None,
    };

    Ok((config, db_name))
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

            let (mut remote_config, db_name) =
                parse_clickhouse_connection_string(connection_string).map_err(|e| {
                    RoutineFailure::error(Message::new(
                        "SeedClickhouse".to_string(),
                        format!("Invalid connection string: {}", e),
                    ))
                })?;

            if db_name.is_none() {
                let mut client = clickhouse::Client::default().with_url(connection_string);
                let url = convert_http_to_clickhouse(connection_string).map_err(|e| {
                    RoutineFailure::error(Message::new(
                        "SeedClickhouse".to_string(),
                        format!("Failed to parse connection string: {}", e),
                    ))
                })?;

                if !url.username().is_empty() {
                    client = client.with_user(url.username());
                }
                if let Some(password) = url.password() {
                    client = client.with_password(password);
                }

                let current_db = client
                    .query("select database()")
                    .fetch_one::<String>()
                    .await
                    .map_err(|e| {
                        RoutineFailure::error(Message::new(
                            "SeedClickhouse".to_string(),
                            format!("Failed to query remote database: {}", e),
                        ))
                    })?;

                remote_config.db_name = current_db;
            }

            // Ensure we have a valid database name
            if remote_config.db_name.is_empty() {
                return Err(RoutineFailure::error(Message::new(
                    "SeedClickhouse".to_string(),
                    "No database specified in connection string and unable to determine current database".to_string(),
                )));
            }

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
                summary.join("\n"),
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
    let remote_port = &remote_config.native_port;
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

    let remote_host_and_port = format!("{}:{}", remote_host, remote_port);

    for table_name in tables {
        let sql = format!(
            "INSERT INTO `{db}`.`{table}` SELECT * FROM remoteSecure('{}', '{}', '{}', '{}', '{}') LIMIT {limit}",
            remote_host_and_port,
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

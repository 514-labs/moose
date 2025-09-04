use crate::cli::commands::SeedSubcommands;
use crate::cli::display::Message;
use crate::cli::display::{self, MessageType};
use crate::cli::routines::RoutineFailure;
use crate::cli::routines::RoutineSuccess;
use crate::framework::core::infrastructure_map::InfrastructureMap;
use crate::framework::core::primitive_map::PrimitiveMap;
use crate::infrastructure::olap::clickhouse::client::ClickHouseClient;
use crate::infrastructure::olap::clickhouse::config::ClickHouseConfig;
use crate::project::Project;
use crate::utilities::clickhouse_url::convert_http_to_clickhouse;
use log::{debug, info};
use tokio::signal;
use tokio::sync::watch;

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
            all,
            table,
        }) => {
            info!(
                "Running seed clickhouse command (source: {})",
                if connection_string.is_some() {
                    "--connection-string flag"
                } else {
                    "MOOSE_SEED_CLICKHOUSE_URL env"
                }
            );

            let infra_map = if project.features.data_model_v2 {
                InfrastructureMap::load_from_user_code(project)
                    .await
                    .map_err(|e| {
                        RoutineFailure::error(Message {
                            action: "SeedClickhouse".to_string(),
                            details: format!("Failed to load InfrastructureMap: {e:?}"),
                        })
                    })?
            } else {
                let primitive_map = PrimitiveMap::load(project).await.map_err(|e| {
                    RoutineFailure::error(Message {
                        action: "SeedClickhouse".to_string(),
                        details: format!("Failed to load Primitives: {e:?}"),
                    })
                })?;
                InfrastructureMap::new(project, primitive_map)
            };

            // Pull connection string from flag or env var
            let conn_str = connection_string
                .clone()
                .or_else(|| std::env::var("MOOSE_SEED_CLICKHOUSE_URL").ok())
                .ok_or_else(|| {
                    RoutineFailure::error(Message::new(
                        "SeedClickhouse".to_string(),
                        "Provide --connection-string or set MOOSE_SEED_CLICKHOUSE_URL".to_string(),
                    ))
                })?;

            let (mut remote_config, db_name) = parse_clickhouse_connection_string(&conn_str)
                .map_err(|e| {
                    RoutineFailure::error(Message::new(
                        "SeedClickhouse".to_string(),
                        format!("Invalid connection string: {e}"),
                    ))
                })?;

            if db_name.is_none() {
                let mut client = clickhouse::Client::default().with_url(&conn_str);
                let url = convert_http_to_clickhouse(&conn_str).map_err(|e| {
                    RoutineFailure::error(Message::new(
                        "SeedClickhouse".to_string(),
                        format!("Failed to parse connection string: {e}"),
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
                            format!("Failed to query remote database: {e}"),
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
                        format!("Failed to create local ClickHouseClient: {e}"),
                    ))
                })?;

            let summary = seed_clickhouse_tables(
                &infra_map,
                &local_clickhouse,
                &remote_config,
                table.clone(),
                if *all { None } else { Some(*limit) },
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
    limit: Option<usize>,
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

    let remote_host_and_port = format!("{remote_host}:{remote_port}");

    for table_name in tables {
        if let Some(l) = limit {
            // Always batch when limit is large to avoid remote caps and memory pressure
            let batch_size: usize = 50_000;
            display::show_message_wrapper(
                MessageType::Info,
                Message::new(
                    "Seed".to_string(),
                    format!("{table_name}: total rows to copy = {l}"),
                ),
            );

            // Preflight remote access
            let preflight = format!(
                "SELECT 1 FROM remoteSecure('{remote_host_and_port}', '{remote_db}', '{table_name}', '{remote_user}', '{remote_password}') LIMIT 1"
            );
            if let Err(e) = local_clickhouse.execute_sql(&preflight).await {
                summary.push(format!("✗ {table_name}: remote access failed - {e}"));
                continue;
            }

            // Graceful cancel (finish current batch)
            let (cancel_tx, cancel_rx) = watch::channel(false);
            {
                let table_for_sig = table_name.clone();
                tokio::spawn(async move {
                    if signal::ctrl_c().await.is_ok() {
                        let _ = cancel_tx.send(true);
                        display::show_message_wrapper(
                            MessageType::Highlight,
                            Message::new(
                                "Seed".to_string(),
                                format!("{table_for_sig}: cancellation requested; finishing current batch"),
                            ),
                        );
                    }
                });
            }

            let mut copied_total: usize = 0;
            let mut batches_copied: usize = 0;
            let mut had_error: bool = false;
            let mut last_error: Option<String> = None;
            while copied_total < l {
                if *cancel_rx.borrow() {
                    break;
                }
                let this_batch = std::cmp::min(batch_size, l - copied_total);
                let sql = format!(
                    "INSERT INTO `{local_db}`.`{table_name}` SELECT * FROM remoteSecure('{remote_host_and_port}', '{remote_db}', '{table_name}', '{remote_user}', '{remote_password}') LIMIT {this_batch} OFFSET {copied_total}"
                );

                debug!(
                    "Executing SQL (limited batch): table={}, offset={}, batch={}",
                    table_name, copied_total, this_batch
                );

                match local_clickhouse.execute_sql(&sql).await {
                    Ok(_) => {
                        copied_total += this_batch;
                        batches_copied += 1;
                        display::show_message_wrapper(
                            MessageType::Info,
                            Message::new(
                                "Seed".to_string(),
                                format!(
                                    "{table_name}: copied batch ~{} ({} / {} rows)",
                                    this_batch, copied_total, l
                                ),
                            ),
                        );

                        if copied_total >= l {
                            break;
                        }

                        // Probe if any rows remain beyond current offset in remote
                        let probe_sql = format!(
                            "SELECT 1 FROM remoteSecure('{remote_host_and_port}', '{remote_db}', '{table_name}', '{remote_user}', '{remote_password}') LIMIT 1 OFFSET {copied_total}"
                        );
                        if let Ok(body) = local_clickhouse.execute_sql(&probe_sql).await {
                            if body.trim().is_empty() {
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        had_error = true;
                        last_error = Some(e.to_string());
                        break;
                    }
                }
            }

            let final_copied = std::cmp::min(copied_total, l);
            if batches_copied > 0 && !had_error {
                summary.push(format!(
                    "✓ {table_name}: copied {} row(s) in {} batch(es)",
                    final_copied, batches_copied
                ));
            } else if batches_copied > 0 && had_error {
                summary.push(format!(
                    "! {table_name}: copied {} row(s) partially in {} batch(es); encountered errors",
                    final_copied, batches_copied
                ));
            } else {
                let detail = last_error.unwrap_or_else(|| "unknown error".to_string());
                summary.push(format!(
                    "✗ {table_name}: failed to copy — no batches completed ({detail})"
                ));
            }
        } else {
            // Copy entire table in batches to avoid remote row limits and memory-heavy sorting
            // Use a conservative batch size and avoid ORDER BY on the remote to prevent full-table sorts
            let batch_size: usize = 50_000;
            let order_by_clause = String::new();

            // Compute total rows up front, to report progress and exact totals
            let count_sql = format!(
                "SELECT count() FROM remoteSecure('{remote_host_and_port}', '{remote_db}', '{table_name}', '{remote_user}', '{remote_password}')"
            );
            let total_rows: Option<u64> = match local_clickhouse.execute_sql(&count_sql).await {
                Ok(body) => body.trim().parse::<u64>().ok(),
                Err(_) => None,
            };
            display::show_message_wrapper(
                MessageType::Info,
                Message::new(
                    "Seed".to_string(),
                    match total_rows {
                        Some(n) => format!("{table_name}: total rows to copy = {}", n),
                        None => {
                            format!("{table_name}: total rows to copy = unknown (count failed)")
                        }
                    },
                ),
            );

            // Set up graceful cancellation: finish current batch, then stop
            let (cancel_tx, cancel_rx) = watch::channel(false);
            {
                let table_for_sig = table_name.clone();
                tokio::spawn(async move {
                    if signal::ctrl_c().await.is_ok() {
                        let _ = cancel_tx.send(true);
                        display::show_message_wrapper(
                            MessageType::Highlight,
                            Message::new(
                                "Seed".to_string(),
                                format!("{table_for_sig}: cancellation requested; finishing current batch"),
                            ),
                        );
                    }
                });
            }

            // Preflight: verify remote table is accessible
            let preflight = format!(
                "SELECT 1 FROM remoteSecure('{remote_host_and_port}', '{remote_db}', '{table_name}', '{remote_user}', '{remote_password}'){order_by_clause} LIMIT 1"
            );
            if let Err(e) = local_clickhouse.execute_sql(&preflight).await {
                summary.push(format!("✗ {table_name}: remote access failed - {e}"));
                continue;
            }

            let mut copied_total: usize = 0;
            let mut batches_copied: usize = 0;
            let mut had_error: bool = false;
            let mut last_error: Option<String> = None;
            loop {
                // Stop before starting next batch if cancellation requested
                if *cancel_rx.borrow() {
                    break;
                }
                let sql = format!(
                    "INSERT INTO `{local_db}`.`{table_name}` SELECT * FROM remoteSecure('{remote_host_and_port}', '{remote_db}', '{table_name}', '{remote_user}', '{remote_password}') LIMIT {batch_size} OFFSET {copied_total}"
                );

                debug!(
                    "Executing SQL (batch): table={}, offset={}, batch={}",
                    table_name, copied_total, batch_size
                );

                match local_clickhouse.execute_sql(&sql).await {
                    Ok(_) => {
                        // We cannot easily know how many rows were inserted from the response,
                        // assume full batch unless next iteration returns 0.
                        copied_total += batch_size;
                        batches_copied += 1;
                        let copied_so_far = match total_rows {
                            Some(t) => std::cmp::min(copied_total as u64, t),
                            None => copied_total as u64,
                        };
                        display::show_message_wrapper(
                            MessageType::Info,
                            Message::new(
                                "Seed".to_string(),
                                match total_rows {
                                    Some(t) => format!(
                                        "{table_name}: copied batch ~{} ({} / {} rows)",
                                        batch_size, copied_so_far, t
                                    ),
                                    None => format!(
                                        "{table_name}: copied batch ~{} (offset {})",
                                        batch_size, copied_total
                                    ),
                                },
                            ),
                        );
                        // If we know total rows, stop when reached
                        if let Some(t) = total_rows {
                            if copied_total as u64 >= t {
                                break;
                            }
                        }
                        // Heuristic: try a probe with LIMIT 1 OFFSET copied_total; if no more rows, break.
                        let probe_sql = format!(
                            "SELECT 1 FROM remoteSecure('{remote_host_and_port}', '{remote_db}', '{table_name}', '{remote_user}', '{remote_password}') LIMIT 1 OFFSET {copied_total}"
                        );
                        match local_clickhouse.execute_sql(&probe_sql).await {
                            Ok(body) => {
                                if body.trim().is_empty() {
                                    // No more rows
                                    break;
                                }
                            }
                            Err(_) => {
                                // Probe failed; proceed to next iteration to fail safe or finish
                            }
                        }
                        // If probe succeeded, continue loop
                    }
                    Err(e) => {
                        // If we received an error due to no more rows or remote limits,
                        // stop the loop and report partial success
                        debug!("Batch copy ended for {}: {}", table_name, e);
                        had_error = true;
                        last_error = Some(e.to_string());
                        break;
                    }
                }
            }
            let final_copied = match total_rows {
                Some(t) => std::cmp::min(copied_total as u64, t),
                None => copied_total as u64,
            };
            if batches_copied > 0 && !had_error {
                summary.push(format!(
                    "✓ {table_name}: copied {} row(s) in {} batch(es)",
                    final_copied, batches_copied
                ));
            } else if batches_copied > 0 && had_error {
                summary.push(format!(
                    "! {table_name}: copied {} row(s) partially in {} batch(es); encountered errors",
                    final_copied, batches_copied
                ));
            } else {
                let detail = last_error.unwrap_or_else(|| "unknown error".to_string());
                summary.push(format!(
                    "✗ {table_name}: failed to copy — no batches completed ({detail})"
                ));
            }
        }
    }

    info!("ClickHouse seeding completed");
    Ok(summary)
}

use crate::cli::display::Message;
use crate::cli::routines::{RoutineFailure, RoutineSuccess};
use crate::infrastructure::olap::clickhouse::{check_ready, create_client, run_query};
use crate::project::Project;
use log::{info, warn};

fn escape_ident(ident: &str) -> String {
    ident.replace('`', "``")
}

async fn list_all_tables(project: &Project) -> Result<Vec<String>, RoutineFailure> {
    let client = create_client(project.clickhouse_config.clone());
    check_ready(&client).await.map_err(|e| {
        RoutineFailure::error(Message::new(
            "ClickHouse".to_string(),
            format!("Failed to connect: {e}"),
        ))
    })?;

    let db_name = &client.config.db_name;
    let query = format!(
        "SELECT name FROM system.tables WHERE database = '{}' AND engine NOT IN ('View','MaterializedView') AND NOT name LIKE '.%' ORDER BY name",
        db_name
    );

    let rows = client
        .client
        .query(&query)
        .fetch_all::<String>()
        .await
        .map_err(|e| {
            RoutineFailure::error(Message::new(
                "ClickHouse".to_string(),
                format!("Failed to list tables: {e}"),
            ))
        })?;

    Ok(rows)
}

async fn truncate_all_rows(project: &Project, tables: &[String]) -> Result<(), RoutineFailure> {
    let client = create_client(project.clickhouse_config.clone());
    check_ready(&client).await.map_err(|e| {
        RoutineFailure::error(Message::new(
            "ClickHouse".to_string(),
            format!("Failed to connect: {e}"),
        ))
    })?;

    let db_name = &client.config.db_name;
    for t in tables {
        let table = escape_ident(t);
        let sql = format!("TRUNCATE TABLE `{}`.`{}`", db_name, table);
        info!("Truncating table {}.{}", db_name, t);
        run_query(&sql, &client).await.map_err(|e| {
            RoutineFailure::error(Message::new(
                "Truncate".to_string(),
                format!("Failed on {}: {e}", t),
            ))
        })?;
    }
    Ok(())
}

async fn delete_last_n_rows(
    project: &Project,
    tables: &[String],
    n: u64,
) -> Result<(), RoutineFailure> {
    let client = create_client(project.clickhouse_config.clone());
    check_ready(&client).await.map_err(|e| {
        RoutineFailure::error(Message::new(
            "ClickHouse".to_string(),
            format!("Failed to connect: {e}"),
        ))
    })?;

    let db_name = &client.config.db_name;

    for t in tables {
        // Discover ORDER BY columns for stable recency semantics
        let create_stmt_query = format!(
            "SELECT create_table_query FROM system.tables WHERE database = '{}' AND name = '{}'",
            db_name, t
        );
        let create_stmt = client
            .client
            .query(&create_stmt_query)
            .fetch_one::<String>()
            .await
            .unwrap_or_else(|_| "".to_string());

        // Fallback to single `timestamp` if we cannot parse; otherwise parse ORDER BY(...)
        let order_by = if let Some(pos) = create_stmt.to_uppercase().find("ORDER BY") {
            let after = &create_stmt[pos + "ORDER BY".len()..];
            let end = after.to_uppercase().find("SETTINGS").unwrap_or(after.len());
            let content = after[..end].trim().trim_matches('(').trim_matches(')');
            let cols: Vec<String> = content
                .split(',')
                .map(|s| s.trim().trim_matches('`').to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if cols.is_empty() {
                vec!["timestamp".to_string()]
            } else {
                cols
            }
        } else {
            vec!["timestamp".to_string()]
        };

        // Build ORDER BY clause and projection for IN subquery
        let proj = if order_by.len() == 1 {
            format!("`{}`", escape_ident(&order_by[0]))
        } else {
            let cols = order_by
                .iter()
                .map(|c| format!("`{}`", escape_ident(c)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("({})", cols)
        };
        let ord = order_by
            .iter()
            .map(|c| format!("`{}` DESC", escape_ident(c)))
            .collect::<Vec<_>>()
            .join(", ");

        let table = escape_ident(t);
        let sql = format!(
            "ALTER TABLE `{}`.`{}` DELETE WHERE {} IN (SELECT {} FROM `{}`.`{}` ORDER BY {} LIMIT {}) SETTINGS mutations_sync=1",
            db_name, table, proj, proj, db_name, table, ord, n
        );
        warn!(
            "Deleting last {} rows from {}.{} using ORDER BY: {:?}",
            n, db_name, t, order_by
        );
        run_query(&sql, &client).await.map_err(|e| {
            RoutineFailure::error(Message::new(
                "Truncate".to_string(),
                format!("Failed on {}: {e}", t),
            ))
        })?;
    }

    Ok(())
}

pub async fn truncate_tables(
    project: &Project,
    tables: Vec<String>,
    all: bool,
    rows: Option<u64>,
) -> Result<RoutineSuccess, RoutineFailure> {
    let target_tables = if all {
        list_all_tables(project).await?
    } else if tables.is_empty() {
        return Err(RoutineFailure::error(Message::new(
            "Truncate".to_string(),
            "Provide table names or use --all".to_string(),
        )));
    } else {
        tables
    };

    if target_tables.is_empty() {
        return Ok(RoutineSuccess::success(Message::new(
            "Truncate".to_string(),
            "No tables matched".to_string(),
        )));
    }

    match rows {
        None => truncate_all_rows(project, &target_tables).await?,
        Some(n) => delete_last_n_rows(project, &target_tables, n).await?,
    }

    Ok(RoutineSuccess::success(Message::new(
        "Truncate".to_string(),
        if rows.is_none() {
            format!("Truncated {} table(s)", target_tables.len())
        } else {
            format!("Deleted last N rows from {} table(s)", target_tables.len())
        },
    )))
}

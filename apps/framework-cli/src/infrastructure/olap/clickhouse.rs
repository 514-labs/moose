use clickhouse::Client;
use clickhouse_rs::ClientHandle;
use crypto_hash::{hex_digest, Algorithm};
use errors::ClickhouseError;
use itertools::Itertools;
use log::{debug, info};
use mapper::std_table_to_clickhouse_table;
use queries::{
    create_table_query, create_view_query, drop_table_query, drop_view_query, update_view_query,
    ClickhouseEngine,
};
use serde::{Deserialize, Serialize};

use crate::framework::core::infrastructure::table::{Column, ColumnType, Table};
use crate::framework::core::infrastructure::view::ViewType;
use crate::framework::core::infrastructure_map::{Change, OlapChange};
use crate::infrastructure::olap::clickhouse::model::{ClickHouseSystemTableRow, ClickHouseTable};
use crate::project::Project;

use self::config::ClickHouseConfig;
use self::model::ClickHouseSystemTable;

pub mod client;
pub mod config;
pub mod errors;
pub mod inserter;
pub mod mapper;
pub mod model;
pub mod queries;
pub mod version_sync;

pub type QueryString = String;

#[derive(Debug, thiserror::Error)]
pub enum ClickhouseChangesError {
    #[error("Error interacting with Clickhouse")]
    Clickhouse(#[from] ClickhouseError),

    #[error("Error interacting with Clickhouse")]
    ClickhouseClient(#[from] clickhouse::error::Error),

    #[error("Not Supported {0}")]
    NotSupported(String),
}

pub async fn execute_changes(
    project: &Project,
    changes: &[OlapChange],
) -> Result<(), ClickhouseChangesError> {
    // TODO refactor this to use the client we want to standardise on in the long term
    // This is currently using the same functions we are using in the current implementation
    let configured_client = create_client(project.clickhouse_config.clone());
    check_ready(&configured_client).await?;

    let db_name = &project.clickhouse_config.db_name;

    for change in changes.iter() {
        match change {
            OlapChange::Table(Change::Added(table)) => {
                log::info!("Creating table: {:?}", table.id());

                let clickhouse_table = std_table_to_clickhouse_table(table)?;
                let create_data_table_query =
                    create_table_query(db_name, clickhouse_table, ClickhouseEngine::MergeTree)?;
                run_query(&create_data_table_query, &configured_client).await?;
            }
            OlapChange::Table(Change::Removed(table)) => {
                log::info!("Removing table: {:?}", table.id());

                let clickhouse_table = std_table_to_clickhouse_table(table)?;
                let drop_query = drop_table_query(db_name, clickhouse_table)?;
                run_query(&drop_query, &configured_client).await?;
            }
            OlapChange::Table(Change::Updated { before, after }) => {
                log::info!("Updating table: {:?} to {:?}", before.id(), after.id());
                let table_diff = compute_table_diff(before, after);
                let alter_statements =
                    generate_alter_statements(&table_diff, db_name, &before.name);
                for statement in alter_statements {
                    run_query(&statement, &configured_client).await?;
                }
            }
            OlapChange::View(Change::Added(view)) => match &view.view_type {
                ViewType::TableAlias { source_table_name } => {
                    let create_view_query = create_view_query(
                        db_name,
                        &view.id(),
                        &format!("SELECT * FROM {}.{}", db_name, source_table_name),
                    )?;
                    run_query(&create_view_query, &configured_client).await?;
                }
            },
            OlapChange::View(Change::Removed(view)) => match &view.view_type {
                ViewType::TableAlias { .. } => {
                    let delete_view_query = drop_view_query(db_name, &view.id())?;
                    run_query(&delete_view_query, &configured_client).await?;
                }
            },
            OlapChange::View(Change::Updated { before, after }) => match &after.view_type {
                ViewType::TableAlias { source_table_name } => {
                    let update_view_query = update_view_query(
                        db_name,
                        &before.id(),
                        &format!("SELECT * FROM {}.{}", db_name, source_table_name),
                    )?;
                    run_query(&update_view_query, &configured_client).await?;
                }
            },
        }
    }

    Ok(())
}

pub struct ConfiguredDBClient {
    pub client: Client,
    pub config: ClickHouseConfig,
}

pub fn create_client(clickhouse_config: ClickHouseConfig) -> ConfiguredDBClient {
    let protocol = if clickhouse_config.use_ssl {
        "https"
    } else {
        "http"
    };
    ConfiguredDBClient {
        client: Client::default()
            .with_url(format!(
                "{}://{}:{}",
                protocol, clickhouse_config.host, clickhouse_config.host_port
            ))
            .with_user(clickhouse_config.user.to_string())
            .with_password(clickhouse_config.password.to_string())
            .with_database(clickhouse_config.db_name.to_string())
            .with_option("flatten_nested", "0"),
        config: clickhouse_config,
    }
}

// Run an arbitrary clickhouse query
pub async fn run_query(
    query: &String,
    configured_client: &ConfiguredDBClient,
) -> Result<(), clickhouse::error::Error> {
    debug!("Running query: {:?}", query);
    let client = &configured_client.client;
    client.query(query.as_str()).execute().await
}

pub async fn check_ready(
    configured_client: &ConfiguredDBClient,
) -> Result<(), clickhouse::error::Error> {
    let dummy_query = "SELECT version()".to_owned();
    crate::utilities::retry::retry(
        || run_query(&dummy_query, configured_client),
        |i, e| {
            i < 20
                && match e {
                    clickhouse::error::Error::Network(v) => {
                        let err_string = v.to_string();
                        debug!("Network error is {}", err_string);
                        err_string.contains("connection closed before message completed")
                            || err_string.contains("connection error: Connection reset by peer")
                            || err_string
                                .contains("operation was canceled: connection was not ready")
                            || err_string.contains("channel closed")
                    }
                    _ => {
                        debug!("Error is {} instead of network error. Will not retry.", e);
                        false
                    }
                }
        },
        tokio::time::Duration::from_millis(200),
    )
    .await
}

pub async fn fetch_all_tables(
    configured_client: &ConfiguredDBClient,
) -> Result<Vec<ClickHouseSystemTable>, clickhouse::error::Error> {
    let client = &configured_client.client;
    let db_name = &configured_client.config.db_name;

    // NOTE: The order of the columns in the query is important and must match the order of your struct fields.
    let query = "SELECT uuid, database, name, dependencies_table, engine FROM system.tables WHERE (database != 'information_schema') AND (database != 'INFORMATION_SCHEMA') AND (database != 'system')";

    debug!("<DCM> Fetching tables from: {:?}", db_name);

    let tables = client
        .query(query)
        .fetch_all::<ClickHouseSystemTableRow>()
        .await?
        .into_iter()
        .map(|row| row.to_table())
        .collect();

    debug!("<DCM> Fetched tables: {:?}", tables);

    Ok(tables)
}

pub async fn fetch_tables_with_version(
    configured_client: &ConfiguredDBClient,
    version: &str,
) -> Result<Vec<ClickHouseSystemTable>, clickhouse::error::Error> {
    let client = &configured_client.client;
    let db_name = &configured_client.config.db_name;

    let query = "SELECT uuid, database, name, dependencies_table, engine FROM system.tables WHERE database = ? AND name LIKE ?";

    let tables = client
        .query(query)
        .bind(db_name)
        .bind(version)
        .fetch_all::<ClickHouseSystemTableRow>()
        .await?
        .into_iter()
        .map(|row| row.to_table())
        .collect();

    Ok(tables)
}

pub async fn delete_table_or_view(
    table_or_view_name: &str,
    configured_client: &ConfiguredDBClient,
) -> Result<(), clickhouse::error::Error> {
    let client = &configured_client.client;
    let db_name = &configured_client.config.db_name;

    info!(
        "<DCM> Deleting table or view: {}.{}",
        db_name, table_or_view_name
    );

    client
        .query(format!("DROP TABLE \"{db_name}\".\"{table_or_view_name}\"").as_str())
        .execute()
        .await
}

pub async fn get_engine(
    db_name: &str,
    name: &str,
    configured_client: &ConfiguredDBClient,
) -> anyhow::Result<Option<String>> {
    let mut cursor = configured_client
        .client
        .query("SELECT engine FROM system.tables WHERE database = ? AND name = ?")
        .bind(db_name)
        .bind(name)
        .fetch::<String>()?;

    Ok(cursor.next().await?)
}

pub async fn check_is_table_new(
    table: &ClickHouseTable,
    configured_client: &ConfiguredDBClient,
) -> Result<bool, clickhouse::error::Error> {
    let client = &configured_client.client;

    info!("<DCM> Checking if {} table is new", table.name.clone());
    let result = client
        .query("select engine, total_rows from system.tables where database = ? AND name = ?")
        .bind(configured_client.config.db_name.clone())
        .bind(table.name.clone())
        .fetch_all::<TableDetail>()
        .await?;

    match result.len() {
        // i keep getting 2 rows when I have this logic in the select query
        1 => Ok(result[0].engine != "View" && result[0].total_rows == Some(0)),
        _ => panic!("Expected 1 result, got {:?}", result),
    }
}

pub async fn check_table_size(
    table_name: &str,
    config: &ClickHouseConfig,
    clickhouse: &mut ClientHandle,
) -> Result<i64, clickhouse_rs::errors::Error> {
    info!("<DCM> Checking size of {} table", table_name);
    let result = clickhouse
        .query(&format!(
            "select count(*) from \"{}\".\"{}\"",
            config.db_name.clone(),
            table_name
        ))
        .fetch_all()
        .await?;
    let rows = result.rows().collect_vec();

    let result: u64 = match rows.len() {
        1 => rows[0].get(0)?,
        _ => panic!("Expected 1 result, got {:?}", rows.len()),
    };
    Ok(result as i64)
}

pub async fn fetch_table_names(
    configured_client: &ConfiguredDBClient,
) -> Result<Vec<String>, clickhouse::error::Error> {
    let client = &configured_client.client;
    let db_name = &configured_client.config.db_name;

    debug!("Fetching tables from: {:?}", db_name);

    let query = format!("SELECT name FROM system.tables WHERE (database = '{db_name}')");
    let mut cursor = client.query(query.as_str()).fetch::<String>()?;
    let mut tables = vec![];

    while let Some(name) = cursor.next().await? {
        tables.push(name);
    }

    tables.sort();

    debug!("Fetched tables: {:?}", tables);

    Ok(tables)
}

pub async fn fetch_table_schema(
    configured_client: &ConfiguredDBClient,
    table_name: &str,
) -> Result<Vec<(String, String)>, clickhouse::error::Error> {
    let client = &configured_client.client;
    let db_name = &configured_client.config.db_name;

    debug!("Fetching columns from: {:?}", table_name);

    let query = format!("SELECT name, type FROM system.columns WHERE (database = '{db_name}' AND table = '{table_name}')");
    let mut cursor = client.query(query.as_str()).fetch::<(String, String)>()?;

    let mut columns = vec![];

    while let Some((name, column_type)) = cursor.next().await? {
        columns.push((name, column_type));
    }

    columns.sort();

    debug!("Fetched columns: {:?}", columns);

    Ok(columns)
}

pub fn table_schema_to_hash(
    columns: Vec<(String, String)>,
) -> Result<String, clickhouse::error::Error> {
    let data = columns
        .iter()
        .map(|(name, column_type)| format!("{}{}", name, column_type))
        .collect::<Vec<String>>()
        .join("");

    let hashed = hex_digest(Algorithm::SHA256, data.as_bytes());

    Ok(hashed)
}

#[derive(Debug, Clone, Deserialize, Serialize, clickhouse::Row)]
struct TableDetail {
    pub engine: String,
    pub total_rows: Option<u64>,
}

#[derive(Debug, Clone)]
enum TableDiff {
    AddColumn(Column),
    DropColumn(String),
    ModifyColumn { name: String, new_type: ColumnType },
    ModifyOrderBy(Vec<String>),
}

fn compute_table_diff(before: &Table, after: &Table) -> Vec<TableDiff> {
    let mut diff = Vec::new();

    // Check for added or modified columns
    for after_col in &after.columns {
        match before.columns.iter().find(|c| c.name == after_col.name) {
            // If the column is in the before table, but different, then it is modified
            Some(before_col) if before_col != after_col => {
                diff.push(TableDiff::ModifyColumn {
                    name: after_col.name.clone(),
                    new_type: after_col.data_type.clone(),
                });
            }
            // If the column is not in the before table, then it is added
            None => {
                diff.push(TableDiff::AddColumn(after_col.clone()));
            }
            _ => {}
        }
    }

    // Check for dropped columns
    for before_col in &before.columns {
        if !after.columns.iter().any(|c| c.name == before_col.name) {
            diff.push(TableDiff::DropColumn(before_col.name.clone()));
        }
    }

    // Check for changes in ORDER BY
    if before.order_by != after.order_by {
        diff.push(TableDiff::ModifyOrderBy(after.order_by.clone()));
    }

    diff
}

fn generate_alter_statements(diff: &[TableDiff], db_name: &str, table_name: &str) -> Vec<String> {
    diff.iter()
        .map(|d| match d {
            TableDiff::AddColumn(col) => {
                format!(
                    "ALTER TABLE `{}`.`{}` ADD COLUMN `{}` {}",
                    db_name, table_name, col.name, col.data_type
                )
            }
            TableDiff::DropColumn(name) => {
                format!(
                    "ALTER TABLE `{}`.`{}` DROP COLUMN `{}`",
                    db_name, table_name, name
                )
            }
            TableDiff::ModifyColumn { name, new_type } => {
                format!(
                    "ALTER TABLE `{}`.`{}` MODIFY COLUMN `{}` {}",
                    db_name, table_name, name, new_type
                )
            }
            TableDiff::ModifyOrderBy(new_order) => {
                format!(
                    "ALTER TABLE `{}`.`{}` MODIFY ORDER BY ({})",
                    db_name,
                    table_name,
                    new_order.iter().map(|s| format!("`{}`", s)).join(", ")
                )
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framework::core::{
        infrastructure::table::{Column, ColumnType, Table},
        infrastructure_map::PrimitiveSignature,
    };

    #[test]
    fn test_compute_table_diff() {
        let before = Table {
            name: "test_table".to_string(),
            columns: vec![
                Column {
                    name: "id".to_string(),
                    data_type: ColumnType::Int,
                    required: true,
                    unique: true,
                    primary_key: true,
                    default: None,
                },
                Column {
                    name: "name".to_string(),
                    data_type: ColumnType::String,
                    required: true,
                    unique: false,
                    primary_key: false,
                    default: None,
                },
                Column {
                    name: "to_be_removed".to_string(),
                    data_type: ColumnType::String,
                    required: false,
                    unique: false,
                    primary_key: false,
                    default: None,
                },
            ],
            order_by: vec!["id".to_string()],
            version: "1.0".to_string(),
            source_primitive: PrimitiveSignature::default(),
        };

        let after = Table {
            name: "test_table".to_string(),
            columns: vec![
                Column {
                    name: "id".to_string(),
                    data_type: ColumnType::BigInt, // Changed type
                    required: true,
                    unique: true,
                    primary_key: true,
                    default: None,
                },
                Column {
                    name: "name".to_string(),
                    data_type: ColumnType::String,
                    required: true,
                    unique: false,
                    primary_key: false,
                    default: None,
                },
                Column {
                    name: "age".to_string(), // New column
                    data_type: ColumnType::Int,
                    required: false,
                    unique: false,
                    primary_key: false,
                    default: None,
                },
            ],
            order_by: vec!["id".to_string(), "name".to_string()], // Changed order_by
            version: "1.1".to_string(),
            source_primitive: PrimitiveSignature::default(),
        };

        let diff = compute_table_diff(&before, &after);

        assert_eq!(diff.len(), 4);
        assert!(
            matches!(&diff[0], TableDiff::ModifyColumn { name, new_type } if name == "id" && matches!(new_type, ColumnType::BigInt))
        );
        assert!(matches!(&diff[1], TableDiff::AddColumn(col) if col.name == "age"));
        assert!(matches!(&diff[2], TableDiff::DropColumn(name) if name == "to_be_removed"));
        assert!(
            matches!(&diff[3], TableDiff::ModifyOrderBy(order_by) if *order_by == vec!["id".to_string(), "name".to_string()])
        );
    }

    #[test]
    fn test_generate_alter_statements() {
        let diff = vec![
            TableDiff::AddColumn(Column {
                name: "age".to_string(),
                data_type: ColumnType::Int,
                required: false,
                unique: false,
                primary_key: false,
                default: None,
            }),
            TableDiff::DropColumn("old_column".to_string()),
            TableDiff::ModifyColumn {
                name: "id".to_string(),
                new_type: ColumnType::BigInt,
            },
            TableDiff::ModifyOrderBy(vec!["id".to_string(), "name".to_string()]),
        ];

        let statements = generate_alter_statements(&diff, "test_db", "test_table");

        assert_eq!(statements.len(), 4);
        assert_eq!(
            statements[0],
            "ALTER TABLE `test_db`.`test_table` ADD COLUMN `age` Int"
        );
        assert_eq!(
            statements[1],
            "ALTER TABLE `test_db`.`test_table` DROP COLUMN `old_column`"
        );
        assert_eq!(
            statements[2],
            "ALTER TABLE `test_db`.`test_table` MODIFY COLUMN `id` BigInt"
        );
        assert_eq!(
            statements[3],
            "ALTER TABLE `test_db`.`test_table` MODIFY ORDER BY (`id`, `name`)"
        );
    }
}

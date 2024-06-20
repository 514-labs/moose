use clickhouse::Client;
use crypto_hash::{hex_digest, Algorithm};
use log::{debug, info};
use serde::{Deserialize, Serialize};

use crate::infrastructure::olap::clickhouse::model::{ClickHouseSystemTableRow, ClickHouseTable};

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

#[cfg(test)]
mod tests {
    use crate::infrastructure::olap::clickhouse::model::{
        ClickHouseColumn, ClickHouseColumnType, ClickHouseInt,
    };
    use crate::infrastructure::olap::clickhouse::version_sync::VersionSync;

    #[test]
    fn test_version_sync_function_generation() {
        let columns = vec![
            ClickHouseColumn {
                name: "eventId".to_string(),
                column_type: ClickHouseColumnType::String,
                required: true,
                unique: false,
                primary_key: true,
                default: None,
            },
            ClickHouseColumn {
                name: "timestamp".to_string(),
                column_type: ClickHouseColumnType::DateTime,
                required: true,
                unique: false,
                primary_key: false,
                default: None,
            },
            ClickHouseColumn {
                name: "userId".to_string(),
                column_type: ClickHouseColumnType::String,
                required: true,
                unique: false,
                primary_key: false,
                default: None,
            },
            ClickHouseColumn {
                name: "activity".to_string(),
                column_type: ClickHouseColumnType::String,
                required: true,
                unique: false,
                primary_key: false,
                default: None,
            },
        ];
        assert_eq!(
            VersionSync::generate_migration_function(&columns[0..3], &columns),
            "(eventId, timestamp, userId) -> (eventId, timestamp, userId, 'activity')"
        );

        let mut no_timestamp = columns.clone();
        no_timestamp.remove(1);
        assert_eq!(
            VersionSync::generate_migration_function(&no_timestamp, &columns),
            "(eventId, userId, activity) -> (eventId, '2024-02-20T23:14:57.788Z', userId, activity)"
        );

        let mut int_user_id = columns.clone();
        int_user_id[2].column_type = ClickHouseColumnType::ClickhouseInt(ClickHouseInt::Int32);
        assert_eq!(
            VersionSync::generate_migration_function(&columns, &int_user_id),
            "(eventId, timestamp, userId, activity) -> (eventId, timestamp, 0, activity)"
        );
    }
}

pub type QueryString = String;

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
        .query(format!("DROP TABLE {db_name}.{table_or_view_name}").as_str())
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
    table: &ClickHouseTable,
    configured_client: &ConfiguredDBClient,
) -> Result<i64, clickhouse::error::Error> {
    let client = &configured_client.client;

    info!("<DCM> Checking size of {} table", table.name.clone());
    let result: Vec<i64> = client
        .query(&format!(
            "select count(*) from {}.{}",
            configured_client.config.db_name.clone(),
            table.name
        ))
        .fetch_all::<i64>()
        .await?;

    match result.len() {
        1 => Ok(result[0]),
        _ => panic!("Expected 1 result, got {:?}", result),
    }
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

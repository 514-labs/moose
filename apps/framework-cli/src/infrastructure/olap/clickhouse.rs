mod client;
pub mod config;
pub mod mapper;
pub mod model;
mod queries;

use clickhouse::inserter::Inserter;
use clickhouse::Client;
use clickhouse::Row;

use lazy_static::lazy_static;
use log::debug;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::infrastructure::olap::clickhouse::model::ClickHouseSystemTableRow;
use crate::infrastructure::olap::clickhouse::queries::CreateVersionSyncTriggerQuery;

use self::config::ClickhouseConfig;
use self::model::ClickHouseSystemTable;
use self::model::ClickHouseTable;

#[derive(Debug, Clone)]
pub struct VersionSync {
    pub db_name: String,
    pub model_name: String,
    pub source_version: String,
    pub source_table: ClickHouseTable,
    pub dest_version: String,
    pub dest_table: ClickHouseTable,
    pub migration_function: String,
}

lazy_static! {
    pub static ref VERSION_SYNC_REGEX: Regex =
        //            source_model_name         source     target_model_name   dest_version
        Regex::new(r"^([a-zA-Z0-9_]+)_migrate__([0-9_]+)__(([a-zA-Z0-9_]+)__)?([0-9_]+).sql$")
            .unwrap();
}

impl VersionSync {
    fn migration_function_name(&self) -> String {
        format!(
            "{}_migrate__{}__{}",
            self.model_name,
            self.source_version.replace('.', "_"),
            self.dest_version.replace('.', "_"),
        )
    }

    fn migration_trigger_name(&self) -> String {
        format!(
            "{}_trigger__{}__{}",
            self.model_name,
            self.source_version.replace('.', "_"),
            self.dest_version.replace('.', "_"),
        )
    }

    pub fn create_function_query(&self) -> String {
        format!(
            "CREATE FUNCTION {} AS {}",
            self.migration_function_name(),
            self.migration_function
        )
    }
    pub fn drop_function_query(&self) -> String {
        format!("DROP FUNCTION IF EXISTS {}", self.migration_function_name())
    }

    pub fn create_trigger_query(self) -> String {
        CreateVersionSyncTriggerQuery::build(self)
    }

    pub fn drop_trigger_query(&self) -> String {
        format!(
            "DROP VIEW IF EXISTS {}.{}",
            self.db_name,
            self.migration_trigger_name()
        )
    }
}

pub type QueryString = String;

pub struct ConfiguredDBClient {
    pub client: Client,
    pub config: ClickhouseConfig,
}

pub fn create_client(clickhouse_config: ClickhouseConfig) -> ConfiguredDBClient {
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
            .with_database(clickhouse_config.db_name.to_string()),
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

    debug!("Fetching tables from: {:?}", db_name);

    let mut cursor = client.query(query).fetch::<ClickHouseSystemTableRow>()?;

    let mut tables = vec![];

    while let Some(row) = cursor.next().await? {
        tables.push(row.to_table());
    }

    debug!("Fetched tables: {:?}", tables);

    Ok(tables)
}

pub async fn delete_table_or_view(
    table_or_view_name: String,
    configured_client: &ConfiguredDBClient,
) -> Result<(), clickhouse::error::Error> {
    let client = &configured_client.client;
    let db_name = &configured_client.config.db_name;

    client
        .query(format!("DROP TABLE {db_name}.{table_or_view_name}").as_str())
        .execute()
        .await
}

#[derive(Debug, Row, Serialize, Deserialize)]
pub struct GenericRow {
    name: String,
}

pub fn create_inserter(
    clickhouse_config: ClickhouseConfig,
    table_name: &str,
) -> anyhow::Result<Inserter<GenericRow>> {
    let configure_client = create_client(clickhouse_config);

    let inserter = configure_client.client.inserter(table_name)?;

    Ok(inserter
        .with_timeouts(Some(Duration::from_secs(1)), Some(Duration::from_secs(20)))
        .with_max_entries(750_000)
        .with_period(Some(Duration::from_secs(15))))
}

#[tokio::test]
async fn test_inserter() {
    let clickhouse_config = ClickhouseConfig {
        user: "panda".to_string(),
        password: "pandapass".to_string(),
        host: "localhost".to_string(),
        use_ssl: false,
        postgres_port: 5432,
        kafka_port: 9092,
        host_port: 18123,
        db_name: "local".to_string(),
    };

    let mut inserter = create_inserter(clickhouse_config, "test_table").unwrap();

    inserter
        .write(&GenericRow {
            name: "test".to_string(),
        })
        .await
        .unwrap();

    inserter.commit().await.unwrap();
    inserter.end().await.unwrap();
}

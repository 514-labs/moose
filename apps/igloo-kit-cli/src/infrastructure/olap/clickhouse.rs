pub mod config;
pub mod mapper;
mod queries;

use std::fmt::{self};

use clickhouse::Client;
use reqwest::Url;
use schema_ast::ast::FieldArity;

use crate::framework::schema::{MatViewOps, TableOps, UnsupportedDataTypeError};

use self::{
    config::ClickhouseConfig,
    queries::{
        CreateMaterializedViewQuery, CreateTableQuery, DropMaterializedViewQuery, DropTableQuery,
    },
};

#[derive(Debug, Clone)]
pub enum ClickhouseTableType {
    Table,
    View,
    MaterializedView,
    Unsupported,
}

impl fmt::Display for ClickhouseTableType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone)]
pub enum ClickhouseColumnType {
    String,
    Boolean,
    ClickhouseInt(ClickhouseInt),
    ClickhouseFloat(ClickhouseFloat),
    Decimal,
    DateTime,
    Json,
    Bytes,
    Unsupported,
}

impl fmt::Display for ClickhouseColumnType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone)]
pub enum ClickhouseInt {
    Int8,
    Int16,
    Int32,
    Int64,
    Int128,
    Int256,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    UInt128,
    UInt256,
}

impl fmt::Display for ClickhouseInt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone)]
pub enum ClickhouseFloat {
    Float32,
    Float64,
}

impl fmt::Display for ClickhouseFloat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone)]
pub enum ClickhouseColumnDefaults {
    Now,
}

#[derive(Debug, Clone)]
pub struct ClickhouseColumn {
    pub name: String,
    pub column_type: ClickhouseColumnType,
    pub arity: FieldArity,
    pub unique: bool,
    pub primary_key: bool,
    pub default: Option<ClickhouseColumnDefaults>,
}

#[derive(Debug, Clone)]
pub struct ClickhouseTable {
    pub db_name: String,
    pub name: String,
    pub columns: Vec<ClickhouseColumn>,
    pub table_type: ClickhouseTableType,
}

impl ClickhouseTable {
    pub fn new(
        db_name: String,
        name: String,
        columns: Vec<ClickhouseColumn>,
        table_type: ClickhouseTableType,
    ) -> ClickhouseTable {
        ClickhouseTable {
            db_name,
            name,
            columns,
            table_type,
        }
    }
}

impl TableOps for ClickhouseTable {
    fn create_table_query(&self) -> Result<String, UnsupportedDataTypeError> {
        CreateTableQuery::new(
            self.clone(),
            "redpanda-1".to_string(),
            9092,
            self.name.clone(),
        )
    }

    fn drop_table_query(&self) -> Result<String, UnsupportedDataTypeError> {
        DropTableQuery::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct ClickhouseView {
    pub db_name: String,
    pub name: String,
    pub source_table: ClickhouseTable,
}

impl ClickhouseView {
    pub fn new(db_name: String, name: String, source_table: ClickhouseTable) -> ClickhouseView {
        ClickhouseView {
            db_name,
            name,
            source_table,
        }
    }
}

pub type QueryString = String;

impl MatViewOps for ClickhouseView {
    fn create_materialized_view_query(&self) -> Result<QueryString, UnsupportedDataTypeError> {
        CreateMaterializedViewQuery::new(self.clone())
    }
    fn drop_materialized_view_query(&self) -> Result<QueryString, UnsupportedDataTypeError> {
        DropMaterializedViewQuery::new(self.clone())
    }
}

pub struct ConfiguredDBClient {
    pub client: Client,
    pub config: ClickhouseConfig,
}

pub fn create_client(clickhouse_config: ClickhouseConfig) -> ConfiguredDBClient {
    ConfiguredDBClient {
        client: Client::default()
            .with_url(
                Url::parse(&format!(
                    "http://{}:{}",
                    clickhouse_config.host, clickhouse_config.host_port
                ))
                .unwrap(),
            )
            .with_user(format!("{}", clickhouse_config.user))
            .with_password(format!("{}", clickhouse_config.password))
            .with_database(format!("{}", clickhouse_config.db_name)),
        config: clickhouse_config,
    }
}

// Run an arbitrary clickhouse query
pub async fn run_query(
    query: QueryString,
    configured_client: &ConfiguredDBClient,
) -> Result<(), clickhouse::error::Error> {
    let client = &configured_client.client;
    client.query(query.as_str()).execute().await
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

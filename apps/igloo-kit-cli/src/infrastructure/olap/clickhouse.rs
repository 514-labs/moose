pub mod config;
pub mod mapper;
mod queries;

use std::fmt;

use clickhouse::Client;
use reqwest::Url;
use schema_ast::ast::FieldArity;

use crate::framework::schema::{OpsTable, UnsupportedDataTypeError};

use self::{queries::{CreateTableQuery, DropTableQuery}, config::ClickhouseConfig};

#[derive(Debug, Clone)]
pub enum ClickhouseTableType {
    Table,
    View,
    MaterializedView,
    Unsupported
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
    pub default: Option<ClickhouseColumnDefaults>
}

#[derive(Debug, Clone)]
pub struct ClickhouseTable {
    pub db_name: String,
    pub name: String,
    pub columns: Vec<ClickhouseColumn>,
    pub table_type: ClickhouseTableType,
}

impl ClickhouseTable {
    pub fn new(db_name: String, name: String, columns: Vec<ClickhouseColumn>, table_type: ClickhouseTableType) -> ClickhouseTable {
        ClickhouseTable {
            db_name,
            name,
            columns,
            table_type,
        }
    }
}

impl OpsTable for ClickhouseTable {
    fn create_table_query(&self) -> Result<String, UnsupportedDataTypeError> {
        CreateTableQuery::new(self.clone(), "redpanda-1".to_string(), 9092, self.name.clone())
    }

    fn drop_table_query(&self) -> Result<String, UnsupportedDataTypeError> {
        DropTableQuery::new(self.clone())
    }
}

pub struct ConfiguredClient {
    pub client: Client,
    pub config: ClickhouseConfig,
}


pub fn create_client(clickhouse_config: ClickhouseConfig) -> ConfiguredClient {
    ConfiguredClient {
        client: Client::default()
        .with_url(Url::parse(&format!("http://{}:{}", clickhouse_config.host, clickhouse_config.host_port)).unwrap())
        .with_user(format!("{}", clickhouse_config.user))
        .with_password(format!("{}", clickhouse_config.password))
        .with_database(format!("{}", clickhouse_config.db_name)),
        config: clickhouse_config,
    }    
}

// Run an arbitrary clickhouse query
pub async fn run_query(query: String, configured_client: &ConfiguredClient) -> Result<(), clickhouse::error::Error> {
    let client = &configured_client.client;
    client.query(query.as_str()).execute().await
}

// Creates a table in clickhouse from a file name. this table should have a single field that accepts a json blob
pub async fn create_table(table_name: String, topic: String, configured_client: &ConfiguredClient) -> Result<(), clickhouse::error::Error> {
    let client = &configured_client.client;
    let config = &configured_client.config;
    let db_name = &config.db_name;
    let cluster_network = &config.cluster_network;
    let kafka_port = &config.kafka_port;

    // If you want to change the settings when doing a query you can do it as follows: SETTINGS allow_experimental_object_type = 1;
    client.query(format!("CREATE TABLE IF NOT EXISTS {db_name}.{table_name} 
        (data String) ENGINE = Kafka('{cluster_network}:{kafka_port}', '{topic}', 'clickhouse-group', 'JSONEachRow') ;")
        .as_str() ).execute().await
}

pub async fn delete_table(table_name: String, configured_client: &ConfiguredClient) -> Result<(), clickhouse::error::Error> {
    let client = &configured_client.client;
    let db_name = &configured_client.config.db_name;

    client.query(format!("DROP TABLE {db_name}.{table_name}").as_str()).execute().await
}

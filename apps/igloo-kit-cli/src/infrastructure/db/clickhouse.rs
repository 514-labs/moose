use std::fmt;

use clickhouse::Client;
use diagnostics::Diagnostics;
use reqwest::Url;
use schema_ast::ast::{SchemaAst, Top, WithName, Field};

#[derive(Clone)]
pub struct ClickhouseConfig {
    pub db_name: String, // ex. local
    pub user: String,
    pub password: String,
    pub host: String, // ex. localhost
    pub host_port: i32, // ex. 18123
    pub postgres_port: i32, // ex. 9005
    pub kafka_port: i32, // ex. 9092
    pub cluster_network: String, // ex. panda-house
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

#[derive(Debug, Clone)]
pub struct Table {
    pub name: String,
    pub columns: Vec<Column>,
}

#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub data_type: String,
}
#[derive(Debug, Clone)]
pub struct UnsupportedDataTypeError {
    type_name: String,
}

impl fmt::Display for UnsupportedDataTypeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "The following type is unsupported: {}", self.type_name)
    }
}


fn field_to_column(f: &Field) -> Result<Column, UnsupportedDataTypeError> {
    match &f.field_type {
        schema_ast::ast::FieldType::Supported(ft) => {
            Ok(Column {
                name: f.name().to_string(),
                data_type: ft.name.to_string(),
            })
        },
        schema_ast::ast::FieldType::Unsupported(x, _) => {
            Err(UnsupportedDataTypeError{type_name: x.to_string()})
        }
    }
}

fn top_to_table(t: &Top) -> Result<Table, UnsupportedDataTypeError> {
    match t {
        Top::Model(m) => {
            let table_name = m.name().to_string();

            let columns: Result<Vec<Column>, UnsupportedDataTypeError> = m.iter_fields().map(|(_id, f)| {
                field_to_column(f)
            }).collect();
            
            Ok(Table {
                name: table_name,
                columns: columns?
            })
        }
        _ => { 
            Err(UnsupportedDataTypeError {type_name: "anything that isn't a model".to_string()})
        }
    }
}

pub fn ast_mapper(ast: SchemaAst) -> Result<Vec<Table>, UnsupportedDataTypeError>  {
    ast.iter_tops().map(|(id, t)| {
        top_to_table(t)
    }).collect::<Result<Vec<Table>, UnsupportedDataTypeError>>()
}
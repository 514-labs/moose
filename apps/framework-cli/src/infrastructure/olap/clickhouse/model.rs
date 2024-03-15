use super::queries::{
    CreateKafkaTriggerViewQuery, CreateTableQuery, DropMaterializedViewQuery, DropTableQuery,
};
use crate::framework::schema::DataEnum;
use crate::infrastructure::olap::clickhouse::queries::ClickhouseEngine;
use crate::{
    framework::schema::{FieldArity, UnsupportedDataTypeError},
    utilities::constants::REDPANDA_CONTAINER_NAME,
};

use serde::{Deserialize, Serialize};
use std::fmt::{self};

#[derive(Debug, Clone)]
pub enum ClickHouseTableType {
    Table,
    View,
    MaterializedView,
    Unsupported,
}

impl fmt::Display for ClickHouseTableType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone)]
pub enum ClickHouseColumnType {
    String,
    Boolean,
    ClickhouseInt(ClickHouseInt),
    ClickhouseFloat(ClickHouseFloat),
    Decimal,
    DateTime,
    Json,
    Bytes,
    Enum(DataEnum),
    Unsupported,
}

impl fmt::Display for ClickHouseColumnType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone)]
pub enum ClickHouseInt {
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

impl fmt::Display for ClickHouseInt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone)]
pub enum ClickHouseFloat {
    Float32,
    Float64,
}

impl fmt::Display for ClickHouseFloat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone)]
pub enum ClickHouseColumnDefaults {
    Now,
}

#[derive(Debug, Clone)]
pub struct ClickHouseColumn {
    pub name: String,
    pub column_type: ClickHouseColumnType,
    pub arity: FieldArity,
    pub unique: bool,
    pub primary_key: bool,
    pub default: Option<ClickHouseColumnDefaults>,
}

pub struct ClickHouseRecord {
    pub columns: Vec<String>,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, clickhouse::Row)]
pub struct ClickHouseSystemTableRow {
    #[serde(with = "clickhouse::serde::uuid")]
    pub uuid: uuid::Uuid,
    pub database: String,
    pub name: String,
    pub dependencies_table: Vec<String>,
    pub engine: String,
}

impl ClickHouseSystemTableRow {
    pub fn to_table(&self) -> ClickHouseSystemTable {
        ClickHouseSystemTable {
            uuid: self.uuid.to_string(),
            database: self.database.to_string(),
            name: self.name.to_string(),
            dependencies_table: self.dependencies_table.to_vec(),
            engine: self.engine.to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, clickhouse::Row)]
pub struct ClickHouseSystemTable {
    pub uuid: String,
    pub database: String,
    pub name: String,
    pub dependencies_table: Vec<String>,
    pub engine: String,
}

#[derive(Debug, Clone)]
pub struct ClickHouseTable {
    pub db_name: String,
    pub name: String,
    pub columns: Vec<ClickHouseColumn>,
    pub table_type: ClickHouseTableType,
}

impl ClickHouseTable {
    pub fn new(
        db_name: String,
        name: String,
        columns: Vec<ClickHouseColumn>,
        table_type: ClickHouseTableType,
    ) -> ClickHouseTable {
        ClickHouseTable {
            db_name,
            name,
            columns,
            table_type,
        }
    }

    pub fn kafka_table_name(&self) -> String {
        format!("{}_kafka", self.name)
    }
    pub fn view_name(&self) -> String {
        format!("{}_trigger", self.name)
    }

    fn kafka_table(&self) -> ClickHouseTable {
        ClickHouseTable {
            name: self.kafka_table_name(),
            ..self.clone()
        }
    }

    pub fn create_kafka_table_query(
        &self,
        project_name: &str,
    ) -> Result<String, UnsupportedDataTypeError> {
        CreateTableQuery::kafka(
            self.kafka_table(),
            format!("{}-{}", project_name, REDPANDA_CONTAINER_NAME),
            9092,
            self.name.clone(),
        )
    }
    pub fn create_data_table_query(&self) -> Result<String, UnsupportedDataTypeError> {
        CreateTableQuery::build(self.clone(), ClickhouseEngine::MergeTree)
    }

    pub fn drop_kafka_table_query(&self) -> Result<String, UnsupportedDataTypeError> {
        DropTableQuery::build(self.kafka_table())
    }

    pub fn drop_data_table_query(&self) -> Result<String, UnsupportedDataTypeError> {
        DropTableQuery::build(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct ClickHouseKafkaTrigger {
    pub db_name: String,
    pub name: String,
    pub source_table_name: String,
    pub dest_table_name: String,
}

impl ClickHouseKafkaTrigger {
    pub fn new(
        db_name: String,
        name: String,
        source_table_name: String,
        dest_table_name: String,
    ) -> ClickHouseKafkaTrigger {
        ClickHouseKafkaTrigger {
            db_name,
            name,
            source_table_name,
            dest_table_name,
        }
    }

    pub fn from_clickhouse_table(table: &ClickHouseTable) -> ClickHouseKafkaTrigger {
        ClickHouseKafkaTrigger {
            db_name: table.db_name.clone(),
            name: table.view_name(),
            source_table_name: table.kafka_table_name(),
            dest_table_name: table.name.clone(),
        }
    }

    pub fn create_materialized_view_query(&self) -> Result<String, UnsupportedDataTypeError> {
        Ok(CreateKafkaTriggerViewQuery::build(self.clone()))
    }
    pub fn drop_materialized_view_query(&self) -> Result<String, UnsupportedDataTypeError> {
        DropMaterializedViewQuery::build(self.clone())
    }
}

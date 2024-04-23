use super::queries::{CreateTableQuery, DropTableQuery};
use crate::framework::schema::DataEnum;
use crate::framework::schema::{FieldArity, UnsupportedDataTypeError};
use crate::infrastructure::olap::clickhouse::queries::ClickhouseEngine;

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

#[derive(Debug, Clone, Eq, PartialEq)]
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

#[derive(Debug, Clone, Eq, PartialEq)]
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

#[derive(Debug, Clone, Eq, PartialEq)]
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

pub struct ClickHouseValue {
    pub value_type: ClickHouseColumnType,

    // This is a string right now because that's the value we send over the wire with the HTTP protocol
    // if we used the RowBinary // https://clickhouse.yandex/docs/en/query_language/syntax/#syntax-identifiers
    // or another format, we could optimize
    value: String,
}

const NULL: &str = "NULL";

// TODO - add support for Decimal, Json, Bytes, Enum
impl ClickHouseValue {
    pub fn new_null() -> ClickHouseValue {
        ClickHouseValue {
            value_type: ClickHouseColumnType::String,
            value: NULL.to_string(),
        }
    }

    pub fn new_string(value: String) -> ClickHouseValue {
        ClickHouseValue {
            value_type: ClickHouseColumnType::String,
            value,
        }
    }

    pub fn new_boolean(value: bool) -> ClickHouseValue {
        ClickHouseValue {
            value_type: ClickHouseColumnType::Boolean,
            value: format!("{}", value),
        }
    }

    pub fn new_int_64(value: i64) -> ClickHouseValue {
        ClickHouseValue {
            value_type: ClickHouseColumnType::ClickhouseInt(ClickHouseInt::Int64),
            value: format!("{}", value),
        }
    }

    pub fn new_float_64(value: f64) -> ClickHouseValue {
        ClickHouseValue {
            value_type: ClickHouseColumnType::ClickhouseFloat(ClickHouseFloat::Float64),
            value: format!("{}", value),
        }
    }

    pub fn new_date_time(value: DateTime<FixedOffset>) -> ClickHouseValue {
        ClickHouseValue {
            value_type: ClickHouseColumnType::DateTime,
            value: format!("{}", value.format("%Y-%m-%d %H:%M:%S")),
        }
    }
}

impl fmt::Display for ClickHouseValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.value_type {
            ClickHouseColumnType::String => write!(f, "'{}'", &self.value),
            ClickHouseColumnType::Boolean => write!(f, "{}", &self.value),
            ClickHouseColumnType::ClickhouseInt(_) => {
                write!(f, "{}", &self.value)
            }
            ClickHouseColumnType::ClickhouseFloat(_) => {
                write!(f, "{}", &self.value)
            }
            ClickHouseColumnType::DateTime => write!(f, "'{}'", &self.value),
            _ => Err(std::fmt::Error),
        }
    }
}

pub struct ClickHouseRecord {
    values: HashMap<String, ClickHouseValue>,
}

impl Default for ClickHouseRecord {
    fn default() -> Self {
        Self::new()
    }
}

impl ClickHouseRecord {
    pub fn new() -> ClickHouseRecord {
        ClickHouseRecord {
            values: HashMap::new(),
        }
    }

    pub fn insert(&mut self, column: String, value: ClickHouseValue) {
        self.values.insert(sanitize_column_name(column), value);
    }

    pub fn get(&self, column: &str) -> Option<&ClickHouseValue> {
        self.values.get(column)
    }
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

    pub fn create_data_table_query(&self) -> Result<String, UnsupportedDataTypeError> {
        CreateTableQuery::build(self.clone(), ClickhouseEngine::MergeTree)
    }

    pub fn drop_data_table_query(&self) -> Result<String, UnsupportedDataTypeError> {
        DropTableQuery::build(self.clone())
    }
}

pub fn sanitize_column_name(name: String) -> String {
    name.replace([' ', '-'], "_")
}

use super::errors::ClickhouseError;
use super::queries::{create_table_query, drop_table_query};
use crate::framework::data_model::schema::DataEnum;
use crate::infrastructure::olap::clickhouse::queries::ClickhouseEngine;
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

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
pub struct ClickHouseNested {
    name: String,
    columns: Vec<ClickHouseColumn>,
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
    Array(Box<ClickHouseColumnType>),
    Enum(DataEnum),
    Nested(Vec<ClickHouseColumn>),
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ClickHouseColumnDefaults {
    Now,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ClickHouseColumn {
    pub name: String,
    pub column_type: ClickHouseColumnType,
    pub required: bool,
    pub unique: bool,
    pub primary_key: bool,
    pub default: Option<ClickHouseColumnDefaults>,
}

impl ClickHouseColumn {
    pub fn is_array(&self) -> bool {
        matches!(&self.column_type, ClickHouseColumnType::Array(_))
    }
}

pub enum ClickHouseRuntimeEnum {
    ClickHouseInt(u8),
    ClickHouseString(String),
}

#[derive(Debug, Clone)]
pub enum ClickHouseValue {
    String(String),
    Boolean(String),
    ClickhouseInt(String),
    ClickhouseFloat(String),
    Decimal,
    DateTime(String),
    Json,
    Bytes,
    Array(Vec<ClickHouseValue>),
    Enum(String),
    Nested(Vec<ClickHouseValue>),
    Null(String),
}

const NULL: &str = "NULL";

// TODO - add support for Decimal, Json, Bytes
impl ClickHouseValue {
    pub fn new_null() -> ClickHouseValue {
        ClickHouseValue::Null(NULL.to_string())
    }

    pub fn new_string(value: String) -> ClickHouseValue {
        ClickHouseValue::String(value)
    }

    pub fn new_boolean(value: bool) -> ClickHouseValue {
        ClickHouseValue::Boolean(format!("{}", value))
    }

    pub fn new_int_64(value: i64) -> ClickHouseValue {
        ClickHouseValue::ClickhouseInt(format!("{}", value))
    }

    pub fn new_float_64(value: f64) -> ClickHouseValue {
        ClickHouseValue::ClickhouseFloat(format!("{}", value))
    }

    pub fn new_date_time(value: DateTime<FixedOffset>) -> ClickHouseValue {
        ClickHouseValue::DateTime(value.to_utc().to_rfc3339().to_string())
    }

    pub fn new_array(value: Vec<ClickHouseValue>) -> ClickHouseValue {
        ClickHouseValue::Array(value)
    }

    pub fn new_enum(value: ClickHouseRuntimeEnum) -> ClickHouseValue {
        match value {
            ClickHouseRuntimeEnum::ClickHouseInt(v) => ClickHouseValue::Enum(format!("{}", v)),
            ClickHouseRuntimeEnum::ClickHouseString(v) => ClickHouseValue::Enum(format!("'{}'", v)),
        }
    }

    pub fn new_tuple(members: Vec<ClickHouseValue>) -> ClickHouseValue {
        let vals: Vec<ClickHouseValue> = members;
        ClickHouseValue::Nested(vals)
    }

    pub fn clickhouse_to_string(&self) -> String {
        match &self {
            ClickHouseValue::String(v) => format!(
                "\'{}\'",
                v.replace('\\', "\\\\").replace('\'', "\\\'").clone()
            ),
            ClickHouseValue::Boolean(v) => v.clone(),
            ClickHouseValue::ClickhouseInt(v) => v.clone(),
            ClickHouseValue::ClickhouseFloat(v) => v.clone(),
            ClickHouseValue::DateTime(v) => v.clone(),
            ClickHouseValue::Array(v) => format!(
                "[{}]",
                v.iter()
                    .map(|v| v.clickhouse_to_string())
                    .collect::<Vec<String>>()
                    .join(",")
            ),
            ClickHouseValue::Enum(v) => v.clone(),
            ClickHouseValue::Nested(v) => format!(
                "[({})]",
                v.iter()
                    .map(|v| v.clickhouse_to_string())
                    .collect::<Vec<String>>()
                    .join(",")
            ),
            ClickHouseValue::Null(v) => v.clone(),
            _ => String::from(""),
        }
    }
}

#[derive(Debug, Clone)]
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
    pub name: String,
    pub columns: Vec<ClickHouseColumn>,
    pub table_type: ClickHouseTableType,
    pub order_by: Vec<String>,
}

impl ClickHouseTable {
    pub fn create_data_table_query(&self, db_name: &str) -> Result<String, ClickhouseError> {
        create_table_query(db_name, self.clone(), ClickhouseEngine::MergeTree)
    }

    pub fn drop_data_table_query(&self, db_name: &str) -> Result<String, ClickhouseError> {
        drop_table_query(db_name, self.clone())
    }
}

pub fn sanitize_column_name(name: String) -> String {
    name.replace([' ', '-'], "_")
}

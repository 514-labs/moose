use super::errors::ClickhouseError;
use super::queries::{create_table_query, drop_table_query};
use crate::framework::core::infrastructure::table::{
    Column, ColumnType, DataEnum, FloatType, IntType, Nested,
};
use crate::framework::versions::Version;
use crate::infrastructure::olap::clickhouse::queries::ClickhouseEngine;
use chrono::{DateTime, FixedOffset};
use regex::Regex;
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
        write!(f, "{self:?}")
    }
}

#[allow(unused)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ClickHouseNested {
    name: String,
    columns: Vec<ClickHouseColumn>,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AggregationFunction<T> {
    pub function_name: String,
    pub argument_types: Vec<T>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ClickHouseColumnType {
    String,
    Boolean,
    ClickhouseInt(ClickHouseInt),
    ClickhouseFloat(ClickHouseFloat),
    Decimal {
        precision: u8,
        scale: u8,
    },
    DateTime,
    Json,
    Bytes,
    Array(Box<ClickHouseColumnType>),
    Nullable(Box<ClickHouseColumnType>),
    Enum(DataEnum),
    NamedTuple(Vec<(String, ClickHouseColumnType)>),
    Nested(Vec<ClickHouseColumn>),
    AggregateFunction(
        AggregationFunction<ClickHouseColumnType>,
        // the return type of the aggregation function
        Box<ClickHouseColumnType>,
    ),
    Map(Box<ClickHouseColumnType>, Box<ClickHouseColumnType>),
    Uuid,
    Date,
    Date32,
    DateTime64 {
        precision: u8,
    },
    LowCardinality(Box<ClickHouseColumnType>),
    IpV4,
    IpV6,
}

impl fmt::Display for ClickHouseColumnType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl ClickHouseColumnType {
    // TODO: delete? this is used only by `check_table` which is unused
    pub fn to_std_column_type(&self) -> (ColumnType, bool) {
        let mut required = true;
        let column_type = match self {
            ClickHouseColumnType::String => ColumnType::String,
            ClickHouseColumnType::Boolean => ColumnType::Boolean,
            ClickHouseColumnType::ClickhouseInt(ClickHouseInt::Int8) => {
                ColumnType::Int(IntType::Int8)
            }
            ClickHouseColumnType::ClickhouseInt(ClickHouseInt::Int16) => {
                ColumnType::Int(IntType::Int16)
            }
            ClickHouseColumnType::ClickhouseInt(ClickHouseInt::Int32) => {
                ColumnType::Int(IntType::Int32)
            }
            ClickHouseColumnType::ClickhouseInt(ClickHouseInt::Int64) => {
                ColumnType::Int(IntType::Int64)
            }
            ClickHouseColumnType::ClickhouseInt(ClickHouseInt::Int128) => {
                ColumnType::Int(IntType::Int128)
            }
            ClickHouseColumnType::ClickhouseInt(ClickHouseInt::Int256) => {
                ColumnType::Int(IntType::Int256)
            }
            ClickHouseColumnType::ClickhouseInt(ClickHouseInt::UInt8) => {
                ColumnType::Int(IntType::UInt8)
            }
            ClickHouseColumnType::ClickhouseInt(ClickHouseInt::UInt16) => {
                ColumnType::Int(IntType::UInt16)
            }
            ClickHouseColumnType::ClickhouseInt(ClickHouseInt::UInt32) => {
                ColumnType::Int(IntType::UInt32)
            }
            ClickHouseColumnType::ClickhouseInt(ClickHouseInt::UInt64) => {
                ColumnType::Int(IntType::UInt64)
            }
            ClickHouseColumnType::ClickhouseInt(ClickHouseInt::UInt128) => {
                ColumnType::Int(IntType::UInt128)
            }
            ClickHouseColumnType::ClickhouseInt(ClickHouseInt::UInt256) => {
                ColumnType::Int(IntType::UInt256)
            }

            ClickHouseColumnType::ClickhouseFloat(ClickHouseFloat::Float32) => {
                ColumnType::Float(FloatType::Float32)
            }
            ClickHouseColumnType::ClickhouseFloat(ClickHouseFloat::Float64) => {
                ColumnType::Float(FloatType::Float64)
            }
            ClickHouseColumnType::Decimal { precision, scale } => ColumnType::Decimal {
                precision: *precision,
                scale: *scale,
            },
            ClickHouseColumnType::Date => ColumnType::Date16,
            ClickHouseColumnType::Date32 => ColumnType::Date,
            ClickHouseColumnType::DateTime => ColumnType::DateTime { precision: None },
            ClickHouseColumnType::DateTime64 { precision } => ColumnType::DateTime {
                precision: Some(*precision),
            },
            ClickHouseColumnType::Json => ColumnType::Json,
            ClickHouseColumnType::Bytes => ColumnType::Bytes,
            ClickHouseColumnType::Array(inner_type) => {
                let (element_type, inner_required) = inner_type.to_std_column_type();
                ColumnType::Array {
                    element_type: Box::new(element_type),
                    element_nullable: !inner_required,
                }
            }
            ClickHouseColumnType::Enum(enum_def) => ColumnType::Enum(enum_def.clone()),
            ClickHouseColumnType::Nested(columns) => ColumnType::Nested(Nested {
                name: "Unknown".to_string(),
                columns: columns
                    .iter()
                    .map(|col| {
                        let (data_type, required) = col.column_type.to_std_column_type();
                        Column {
                            name: col.name.clone(),
                            data_type,
                            required: col.required && required,
                            unique: col.unique,
                            primary_key: col.primary_key,
                            default: None,
                            annotations: Default::default(),
                            comment: None,
                        }
                    })
                    .collect(),
                jwt: false,
            }),
            ClickHouseColumnType::Nullable(inner) => {
                required = false;
                inner.to_std_column_type().0
            }
            ClickHouseColumnType::AggregateFunction(_, return_type) => {
                return return_type.to_std_column_type();
            }
            ClickHouseColumnType::Map(key_type, value_type) => {
                let (key_std_type, key_required) = key_type.to_std_column_type();
                let (value_std_type, value_required) = value_type.to_std_column_type();

                let final_key_type = if key_required {
                    key_std_type
                } else {
                    ColumnType::Nullable(Box::new(key_std_type))
                };

                let final_value_type = if value_required {
                    value_std_type
                } else {
                    ColumnType::Nullable(Box::new(value_std_type))
                };

                ColumnType::Map {
                    key_type: Box::new(final_key_type),
                    value_type: Box::new(final_value_type),
                }
            }
            ClickHouseColumnType::Uuid => ColumnType::Uuid,
            ClickHouseColumnType::LowCardinality(t) => return t.to_std_column_type(),
            ClickHouseColumnType::IpV4 => ColumnType::IpV4,
            ClickHouseColumnType::IpV6 => ColumnType::IpV6,
            ClickHouseColumnType::NamedTuple(fields) => ColumnType::NamedTuple(
                fields
                    .iter()
                    .map(|(name, t)| {
                        let (t, required) = t.to_std_column_type();
                        let t = if required {
                            t
                        } else {
                            ColumnType::Nullable(Box::new(t))
                        };
                        (name.clone(), t)
                    })
                    .collect(),
            ),
        };
        (column_type, required)
    }

    pub fn from_type_str(type_str: &str) -> Option<Self> {
        // When we select from `system.columns`, the `Nested` columns are dotted names
        // so it's not handled here
        // unless we change the translation to `Tuple`
        let result = match type_str {
            "String" => Self::String,
            "Bool" | "Boolean" => Self::Boolean,
            // Integer types
            "Int8" => Self::ClickhouseInt(ClickHouseInt::Int8),
            "Int16" => Self::ClickhouseInt(ClickHouseInt::Int16),
            "Int32" => Self::ClickhouseInt(ClickHouseInt::Int32),
            "Int64" => Self::ClickhouseInt(ClickHouseInt::Int64),
            "Int128" => Self::ClickhouseInt(ClickHouseInt::Int128),
            "Int256" => Self::ClickhouseInt(ClickHouseInt::Int256),
            "UInt8" => Self::ClickhouseInt(ClickHouseInt::UInt8),
            "UInt16" => Self::ClickhouseInt(ClickHouseInt::UInt16),
            "UInt32" => Self::ClickhouseInt(ClickHouseInt::UInt32),
            "UInt64" => Self::ClickhouseInt(ClickHouseInt::UInt64),
            "UInt128" => Self::ClickhouseInt(ClickHouseInt::UInt128),
            "UInt256" => Self::ClickhouseInt(ClickHouseInt::UInt256),
            // Float types
            "Float32" => Self::ClickhouseFloat(ClickHouseFloat::Float32),
            "Float64" => Self::ClickhouseFloat(ClickHouseFloat::Float64),

            // Other types
            t if t.starts_with("Decimal(") => {
                let precision_and_scale = t
                    .trim_start_matches("Decimal(")
                    .trim_end_matches(')')
                    .split(',')
                    .map(|s| s.trim().parse::<u8>().ok())
                    .collect::<Vec<Option<u8>>>();

                let default_precision = Some(10);
                let default_scale = Some(0);

                // outer option is existence, inner option is parsing
                // if parsing failed, return None
                let precision = (*precision_and_scale.first().unwrap_or(&default_precision))?;
                let scale = (*precision_and_scale.get(1).unwrap_or(&default_scale))?;
                Self::Decimal { precision, scale }
            }

            t if t.starts_with("DateTime64(") => {
                let precision = t
                    .trim_start_matches("DateTime64(")
                    .trim_end_matches(')')
                    .trim()
                    .parse::<u8>()
                    .ok()?;

                Self::DateTime64 { precision }
            }
            "Date32" => Self::Date32,
            "Date" => Self::Date,
            "IPv4" => Self::IpV4,
            "IPv6" => Self::IpV6,
            "DateTime" | "DateTime('UTC')" => Self::DateTime,
            "JSON" => Self::Json,

            // recursively parsing Nullable and Array
            t if t.starts_with("Nullable(") => {
                let inner = t.trim_start_matches("Nullable(").trim_end_matches(')');
                match Self::from_type_str(inner) {
                    None => return None,
                    Some(inner_t) => Self::Nullable(Box::new(inner_t)),
                }
            }
            t if t.starts_with("Array(") => {
                let inner = t.trim_start_matches("Array(").trim_end_matches(')');
                match Self::from_type_str(inner) {
                    None => return None,
                    Some(inner_t) => Self::Array(Box::new(inner_t)),
                }
            }

            t if t.starts_with("Tuple(") => {
                let inner = t.trim_start_matches("Tuple(").trim_end_matches(')');
                // Simple parsing for now - assumes format like "name1 Type1, name2 Type2"
                let mut fields = Vec::new();
                for (i, part) in inner.split(',').enumerate() {
                    let part = part.trim();
                    if let Some(space_pos) = part.find(' ') {
                        // Named tuple element: "name Type"
                        let name = part[..space_pos].trim().to_string();
                        let type_str = part[space_pos + 1..].trim();
                        if let Some(field_type) = Self::from_type_str(type_str) {
                            fields.push((name, field_type));
                        } else {
                            return None;
                        }
                    } else {
                        // Unnamed tuple element, use index as name
                        if let Some(field_type) = Self::from_type_str(part) {
                            fields.push((format!("field_{i}"), field_type));
                        } else {
                            return None;
                        }
                    }
                }
                Self::NamedTuple(fields)
            }

            t if t.starts_with("Map(") => {
                let inner = t.trim_start_matches("Map(").trim_end_matches(')');
                let parts: Vec<&str> = inner.split(',').collect();
                if parts.len() == 2 {
                    let key_type_str = parts[0].trim();
                    let value_type_str = parts[1].trim();
                    if let (Some(key_type), Some(value_type)) = (
                        Self::from_type_str(key_type_str),
                        Self::from_type_str(value_type_str),
                    ) {
                        Self::Map(Box::new(key_type), Box::new(value_type))
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            }

            t if t.starts_with("Enum8(") || t.starts_with("Enum16(") => {
                let enum_content = type_str
                    .trim_start_matches("Enum8(")
                    .trim_start_matches("Enum16(")
                    .trim_end_matches(')');

                // Use regex to match enum values, handling potential commas in the names
                let re = Regex::new(r"'([^']*)'\s*=\s*(\d+)").unwrap();
                let values = re
                    .captures_iter(enum_content)
                    .map(|cap| {
                        let name = cap[1].to_string();
                        let value = cap[2].parse::<u8>().unwrap_or(0);

                        crate::framework::core::infrastructure::table::EnumMember {
                            name: name.clone(),
                            value: crate::framework::core::infrastructure::table::EnumValue::Int(
                                value,
                            ),
                        }
                    })
                    .collect::<Vec<_>>();

                Self::Enum(DataEnum {
                    name: "Unknown".to_string(),
                    values,
                })
            }
            _ => return None,
        };
        Some(result)
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
        write!(f, "{self:?}")
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ClickHouseFloat {
    Float32,
    Float64,
}

impl fmt::Display for ClickHouseFloat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
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
    pub comment: Option<String>, // Column comment for metadata storage
}

impl ClickHouseColumn {
    pub fn is_array(&self) -> bool {
        matches!(&self.column_type, ClickHouseColumnType::Array(_))
    }
    pub fn is_nested(&self) -> bool {
        matches!(&self.column_type, ClickHouseColumnType::Nested(_))
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
    Json(serde_json::Map<String, serde_json::Value>),
    Bytes,
    Array(Vec<ClickHouseValue>),
    Enum(String),
    Nested(Vec<ClickHouseValue>),
    NamedTuple(Vec<ClickHouseValue>),
    Map(Vec<(ClickHouseValue, ClickHouseValue)>),
    Null,
}

const NULL: &str = "NULL";

// TODO - add support for Decimal, Json, Bytes
impl ClickHouseValue {
    pub fn new_null() -> ClickHouseValue {
        ClickHouseValue::Null
    }

    pub fn new_string(value: String) -> ClickHouseValue {
        ClickHouseValue::String(value)
    }

    pub fn new_boolean(value: bool) -> ClickHouseValue {
        ClickHouseValue::Boolean(format!("{value}"))
    }

    pub fn new_int_64(value: i64) -> ClickHouseValue {
        ClickHouseValue::ClickhouseInt(format!("{value}"))
    }

    pub fn new_float_64(value: f64) -> ClickHouseValue {
        ClickHouseValue::ClickhouseFloat(format!("{value}"))
    }

    pub fn new_date_time(value: DateTime<FixedOffset>) -> ClickHouseValue {
        ClickHouseValue::DateTime(value.to_utc().to_rfc3339().to_string())
    }

    pub fn new_array(value: Vec<ClickHouseValue>) -> ClickHouseValue {
        ClickHouseValue::Array(value)
    }

    pub fn new_enum(value: ClickHouseRuntimeEnum) -> ClickHouseValue {
        match value {
            ClickHouseRuntimeEnum::ClickHouseInt(v) => ClickHouseValue::Enum(format!("{v}")),
            ClickHouseRuntimeEnum::ClickHouseString(v) => ClickHouseValue::Enum(format!("'{v}'")),
        }
    }

    pub fn new_json(map: serde_json::Map<String, serde_json::Value>) -> ClickHouseValue {
        ClickHouseValue::Json(map)
    }

    pub fn new_tuple(members: Vec<ClickHouseValue>) -> ClickHouseValue {
        let vals: Vec<ClickHouseValue> = members;
        ClickHouseValue::NamedTuple(vals)
    }

    pub fn new_map(map: Vec<(ClickHouseValue, ClickHouseValue)>) -> ClickHouseValue {
        ClickHouseValue::Map(map)
    }

    pub fn clickhouse_to_string(&self) -> String {
        match &self {
            ClickHouseValue::String(v) => format!("\'{}\'", escape_ch_string(v)),
            ClickHouseValue::Boolean(v) => v.clone(),
            ClickHouseValue::ClickhouseInt(v) => v.clone(),
            ClickHouseValue::ClickhouseFloat(v) => v.clone(),
            ClickHouseValue::DateTime(v) => format!("'{v}'"),
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
            ClickHouseValue::NamedTuple(v) => format!(
                "({})",
                v.iter()
                    .map(|v| v.clickhouse_to_string())
                    .collect::<Vec<String>>()
                    .join(",")
            ),
            ClickHouseValue::Map(v) => format!(
                "{{{}}}",
                v.iter()
                    .map(|(k, v)| format!(
                        "{}: {}",
                        k.clickhouse_to_string(),
                        v.clickhouse_to_string()
                    ))
                    .collect::<Vec<String>>()
                    .join(",")
            ),
            ClickHouseValue::Null => NULL.to_string(),
            ClickHouseValue::Json(v) => {
                format!("'{}'", escape_ch_string(&serde_json::to_string(v).unwrap()))
            }
            _ => String::from(""),
        }
    }
}

fn escape_ch_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\'', "\\\'")
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
    pub version: Option<Version>,
    pub columns: Vec<ClickHouseColumn>,
    pub order_by: Vec<String>,
    pub engine: ClickhouseEngine,
}

impl ClickHouseTable {
    pub fn create_data_table_query(&self, db_name: &str) -> Result<String, ClickhouseError> {
        create_table_query(db_name, self.clone())
    }

    pub fn drop_data_table_query(&self, db_name: &str) -> Result<String, ClickhouseError> {
        drop_table_query(db_name, self.clone())
    }

    pub fn primary_key_columns(&self) -> Vec<&str> {
        self.columns
            .iter()
            .filter_map(|c| {
                if c.primary_key {
                    Some(c.name.as_str())
                } else {
                    None
                }
            })
            .collect()
    }
}

pub fn sanitize_column_name(name: String) -> String {
    name.replace([' ', '-'], "_")
}

/// Wraps a column name in backticks for safe use in ClickHouse SQL queries
pub fn wrap_column_name(name: &str) -> String {
    format!("`{name}`")
}

/// Wraps multiple column names in backticks and joins them with the specified separator
pub fn wrap_and_join_column_names(names: &[String], separator: &str) -> String {
    names
        .iter()
        .map(|name| wrap_column_name(name))
        .collect::<Vec<String>>()
        .join(separator)
}

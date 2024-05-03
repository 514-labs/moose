use std::fmt;

use serde::ser::SerializeStruct;
use serde::Serialize;

use super::config::DataModelConfig;

#[derive(Debug, Clone, Serialize, Eq, PartialEq)]
pub struct DataModel {
    pub db_name: String,
    pub columns: Vec<Column>,
    pub name: String,
    pub config: DataModelConfig,
}

impl DataModel {
    pub fn to_table(&self, version: &str) -> Table {
        Table {
            db_name: self.db_name.clone(),
            table_type: TableType::Table,
            name: format!("{}_{}", self.name, version.replace('.', "_")),
            columns: self.columns.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Eq, PartialEq)]
/// An internal framework representation for an enum.
/// Avoiding the use of the `Enum` keyword to avoid conflicts with Prisma's Enum type
pub struct DataEnum {
    pub name: String,
    pub values: Vec<EnumMember>,
}

#[derive(Debug, Clone, Serialize, Eq, PartialEq)]
pub struct EnumMember {
    pub name: String,
    pub value: EnumValue,
}

#[derive(Debug, Clone, Serialize, Eq, PartialEq)]
pub enum EnumValue {
    Int(u8),
    String(String),
}

#[derive(Debug, Clone)]
pub enum TableType {
    Table,
    View,
    Unsupported,
}

#[derive(Debug, Clone)]
pub struct Table {
    pub db_name: String,
    pub table_type: TableType,
    pub name: String,
    pub columns: Vec<Column>,
}

#[derive(Debug, Clone, Serialize, Eq, PartialEq)]
pub struct Column {
    pub name: String,
    pub data_type: ColumnType,
    pub required: bool,
    pub unique: bool,
    pub primary_key: bool,
    pub default: Option<ColumnDefaults>,
}

#[derive(Debug, Clone, Serialize, Eq, PartialEq)]
pub enum ColumnDefaults {
    AutoIncrement,
    CUID,
    UUID,
    Now,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ColumnType {
    String,
    Boolean,
    Int,
    BigInt,
    Float,
    Decimal,
    DateTime,
    Enum(DataEnum),
    Array(Box<ColumnType>),
    Json,  // TODO: Eventually support for only views and tables (not topics)
    Bytes, // TODO: Explore if we ever need this type
}

impl fmt::Display for ColumnType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ColumnType::String => write!(f, "String"),
            ColumnType::Boolean => write!(f, "Boolean"),
            ColumnType::Int => write!(f, "Int"),
            ColumnType::BigInt => write!(f, "BigInt"),
            ColumnType::Float => write!(f, "Float"),
            ColumnType::Decimal => write!(f, "Decimal"),
            ColumnType::DateTime => write!(f, "DateTime"),
            ColumnType::Enum(e) => write!(f, "Enum<{}>", e.name),
            ColumnType::Array(inner) => write!(f, "Array<{}>", inner),
            ColumnType::Json => write!(f, "Json"),
            ColumnType::Bytes => write!(f, "Bytes"),
        }
    }
}

impl Serialize for ColumnType {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            ColumnType::String => serializer.serialize_str("String"),
            ColumnType::Boolean => serializer.serialize_str("Boolean"),
            ColumnType::Int => serializer.serialize_str("Int"),
            ColumnType::BigInt => serializer.serialize_str("BigInt"),
            ColumnType::Float => serializer.serialize_str("Float"),
            ColumnType::Decimal => serializer.serialize_str("Decimal"),
            ColumnType::DateTime => serializer.serialize_str("DateTime"),
            ColumnType::Enum(data_enum) => {
                let mut state = serializer.serialize_struct("Enum", 2)?;
                state.serialize_field("name", &data_enum.name)?;
                state.serialize_field("values", &data_enum.values)?;
                state.end()
            }
            ColumnType::Array(_) => {
                let serial = format!("{}", self);
                serializer.serialize_str(&serial)
            }
            ColumnType::Json => serializer.serialize_str("Json"),
            ColumnType::Bytes => serializer.serialize_str("Bytes"),
        }
    }
}

pub fn is_enum_type(string_type: &str, enums: &[DataEnum]) -> bool {
    enums.iter().any(|e| e.name == string_type)
}

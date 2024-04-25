//! # Schema
//! Schemas are the internal representation for data models in the framework.
//!
//! The schema module is responsible for parsing the schema file and converting it into a format that can be used by the framework.
//!
//! We currently support the following data types:
//! - String
//! - Boolean
//! - Int
//! - BigInt
//! - Float
//! - Decimal
//! - DateTime
//! - Array
//! - Enum
//!
//! We only implemented part of the prisma schema parsing. We only support models, enums and fields. We don't support relations, or anything else for the moment

use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::{
    fmt,
    path::{Path, PathBuf},
};

use log::debug;
use serde::ser::SerializeStruct;
use serde::Serialize;

use crate::framework::controller::FrameworkObject;
use crate::framework::prisma;
use crate::framework::typescript;
use crate::utilities::constants::TS_INTERFACE_GENERATE_EXT;
use crate::utilities::system::file_name_contains;

use super::controller::MappingError;

pub mod templates;

#[derive(Debug, Clone)]
pub struct DuplicateModelError {
    pub model_name: String,
    pub file_path: PathBuf,
    pub other_file_path: PathBuf,
}

impl DuplicateModelError {
    pub fn try_insert(
        map: &mut HashMap<String, FrameworkObject>,
        fo: FrameworkObject,
        current_path: &Path,
    ) -> Result<(), Self> {
        let maybe_existing = map.insert(fo.data_model.name.clone(), fo);
        match maybe_existing {
            None => Ok(()),
            Some(other_fo) => Err(DuplicateModelError {
                model_name: other_fo.data_model.name,
                file_path: current_path.to_path_buf(),
                other_file_path: other_fo.original_file_path,
            }),
        }
    }
}

impl Display for DuplicateModelError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Duplicate model {} in files: {}, {}",
            self.model_name,
            self.file_path.display(),
            self.other_file_path.display()
        )
    }
}

impl std::error::Error for DuplicateModelError {}

#[derive(Debug, thiserror::Error)]
#[error("Failed to parse the data model file")]
#[non_exhaustive]
pub enum DataModelParsingError {
    PrismaParsingError(#[from] prisma::parser::PrismaParsingError),
    TypescriptParsingError(#[from] typescript::parser::TypescriptParsingError),
    MappingError(#[from] MappingError),
}

pub fn is_schema_file(path: &Path) -> bool {
    path.extension()
        .map(|extension| extension == "prisma" || extension == "ts")
        .unwrap_or(false)
        && !file_name_contains(path, TS_INTERFACE_GENERATE_EXT)
}

// TODO: Make the parse schema file a variable and pass it into the function
pub fn parse_data_model_file<O>(
    path: &Path,
    version: &str,
    mapper: fn(DataModel, path: &Path, version: &str) -> Result<O, MappingError>,
) -> Result<Vec<O>, DataModelParsingError> {
    let is_prisma_schema = path.extension().map(|e| e == "prisma").unwrap_or(false);

    let file_objects = if is_prisma_schema {
        debug!("Parsing prisma file {:?}", path);
        prisma::parser::extract_data_model_from_file(path)?
    } else {
        debug!("Parsing typescript file {:?}", path);
        typescript::parser::extract_data_model_from_file(path)?
    };

    let mut result = Vec::new();
    for data_model in file_objects.models {
        result.push(mapper(data_model, path, version)?);
    }
    Ok(result)
}

pub struct FileObjects {
    pub models: Vec<DataModel>,
    pub enums: Vec<DataEnum>,
}

impl FileObjects {
    pub fn new(models: Vec<DataModel>, enums: Vec<DataEnum>) -> FileObjects {
        FileObjects { models, enums }
    }
}

#[derive(Debug, Clone, Serialize, Eq, PartialEq)]
pub struct DataModel {
    pub db_name: String,
    pub columns: Vec<Column>,
    pub name: String,
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
    pub value: Option<EnumValue>,
}

#[derive(Debug, Clone, Serialize, Eq, PartialEq)]
pub enum EnumValue {
    Int(u8),
    String(String),
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

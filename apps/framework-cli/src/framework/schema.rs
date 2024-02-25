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
//!
//! We only implemented part of the prisma schema parsing. We only support models and fields. We don't support enums, relations, or anything else for the moment

use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::{
    fmt,
    path::{Path, PathBuf},
};

use crate::framework::controller::FrameworkObject;
use diagnostics::Diagnostics;
use config::Value;
use diagnostics::Diagnostics;

use log::debug;
use schema_ast::ast::{Enum, Model};
use schema_ast::{
    ast::{Attribute, Field, SchemaAst, Top, WithName},
    parse_schema,
};
use serde::Serialize;

use crate::infrastructure::olap::clickhouse::mapper::arity_mapper;

pub mod templates;

#[derive(Debug, Clone)]
pub enum ParsingError {
    FileNotFound { path: PathBuf },
    UnsupportedDataTypeError { type_name: String },
    OtherError,
}

#[derive(Debug, Clone)]
pub struct UnsupportedDataTypeError {
    pub type_name: String,
}

impl Display for UnsupportedDataTypeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "UnsupportedDataTypeError: {}", self.type_name)
    }
}

impl std::error::Error for UnsupportedDataTypeError {}

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

pub fn is_prisma_file(path: &Path) -> bool {
    path.extension().map(|e| e == "prisma").unwrap_or(false)
}

// TODO: Make the parse schema file a variable and pass it into the function
pub fn parse_schema_file<O>(
    path: &Path,
    version: &str,
    mapper: fn(DataModel, path: &Path, version: &str) -> O,
) -> Result<Vec<O>, ParsingError> {
    let schema_file = std::fs::read_to_string(path).map_err(|_| ParsingError::FileNotFound {
        path: path.to_path_buf(),
    })?;

    let mut diagnostics = Diagnostics::default();

    let ast = parse_schema(&schema_file, &mut diagnostics);

    let file_objects = ast_mapper(ast)?
        .into_iter()
        .map(|data_model| mapper(data_model, path, version))
        .collect()?;

    Ok(file_objects.models.into_iter().map(mapper).collect())
}

pub struct FileObjects {
    pub models: Vec<DataModel>,
    pub enums: Vec<ValueEnum>,
}

impl FileObjects {
    pub fn new(models: Vec<DataModel>, enums: Vec<ValueEnum>) -> FileObjects {
        FileObjects { models, enums }
    }
}

#[derive(Debug, Clone, Serialize, Eq, PartialEq)]
pub struct DataModel {
    pub db_name: String,
    pub columns: Vec<Column>,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ValueEnum {
    pub name: String,
    pub values: Vec<String>,
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

pub trait TableOps {
    fn create_table_query(&self) -> Result<String, UnsupportedDataTypeError>;
    fn drop_table_query(&self) -> Result<String, UnsupportedDataTypeError>;
}

pub trait MatViewOps {
    fn create_materialized_view_query(&self) -> Result<String, UnsupportedDataTypeError>;
    fn drop_materialized_view_query(&self) -> Result<String, UnsupportedDataTypeError>;
}

#[derive(Copy, Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub enum FieldArity {
    Required,
    Optional,
    List,
}

impl FieldArity {
    pub fn is_list(&self) -> bool {
        matches!(self, &FieldArity::List)
    }

    pub fn is_optional(&self) -> bool {
        matches!(self, &FieldArity::Optional)
    }

    pub fn is_required(&self) -> bool {
        matches!(self, &FieldArity::Required)
    }
}

#[derive(Debug, Clone, Serialize, Eq, PartialEq)]
pub struct Column {
    pub name: String,
    pub data_type: ColumnType,
    pub arity: FieldArity,
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

impl fmt::Display for ParsingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "The following type is unsupported: {:?}", self)
    }
}

#[derive(Debug, Clone, Serialize, Eq, PartialEq)]
pub enum ColumnType {
    String,
    Boolean,
    Int,
    BigInt,
    Float,
    Decimal,
    DateTime,
    Enum(ValueEnum),
    Json,  // TODO: Eventually support for only views and tables (not topics)
    Bytes, // TODO: Explore if we ever need this type
    Unsupported,
}

impl fmt::Display for ColumnType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

fn map_column_string_type_to_column_type(string_type: &str) -> ColumnType {
    match string_type {
        "String" => ColumnType::String,
        "Boolean" => ColumnType::Boolean,
        "Int" => ColumnType::Int,
        "BigInt" => ColumnType::BigInt,
        "Float" => ColumnType::Float,
        "Decimal" => ColumnType::Decimal,
        "DateTime" => ColumnType::DateTime,
        _ => ColumnType::Unsupported,
    }
}

pub struct FieldAttributes {
    unique: bool,
    primary_key: bool,
    default: Option<ColumnDefaults>,
}

impl FieldAttributes {
    #[allow(clippy::never_loop, clippy::match_single_binding)]
    fn new(attributes: Vec<Attribute>) -> Result<FieldAttributes, ParsingError> {
        let unique: bool = false;
        let mut primary_key: bool = false;
        let default: Option<ColumnDefaults> = None;

        // TODO: Implement default values and primary keys once we have the ingestion table architecture setup
        for attribute in attributes {
            match attribute.name() {
                "id" => primary_key = true,
                _ => {
                    return Err(ParsingError::UnsupportedDataTypeError {
                        type_name: format!(
                            "we currently don't support attribute {}",
                            attribute.name()
                        ),
                    });
                }
            }
        }

        Ok(FieldAttributes {
            unique,
            primary_key,
            default,
        })
    }
}

fn is_enum_type(string_type: &str, enums: &Vec<ValueEnum>) -> bool {
    enums.iter().any(|e| e.name == string_type)
}

fn field_to_column(f: &Field, enums: &Vec<ValueEnum>) -> Result<Column, ParsingError> {
    let attributes = FieldAttributes::new(f.attributes.clone())?;

    match &f.field_type {
        schema_ast::ast::FieldType::Supported(ft) => Ok(Column {
            name: f.name().to_string(),
            data_type: match is_enum_type(&ft.name, enums) {
                true => ColumnType::Enum(enums.iter().find(|e| e.name == ft.name).unwrap().clone()),
                false => map_column_string_type_to_column_type(&ft.name),
            },
            arity: arity_mapper(f.arity),
            unique: attributes.unique,
            primary_key: attributes.primary_key,
            default: attributes.default,
        }),
        schema_ast::ast::FieldType::Unsupported(x, _) => {
            Err(ParsingError::UnsupportedDataTypeError {
                type_name: x.to_string(),
            })
        }
    }
}

fn top_to_datamodel(m: &Model, enums: &Vec<ValueEnum>) -> Result<DataModel, ParsingError> {
    let schema_name = m.name().to_string();
    let mut version = 1;

    let attributes = m.attributes();

    // Get the value of the version attribute in the ugliest way possible
    let version_attribute = attributes.iter().find(|a| a.name() == "version");

    if let Some(attribute) = version_attribute {
        version = attribute
            .arguments
            .arguments
            .first()
            .map(|arg| {
                arg.value
                    .as_numeric_value()
                    .unwrap()
                    .0
                    .parse::<i8>()
                    .unwrap()
            })
            .unwrap_or(version);
    }

    let columns: Result<Vec<Column>, ParsingError> = m
        .iter_fields()
        .map(|(_id, f)| field_to_column(f, enums))
        .collect();

    Ok(DataModel {
        db_name: "local".to_string(),
        columns: columns?,
        name: schema_name,
        version,
    })
}

fn top_to_enum(e: &Enum) -> ValueEnum {
    let name = e.name().to_string();
    let values = e
        .iter_values()
        .map(|(_id, v)| v.name().to_string())
        .collect();
    ValueEnum { name, values }
}

pub fn ast_mapper(ast: SchemaAst) -> Result<FileObjects, ParsingError> {
    let mut models = Vec::new();
    let mut enums = Vec::new();

    ast.iter_tops().for_each(|(_id, t)| match t {
        Top::Model(m) => {
            models.push(m);
        }
        Top::Enum(e) => {
            enums.push(top_to_enum(e));
        }
        _ => {
            ParsingError::UnsupportedDataTypeError {
                type_name: "we currently only support models and enums".to_string(),
            };
        }
    });

    let parsed_models = models
        .into_iter()
        .map(|m| top_to_datamodel(m, &enums))
        .collect::<Result<Vec<DataModel>, ParsingError>>()?;

    Ok(FileObjects::new(parsed_models, enums))
}

pub async fn process_schema_file(
    schema_file_path: &Path,
    project: Arc<Project>,
    configured_client: &ConfiguredDBClient,
    route_table: &mut HashMap<PathBuf, RouteMeta>,
) -> anyhow::Result<()> {
    let framework_objects = get_framework_objects_from_schema_file(schema_file_path)?;
    let mut compilable_objects: Vec<TypescriptObjects> = Vec::new();
    process_objects(
        framework_objects,
        project.clone(),
        schema_file_path,
        configured_client,
        &mut compilable_objects,
        route_table,
    )
    .await?;
    debug!("All objects created, generating sdk...");
    let sdk_location = generate_ts_sdk(project, compilable_objects)?;

    let package_manager = package_managers::PackageManager::Npm;
    package_managers::install_packages(&sdk_location, &package_manager)?;
    package_managers::run_build(&sdk_location, &package_manager)?;
    package_managers::link_sdk(&sdk_location, None, &package_manager)?;
    Ok(())
}

#[cfg(test)]
mod tests {

    use crate::framework::schema::parse_schema_file;

    #[test]
    fn test_parse_schema_file() {
        let current_dir = std::env::current_dir().unwrap();

        let test_file = current_dir.join("tests/psl/simple.prisma");

        let result = parse_schema_file(&test_file, |x| x);
        println!("{:?}", result);
        assert!(result.is_ok());
    }
}

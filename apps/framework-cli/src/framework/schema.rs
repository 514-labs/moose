use std::{
    collections::HashMap,
    fmt,
    io::Error,
    path::{Path, PathBuf},
};

use diagnostics::Diagnostics;

use log::debug;
use schema_ast::{
    ast::{Attribute, Field, SchemaAst, Top, WithAttributes, WithName},
    parse_schema,
};
use serde::Serialize;

use crate::{
    framework::{
        controller::{get_framework_objects_from_schema_file, process_objects},
        sdks::{generate_ts_sdk, TypescriptObjects},
    },
    infrastructure::olap::clickhouse::{mapper::arity_mapper, ConfiguredDBClient},
    project::Project,
    utilities::package_managers,
};

use super::controller::RouteMeta;

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

/// A function that maps an input type to an output type
type MapperFunc<I, O> = fn(i: I) -> O;

// TODO: Make the parse schema file a variable and pass it into the function
pub fn parse_schema_file<O>(
    path: &Path,
    mapper: MapperFunc<DataModel, O>,
) -> Result<Vec<O>, ParsingError> {
    let schema_file = std::fs::read_to_string(path).map_err(|_| ParsingError::FileNotFound {
        path: path.to_path_buf(),
    })?;

    let mut diagnostics = Diagnostics::default();

    let ast = parse_schema(&schema_file, &mut diagnostics);

    Ok(ast_mapper(ast)?.into_iter().map(mapper).collect())
}

#[derive(Debug, Clone, Serialize)]
pub struct DataModel {
    pub db_name: String,
    pub columns: Vec<Column>,
    pub name: String,
    pub version: i8,
}

impl DataModel {
    pub fn new(db_name: String, columns: Vec<Column>, name: String, version: i8) -> DataModel {
        DataModel {
            db_name,
            columns,
            name,
            version,
        }
    }

    pub fn to_table(&self) -> Table {
        Table {
            db_name: self.db_name.clone(),
            table_type: TableType::Table,
            name: self.name.clone(),
            columns: self.columns.clone(),
        }
    }

    pub fn to_view(&self) -> Table {
        Table {
            db_name: self.db_name.clone(),
            table_type: TableType::View,
            name: format!("{}_view", self.name),
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

#[derive(Debug, Clone, Serialize)]
pub struct Column {
    pub name: String,
    pub data_type: ColumnType,
    pub arity: FieldArity,
    pub unique: bool,
    pub primary_key: bool,
    pub default: Option<ColumnDefaults>,
}

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
pub enum ColumnType {
    String,
    Boolean,
    Int,
    BigInt,
    Float,
    Decimal,
    DateTime,
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
        let primary_key: bool = false;
        let default: Option<ColumnDefaults> = None;

        // TODO: Implement default values and primary keys once we have the ingestion table architecture setup
        for attribute in attributes {
            match attribute.name() {
                // "id" => {primary_key = true},
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

fn field_to_column(f: &Field) -> Result<Column, ParsingError> {
    let attributes = FieldAttributes::new(f.attributes.clone())?;

    match &f.field_type {
        schema_ast::ast::FieldType::Supported(ft) => Ok(Column {
            name: f.name().to_string(),
            data_type: map_column_string_type_to_column_type(ft.name.as_str()),
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

fn top_to_schema(t: &Top) -> Result<DataModel, ParsingError> {
    match t {
        Top::Model(m) => {
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

            let columns: Result<Vec<Column>, ParsingError> =
                m.iter_fields().map(|(_id, f)| field_to_column(f)).collect();

            Ok(DataModel {
                db_name: "local".to_string(),
                columns: columns?,
                name: schema_name,
                version,
            })
        }
        _ => Err(ParsingError::UnsupportedDataTypeError {
            type_name: "we don't currently support anything other than models".to_string(),
        }),
    }
}

pub fn ast_mapper(ast: SchemaAst) -> Result<Vec<DataModel>, ParsingError> {
    ast.iter_tops()
        .map(|(_id, t)| top_to_schema(t))
        .collect::<Result<Vec<DataModel>, ParsingError>>()
}

pub async fn process_schema_file(
    schema_file_path: &Path,
    project: &Project,
    configured_client: &ConfiguredDBClient,
    route_table: &mut HashMap<PathBuf, RouteMeta>,
) -> Result<(), Error> {
    let framework_objects = get_framework_objects_from_schema_file(schema_file_path)?;
    let mut compilable_objects: Vec<TypescriptObjects> = Vec::new();
    process_objects(
        framework_objects,
        project,
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

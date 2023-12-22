use std::{fmt, path::PathBuf};

use diagnostics::Diagnostics;

use schema_ast::{
    ast::{Attribute, Field, FieldArity, SchemaAst, Top, WithAttributes, WithName},
    parse_schema,
};

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
    path: PathBuf,
    mapper: MapperFunc<Schema, O>,
) -> Result<Vec<O>, ParsingError> {
    let schema_file = std::fs::read_to_string(path.clone())
        .map_err(|_| ParsingError::FileNotFound { path: path.clone() })?;

    let mut diagnostics = Diagnostics::default();

    let ast = parse_schema(&schema_file, &mut diagnostics);

    Ok(ast_mapper(ast)?.into_iter().map(mapper).collect())
}

pub struct Schema {
    pub db_name: String,
    pub columns: Vec<Column>,
    pub name: String,
    pub version: i8,
}

impl Schema {
    pub fn new(db_name: String, columns: Vec<Column>, name: String, version: i8) -> Schema {
        Schema {
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

#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub data_type: ColumnType,
    pub arity: FieldArity,
    pub unique: bool,
    pub primary_key: bool,
    pub default: Option<ColumnDefaults>,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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
                    })
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
            arity: f.arity,
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

fn top_to_schema(t: &Top) -> Result<Schema, ParsingError> {
    match t {
        Top::Model(m) => {
            let schema_name = m.name().to_string();
            let default_version = 1;

            let attributes = m.attributes();

            println!("{:?}", attributes);

            // Get the value of the version attribute in the ugliest way possible
            let version = attributes
                .iter()
                .find(|a| a.name() == "version")
                .unwrap()
                .arguments
                .arguments // Why prisma team, why?
                .first()
                .map(|arg| {
                    arg.value
                        .as_numeric_value()
                        .unwrap()
                        .0
                        .parse::<i8>()
                        .unwrap()
                });

            let columns: Result<Vec<Column>, ParsingError> =
                m.iter_fields().map(|(_id, f)| field_to_column(f)).collect();

            Ok(Schema {
                db_name: "local".to_string(),
                columns: columns?,
                name: schema_name,
                version: version.unwrap_or(default_version),
            })
        }
        _ => Err(ParsingError::UnsupportedDataTypeError {
            type_name: "we don't currently support anything other than models".to_string(),
        }),
    }
}

pub fn ast_mapper(ast: SchemaAst) -> Result<Vec<Schema>, ParsingError> {
    ast.iter_tops()
        .map(|(_id, t)| top_to_schema(t))
        .collect::<Result<Vec<Schema>, ParsingError>>()
}

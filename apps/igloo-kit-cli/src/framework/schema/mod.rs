use std::{fmt, path::PathBuf};
use std::io::{Error, ErrorKind};

use diagnostics::Diagnostics;

use schema_ast::{parse_schema, ast::{SchemaAst, FieldArity, Top, Field, WithName, Attribute}};

use crate::infrastructure::db::clickhouse::ClickhouseTable;
mod clickhouse_mappers;
pub mod templates;

#[derive(Debug, Clone)]
pub enum ParsingError {
    FileNotFound {path: PathBuf},
    UnsupportedDataTypeError {type_name: String},
    OtherError
}

#[derive(Debug, Clone)]
pub struct UnsupportedDataTypeError {
    pub type_name: String
}


// TODO: Make the parse schema file a variable and pass it into the function
pub fn parse_schema_file(path: PathBuf) -> Result<Vec<ClickhouseTable>, ParsingError>  {
    let schema_file = std::fs::read_to_string(path.clone()).map_err(
        |_| ParsingError::FileNotFound {path: path.clone()}
    )?;

    let mut diagnostics = Diagnostics::default();

    let ast = parse_schema(&schema_file, &mut diagnostics);

    let tables = ast_mapper(ast)?
        .into_iter().map(|table| {
            clickhouse_mappers::std_table_to_clickhouse_table("local".to_string(), table)
        }).collect();

    Ok(tables)
}

#[derive(Debug, Clone)]
pub enum TableType {
    Table,
    View,
    Unsupported,
}


#[derive(Debug, Clone)]
pub struct Table {
    pub table_type: TableType,
    pub name: String,
    pub columns: Vec<Column>,
}

pub trait OpsTable {
    fn create_table_query(&self) -> Result<String, UnsupportedDataTypeError>;
    fn drop_table_query(&self) -> Result<String, UnsupportedDataTypeError>;
}

#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub data_type: ColumnType,
    pub arity: FieldArity,
    pub unique: bool,
    pub primary_key: bool,
    pub default: Option<ColumnDefaults>
}

#[derive(Debug, Clone)]
pub enum ColumnDefaults {
    AutoIncrement,
    CUID,
    UUID,
    Now
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
    Json, // TODO: Eventually support for only views and tables (not topics)
    Bytes, // TODO: Explore if we ever need this type
    Unsupported
}

fn map_column_string_type_to_column_type (string_type: &str) -> ColumnType {
    match string_type {
        "String" => ColumnType::String,
        "Boolean" => ColumnType::Boolean,
        "Int" => ColumnType::Int,
        "BigInt" => ColumnType::BigInt,
        "Float" => ColumnType::Float,
        "Decimal" => ColumnType::Decimal,
        "DateTime" => ColumnType::DateTime,
        _ => ColumnType::Unsupported
    }
}

pub struct FieldAttributes {
    unique: bool,
    primary_key: bool,
    default: Option<ColumnDefaults>
}

impl FieldAttributes {
    fn new(attributes :Vec<Attribute>) -> Result<FieldAttributes, ParsingError> {
        let mut unique: bool = false;
        let mut primary_key: bool = false;
        let mut default: Option<ColumnDefaults> = None;

        // TODO: Implement default values and primary keys once we have the ingestion table architecture setup
        for attribute in attributes {
            match attribute.name() {
                // "id" => {primary_key = true},
                _ => {return Err(ParsingError::UnsupportedDataTypeError{type_name: format!("we currently don't support attribute {}", attribute.name())})}
            }
        }

        return Ok(FieldAttributes {
            unique,
            primary_key,
            default
        })
    }
}

fn field_to_column(f: &Field) -> Result<Column, ParsingError> {
    let attributes = FieldAttributes::new(f.attributes.clone())?;

    match &f.field_type {
        schema_ast::ast::FieldType::Supported(ft) => {
            Ok(Column {
                name: f.name().to_string(),
                data_type: map_column_string_type_to_column_type(ft.name.as_str()),
                arity: f.arity.clone(),
                unique: attributes.unique,
                primary_key: attributes.primary_key,
                default: attributes.default,
            })
        },
        schema_ast::ast::FieldType::Unsupported(x, _) => {
            Err(ParsingError::UnsupportedDataTypeError{type_name: x.to_string()})
        }
    }
}

fn top_to_table(t: &Top) -> Result<Table, ParsingError> {
    match t {
        Top::Model(m) => {
            let table_name = m.name().to_string();

            let columns: Result<Vec<Column>, ParsingError> = m.iter_fields().map(|(_id, f)| {
                field_to_column(f)
            }).collect();
            
            Ok(Table {
                table_type: TableType::Table,
                name: table_name,
                columns: columns?
            })
        }
        _ => { 
            Err(ParsingError::UnsupportedDataTypeError {type_name: "we don't currently support anything other than models".to_string()})
        }
    }
}

pub fn ast_mapper(ast: SchemaAst) -> Result<Vec<Table>, ParsingError>  {
    ast.iter_tops().map(|(_id, t)| {
        top_to_table(t)
    }).collect::<Result<Vec<Table>, ParsingError>>()
}
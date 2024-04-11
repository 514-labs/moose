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
//! - Enum
//!
//! We only implemented part of the prisma schema parsing. We only support models, enums and fields. We don't support relations, or anything else for the moment

use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::{
    fmt,
    path::{Path, PathBuf},
};

use crate::framework::controller::FrameworkObject;
use diagnostics::Diagnostics;

use log::debug;
use schema_ast::ast::{Enum, Model};
use schema_ast::{
    ast::{Attribute, Field, SchemaAst, Top, WithName},
    parse_schema,
};
use serde::Serialize;

use crate::infrastructure::olap::clickhouse::mapper::arity_mapper;
use crate::project::PROJECT;

use swc_common::{self, sync::Lrc, SourceMap};
use swc_ecma_ast::{
    Decl, Expr, Module, ModuleItem, Stmt, TsEnumDecl, TsEnumMember, TsEnumMemberId,
    TsInterfaceDecl, TsKeywordTypeKind, TsType, TsTypeAnn, TsTypeRef,
};
use swc_ecma_parser::{lexer::Lexer, Capturing, Parser, StringInput, Syntax};

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

    let file_objects = prisma_ast_mapper(ast)?;

    Ok(file_objects
        .models
        .into_iter()
        .map(|data_model| mapper(data_model, path, version))
        .collect())
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
    Enum(DataEnum),
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

fn is_enum_type(string_type: &str, enums: &[DataEnum]) -> bool {
    enums.iter().any(|e| e.name == string_type)
}

fn field_to_column(f: &Field, enums: &[DataEnum]) -> Result<Column, ParsingError> {
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

fn prisma_model_to_datamodel(m: &Model, enums: &[DataEnum]) -> Result<DataModel, ParsingError> {
    let schema_name = m.name().to_string();

    let columns: Result<Vec<Column>, ParsingError> = m
        .iter_fields()
        .map(|(_id, f)| field_to_column(f, enums))
        .collect();

    let project = PROJECT.lock().unwrap();
    Ok(DataModel {
        db_name: project.clickhouse_config.db_name.to_string(),
        columns: columns?,
        name: schema_name,
    })
}

fn primsa_to_moose_enum(e: &Enum) -> DataEnum {
    let name = e.name().to_string();
    let values = e
        .iter_values()
        .map(|(_id, v)| v.name().to_string())
        .collect();
    DataEnum { name, values }
}

pub fn prisma_ast_mapper(ast: SchemaAst) -> Result<FileObjects, ParsingError> {
    let mut models = Vec::new();
    let mut enums = Vec::new();

    ast.iter_tops().try_for_each(|(_id, t)| match t {
        Top::Model(m) => {
            models.push(m);
            Ok(())
        }
        Top::Enum(e) => {
            enums.push(primsa_to_moose_enum(e));
            Ok(())
        }
        _ => Err(ParsingError::UnsupportedDataTypeError {
            type_name: "we currently only support models and enums".to_string(),
        }),
    })?;

    let parsed_models = models
        .into_iter()
        .map(|m| prisma_model_to_datamodel(m, &enums))
        .collect::<Result<Vec<DataModel>, ParsingError>>()?;

    Ok(FileObjects::new(parsed_models, enums))
}

pub fn ts_ast_mapper(ast: Module) -> Result<FileObjects, ParsingError> {
    let models = Vec::new();
    let mut enums = Vec::new();

    let mut ts_declarations = Vec::new();

    // collect all inteface and enum declarations
    ast.body.iter().for_each(|item| match item {
        ModuleItem::Stmt(Stmt::Decl(Decl::TsInterface(decl))) => {
            ts_declarations.push(decl);
        }
        ModuleItem::Stmt(Stmt::Decl(Decl::TsEnum(decl))) => {
            enums.push(ts_enum_to_data_enum(decl));
        }
        _ => {}
    });

    println!(
        "the interfaces {:#?}",
        ts_interface_to_model(ts_declarations[0], &enums)
    );

    Ok(FileObjects::new(models, enums))
}

fn ts_enum_to_data_enum(enum_decl: &TsEnumDecl) -> DataEnum {
    let name = enum_decl.id.sym.to_string();
    let values = enum_decl
        .members
        .iter()
        .map(|member| match member {
            TsEnumMember {
                id: TsEnumMemberId::Ident(ident),
                ..
            } => ident.sym.to_string(),
            _ => "unsupported".to_string(),
        })
        .collect();
    DataEnum { name, values }
}

fn ts_interface_to_model(
    interface: &TsInterfaceDecl,
    enums: &[DataEnum],
) -> Result<DataModel, ParsingError> {
    let schema_name = interface.id.sym.to_string();

    let columns: Result<Vec<Column>, ParsingError> = interface
        .body
        .body
        .iter()
        .filter_map(|field| match field {
            swc_ecma_ast::TsTypeElement::TsPropertySignature(prop) => Some({
                // match the key's sym if it's an ident
                let name = match *prop.key.clone() {
                    Expr::Ident(ident) => ident.sym.to_string(),
                    _ => return None,
                };
                // match the type of the value and return the right column type
                let data_type = match *prop.type_ann.clone().expect("no type for property") {
                    TsTypeAnn { type_ann, .. } => match *type_ann {
                        TsType::TsKeywordType(keyword) => match keyword.kind {
                            TsKeywordTypeKind::TsStringKeyword => ColumnType::String,
                            TsKeywordTypeKind::TsBooleanKeyword => ColumnType::Boolean,
                            TsKeywordTypeKind::TsNumberKeyword => ColumnType::Float,

                            _ => {
                                debug!("found a weird type{:?}", keyword);
                                ColumnType::Unsupported
                            }
                        },
                        // match the enum type as a tstyperef
                        TsType::TsTypeRef(TsTypeRef { type_name, .. }) => {
                            let enum_name = match type_name {
                                swc_ecma_ast::TsEntityName::Ident(ident) => ident.sym.to_string(),
                                _ => {
                                    debug!("found a weird type{:?}", type_name);
                                    "unsupported".to_string()
                                }
                            };
                            ColumnType::Enum(
                                enums.iter().find(|e| e.name == enum_name).unwrap().clone(),
                            )
                        }
                        _ => {
                            debug!("found a weird type{:?}", type_ann);
                            ColumnType::Unsupported
                        }
                    },
                };

                // match the optional flag
                let arity = FieldArity::Required;
                let unique = false;
                let primary_key = false;
                let default = None;

                Ok(Column {
                    name,
                    data_type,
                    arity,
                    unique,
                    primary_key,
                    default,
                })
            }),
            _ => None,
        })
        .collect();

    Ok(DataModel {
        db_name: "local".to_string(),
        columns: columns?,
        name: schema_name,
    })
}

pub fn parse_ts_schema_file(path: &Path) -> Module {
    //! Parse a typescript file as a module and return the AST for that module
    let cm: Lrc<SourceMap> = Default::default();

    let fm = cm
        .load_file(Path::new(path))
        .expect("failed to load test.ts");

    let lexer = Lexer::new(
        Syntax::Typescript(Default::default()),
        Default::default(),
        StringInput::from(&*fm),
        None,
    );

    let capturing = Capturing::new(lexer);

    let mut parser = Parser::new_from(capturing);

    let module = parser.parse_module().expect("failed to parse module");

    module
}

#[cfg(test)]
mod tests {

    use crate::framework::{
        controller::framework_object_mapper,
        schema::{parse_schema_file, parse_ts_schema_file, ts_ast_mapper},
    };

    #[test]
    fn test_parse_schema_file() {
        let current_dir = std::env::current_dir().unwrap();

        let test_file = current_dir.join("tests/psl/simple.prisma");

        let result = parse_schema_file(&test_file, "1.0", framework_object_mapper);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ts_mapper() {
        let current_dir = std::env::current_dir().unwrap();

        let test_file = current_dir.join("tests/ts/simple.ts");

        let result = parse_ts_schema_file(&test_file);
        ts_ast_mapper(result);
        assert!(true);
    }

    #[test]
    fn test_parse_typescript_file() {
        let current_dir = std::env::current_dir().unwrap();

        let test_file = current_dir.join("tests/ts/simple.ts");

        println!("{:?}", test_file);

        let result = parse_ts_schema_file(&test_file);
        println!("{:#?}", result);
        assert!(true);
    }

    #[test]
    fn test_parse_import_typescript_file() {
        let current_dir = std::env::current_dir().unwrap();

        let test_file = current_dir.join("tests/ts/import.ts");

        println!("{:?}", test_file);

        let result = parse_ts_schema_file(&test_file);
        println!("{:?}", result);
        assert!(true);
    }

    #[test]
    fn test_parse_extend_typescript_file() {
        let current_dir = std::env::current_dir().unwrap();

        let test_file = current_dir.join("tests/ts/extend.ts");

        println!("{:?}", test_file);

        let result = parse_ts_schema_file(&test_file);
        println!("{:?}", result);
        assert!(true);
    }
}

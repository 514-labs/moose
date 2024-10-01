/// Deprecated - to be removed when we remove old version of the internal app.
use std::path::{Path, PathBuf};

use crate::framework::core::infrastructure::table::{
    is_enum_type, Column, ColumnDefaults, ColumnType, DataEnum, EnumMember, EnumValue,
};
use crate::{framework::data_model::model::DataModel, framework::data_model::parser::FileObjects};
use diagnostics::{Diagnostics, FileId};
use schema_ast::ast::{Attribute, Field, WithName};
use schema_ast::{
    ast::{Enum, Model, SchemaAst, Top},
    parse_schema,
};

#[derive(Debug, thiserror::Error)]
#[error("failed to parse the prisma file")]
#[non_exhaustive]
pub enum PrismaParsingError {
    #[error("File not found: {path}")]
    FileNotFound { path: PathBuf },
    #[error("Unsupported prisma data type: {type_name}")]
    UnsupportedDataTypeError { type_name: String },
}

// TODO do this need to be public?
pub struct FieldAttributes {
    unique: bool,
    primary_key: bool,
    default: Option<ColumnDefaults>,
}

impl FieldAttributes {
    #[allow(clippy::never_loop, clippy::match_single_binding)]
    fn new(attributes: Vec<Attribute>) -> Result<FieldAttributes, PrismaParsingError> {
        let unique: bool = false;
        let mut primary_key: bool = false;
        let default: Option<ColumnDefaults> = None;

        // TODO: Implement default values and primary keys once we have the ingestion table architecture setup
        for attribute in attributes {
            match attribute.name() {
                "id" => primary_key = true,
                _ => {
                    return Err(PrismaParsingError::UnsupportedDataTypeError {
                        type_name: attribute.name().to_string(),
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

pub fn extract_data_model_from_file(
    path: &Path,
    version: &str,
) -> Result<FileObjects, PrismaParsingError> {
    let schema_file = std::fs::read_to_string(path)
        .map_err(|_| PrismaParsingError::FileNotFound { path: path.into() })?;
    let ast = parse_schema(&schema_file, &mut Diagnostics::default(), FileId::ZERO);
    prisma_ast_to_internal_ast(path.to_path_buf(), version, ast)
}

fn prisma_ast_to_internal_ast(
    file_path: PathBuf,
    version: &str,
    ast: SchemaAst,
) -> Result<FileObjects, PrismaParsingError> {
    let mut models = Vec::new();
    let mut enums = Vec::new();

    ast.iter_tops().try_for_each(|(_id, t)| match t {
        Top::Model(m) => {
            models.push(m);
            Ok(())
        }
        Top::Enum(e) => {
            enums.push(to_moose_enum(e));
            Ok(())
        }
        _ => Err(PrismaParsingError::UnsupportedDataTypeError {
            type_name: format!("{:?}", t),
        }),
    })?;

    let parsed_models = models
        .into_iter()
        .map(|m| prisma_model_to_datamodel(&file_path, version, m, &enums))
        .collect::<Result<Vec<DataModel>, PrismaParsingError>>()?;

    Ok(FileObjects::new(parsed_models, enums))
}

fn prisma_model_to_datamodel(
    file_path: &Path,
    version: &str,
    m: &Model,
    enums: &[DataEnum],
) -> Result<DataModel, PrismaParsingError> {
    let schema_name = m.name().to_string();

    let columns: Result<Vec<Column>, PrismaParsingError> = m
        .iter_fields()
        .map(|(_id, f)| field_to_column(f, enums))
        .collect();

    Ok(DataModel {
        columns: columns?,
        name: schema_name,
        config: Default::default(),
        abs_file_path: file_path.to_path_buf(),
        version: version.to_string(),
    })
}

fn to_moose_enum(e: &Enum) -> DataEnum {
    let name = e.name().to_string();

    let mut values = Vec::new();

    let mut integer_increment = 1;
    for (_, v) in e.iter_values() {
        let enum_index = integer_increment;
        integer_increment += 1;

        values.push(EnumMember {
            name: v.name().to_string(),
            value: EnumValue::Int(enum_index),
        });
    }

    DataEnum { name, values }
}

fn field_to_column(f: &Field, enums: &[DataEnum]) -> Result<Column, PrismaParsingError> {
    let attributes = FieldAttributes::new(f.attributes.clone())?;

    if f.arity.is_list() {
        return Err(PrismaParsingError::UnsupportedDataTypeError {
            type_name: "List".to_string(),
        });
    }

    let optional = f.arity.is_optional();

    match &f.field_type {
        schema_ast::ast::FieldType::Supported(ft) => Ok(Column {
            name: f.name().to_string(),
            data_type: match is_enum_type(&ft.name, enums) {
                true => ColumnType::Enum(enums.iter().find(|e| e.name == ft.name).unwrap().clone()),
                false => map_column_string_type_to_column_type(&ft.name)?,
            },
            required: !optional,
            unique: attributes.unique,
            primary_key: attributes.primary_key,
            // Prisma getting deprecated
            jwt: false,
            default: attributes.default,
        }),
        schema_ast::ast::FieldType::Unsupported(x, _) => {
            Err(PrismaParsingError::UnsupportedDataTypeError {
                type_name: x.to_string(),
            })
        }
    }
}

fn map_column_string_type_to_column_type(
    string_type: &str,
) -> Result<ColumnType, PrismaParsingError> {
    match string_type {
        "String" => Ok(ColumnType::String),
        "Boolean" => Ok(ColumnType::Boolean),
        "Int" => Ok(ColumnType::Int),
        "BigInt" => Ok(ColumnType::BigInt),
        "Float" => Ok(ColumnType::Float),
        "Decimal" => Ok(ColumnType::Decimal),
        "DateTime" => Ok(ColumnType::DateTime),
        _ => Err(PrismaParsingError::UnsupportedDataTypeError {
            type_name: string_type.to_string(),
        }),
    }
}

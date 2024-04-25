use crate::{
    framework::schema::{is_enum_type, ColumnType},
    project::PROJECT,
};
use log::debug;
use std::path::{Path, PathBuf};
use swc_common::{self, sync::Lrc, SourceMap};
use swc_ecma_ast::{
    Decl, ExportDecl, Expr, Lit, Module, ModuleDecl, ModuleItem, Stmt, TsEnumDecl, TsEnumMemberId,
    TsInterfaceDecl, TsKeywordTypeKind, TsPropertySignature, TsType, TsTypeAnn, TsTypeRef,
};
use swc_ecma_parser::{lexer::Lexer, Capturing, Parser, StringInput, Syntax};

use crate::framework::schema::{Column, DataEnum, DataModel, FileObjects};

#[derive(Debug, Clone, thiserror::Error)]
#[error("Failed to parse the typescript file")]
#[non_exhaustive]
pub enum TypescriptParsingError {
    FileNotFound {
        path: PathBuf,
    },
    #[error("Typescript Parser - Unsupported data type: {type_name}")]
    UnsupportedDataTypeError {
        type_name: String,
    },
    #[error("Typescript Parser - Invalid typescript file, please refer to the documentation for an example of a valid typescript file")]
    InvalidTypescriptFile,
    OtherError,
}

pub fn extract_data_model_from_file(path: &Path) -> Result<FileObjects, TypescriptParsingError> {
    let ast = parse_ts_module(path)?;
    extract_data_models_from_ast(ast)
}

fn parse_ts_module(path: &Path) -> Result<Module, TypescriptParsingError> {
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

    parser
        .parse_module()
        .map_err(|_| TypescriptParsingError::InvalidTypescriptFile)
}

fn extract_data_models_from_ast(ast: Module) -> Result<FileObjects, TypescriptParsingError> {
    let mut enums = Vec::new();

    let mut ts_declarations = Vec::new();

    // collect all interface and enum declarations
    for item in ast.body.iter() {
        match item {
            ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl { decl, .. }))
            | ModuleItem::Stmt(Stmt::Decl(decl)) => {
                match decl {
                    Decl::TsInterface(decl) => {
                        ts_declarations.push(decl);
                    }
                    Decl::TsEnum(decl) => {
                        enums.push(enum_to_data_enum(decl));
                    }
                    // We ignore all other declarations
                    _ => continue,
                }
            }
            // We ignore all other top module items
            _ => continue,
        }
    }

    let parsed_models = ts_declarations
        .into_iter()
        .map(|m| interface_to_model(m, &enums))
        .collect::<Result<Vec<DataModel>, TypescriptParsingError>>()?;

    Ok(FileObjects::new(parsed_models, enums))
}

fn enum_to_data_enum(enum_decl: &TsEnumDecl) -> DataEnum {
    let name = enum_decl.id.sym.to_string();
    let values = enum_decl
        .members
        .iter()
        .map(|member| match &member.id {
            TsEnumMemberId::Ident(ident) => ident.sym.to_string(),
            TsEnumMemberId::Str(str) => str.value.to_string(),
        })
        .collect();
    DataEnum { name, values }
}

fn interface_to_model(
    interface: &TsInterfaceDecl,
    enums: &[DataEnum],
) -> Result<DataModel, TypescriptParsingError> {
    let schema_name = interface.id.sym.to_string();

    let columns: Result<Vec<Column>, TypescriptParsingError> = interface
        .body
        .body
        .iter()
        .filter_map(|field| match field {
            swc_ecma_ast::TsTypeElement::TsPropertySignature(prop) => {
                Some(parse_property_signature(prop, enums))
            }
            _ => None,
        })
        .collect();

    let project = PROJECT.lock().unwrap();
    Ok(DataModel {
        db_name: project.clickhouse_config.db_name.to_string(),
        columns: columns?,
        name: schema_name,
    })
}

fn parse_property_signature(
    prop: &TsPropertySignature,
    enums: &[DataEnum],
) -> Result<Column, TypescriptParsingError> {
    let mut primary_key = false;
    let unique = false;
    let default = None;

    // match the key's sym if it's an ident
    let name = match *prop.key.clone() {
        Expr::Ident(ident) => ident.sym.to_string(),
        Expr::Lit(Lit::Str(str)) => str.value.to_string(),
        _ => {
            return Err(TypescriptParsingError::UnsupportedDataTypeError {
                type_name: format!("{:?}", *prop.key.clone()),
            })
        }
    };

    // match the type of the value and return the right column type
    let TsTypeAnn { type_ann, .. } = *prop
        .type_ann
        .clone()
        .ok_or(TypescriptParsingError::InvalidTypescriptFile {})?;

    let data_type = parse_type_ann(type_ann, enums, &mut primary_key)?;

    debug!(
        "name: {}, data_type: {:?}, primary_key: {:?}",
        name, data_type, primary_key
    );

    Ok(Column {
        name,
        data_type,
        required: !prop.optional,
        unique,
        primary_key,
        default,
    })
}

fn parse_type_ann(
    type_ann: Box<TsType>,
    enums: &[DataEnum],
    primary_key: &mut bool,
) -> Result<ColumnType, TypescriptParsingError> {
    match *type_ann {
        TsType::TsKeywordType(keyword) => ts_parse_keyword_type(keyword),
        TsType::TsArrayType(array_type) => {
            let inner_type =
                parse_type_ann(Box::new(*array_type.elem_type.clone()), enums, primary_key)?;
            Ok(ColumnType::Array(Box::new(inner_type)))
        }
        TsType::TsTypeRef(type_ref) => parse_type_ref(type_ref, enums, primary_key),
        _ => Err(TypescriptParsingError::UnsupportedDataTypeError {
            type_name: format!("{:?}", type_ann),
        }),
    }
}

fn ts_parse_keyword_type(
    keyword: swc_ecma_ast::TsKeywordType,
) -> Result<ColumnType, TypescriptParsingError> {
    match keyword.kind {
        TsKeywordTypeKind::TsStringKeyword => Ok(ColumnType::String),
        TsKeywordTypeKind::TsBooleanKeyword => Ok(ColumnType::Boolean),
        TsKeywordTypeKind::TsNumberKeyword => Ok(ColumnType::Float),
        _ => Err(TypescriptParsingError::UnsupportedDataTypeError {
            type_name: format!("{:?}", keyword.kind),
        }),
    }
}

fn parse_type_ref(
    type_ref: TsTypeRef,
    enums: &[DataEnum],
    primary_key: &mut bool,
) -> Result<ColumnType, TypescriptParsingError> {
    let type_ref_name = match type_ref.type_name {
        swc_ecma_ast::TsEntityName::Ident(ident) => Ok(ident.sym.to_string()),
        _ => Err(TypescriptParsingError::UnsupportedDataTypeError {
            type_name: format!("{:?}", type_ref.type_name),
        }),
    }?;

    if type_ref_name == "Key" {
        *primary_key = true;

        match type_ref.type_params {
            Some(params) => {
                if let Some(param) = params.params.first() {
                    return parse_type_ann(param.clone(), enums, primary_key);
                } else {
                    return Err(TypescriptParsingError::UnsupportedDataTypeError {
                        type_name: "no type for key".to_string(),
                    });
                }
            }
            None => {
                return Err(TypescriptParsingError::UnsupportedDataTypeError {
                    type_name: "no type for key".to_string(),
                });
            }
        }
    }

    if type_ref_name == "Date" {
        Ok(ColumnType::DateTime)
    } else if is_enum_type(&type_ref_name, enums) {
        Ok(ColumnType::Enum(
            enums
                .iter()
                .find(|e| e.name == type_ref_name)
                .unwrap()
                .clone(),
        ))
    } else {
        Err(TypescriptParsingError::UnsupportedDataTypeError {
            type_name: type_ref_name,
        })
    }
}

#[cfg(test)]
mod tests {

    use crate::framework::{
        controller::framework_object_mapper, schema::parse_data_model_file,
        typescript::parser::extract_data_model_from_file,
    };

    #[test]
    fn test_parse_schema_file() {
        let current_dir = std::env::current_dir().unwrap();

        let test_file = current_dir.join("tests/psl/simple.prisma");

        let result = parse_data_model_file(&test_file, "1.0", framework_object_mapper);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ts_mapper() {
        let current_dir = std::env::current_dir().unwrap();

        let test_file = current_dir.join("tests/ts/simple.ts");

        let result = extract_data_model_from_file(&test_file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_typescript_file() {
        let current_dir = std::env::current_dir().unwrap();

        let test_file = current_dir.join("tests/ts/simple.ts");

        let result = extract_data_model_from_file(&test_file);

        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_import_typescript_file() {
        let current_dir = std::env::current_dir().unwrap();

        let test_file = current_dir.join("tests/ts/import.ts");

        let result = extract_data_model_from_file(&test_file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_extend_typescript_file() {
        let current_dir = std::env::current_dir().unwrap();

        let test_file = current_dir.join("tests/ts/extend.m.ts");

        let result = extract_data_model_from_file(&test_file);
        assert!(result.is_ok());
    }
}

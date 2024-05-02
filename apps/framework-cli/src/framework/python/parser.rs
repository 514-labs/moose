//! # Python Parser
//! This module is responsible for parsing the python schema file and extracting the data model from it
//!
//! There are two main capabilities in this module:
//! 1. Extracting python from a schema file and turning it an AST
//! 2. Mapping that AST into file objects
//!
//! ## File Objects
//! The file objects are all the data model objects that are extracted from the python schema file
//! and it's associsted supporting objects such as enums.

use std::path::PathBuf;

use rustpython_parser::{
    ast::{self, Expr, Identifier, Stmt},
    Parse,
};

use crate::{
    framework::schema::{Column, ColumnDefaults, ColumnType, DataEnum as FrameworkEnum, DataModel},
    project::PROJECT,
};

#[derive(Debug, Clone, thiserror::Error)]
#[error("Failed to parse the python file")]
#[non_exhaustive]
pub enum PythonParserError {
    FileNotFound {
        path: PathBuf,
    },
    #[error("Python Parser - Unsupported data type: {type_name}")]
    UnsupportedDataTypeError {
        type_name: String,
    },
    #[error("Python Parser - Error parsing class node: {message}")]
    ClassParseError {
        message: String,
    },
    #[error("Python Parser - Error parsing enum node: {message}")]
    EnumParseError {
        message: String,
    },
    #[error("Python Parser - Invalid python file, please refer to the documentation for an example of a valid python file")]
    InvalidPythonFile,
    OtherError,
}

/// # First pass: AST processing functions
/// These functions are responsible for processing the AST and extracting the relevant nodes to
/// turn into data model objects

/// ## Get AST from File
/// This function reads the python schema file and turns it into an AST
fn get_ast_from_file(path: &PathBuf) -> Result<ast::Suite, PythonParserError> {
    // Read the schema file
    // todo!("Add file validation like checking extension and other checks");
    let schema_file =
        std::fs::read_to_string(path).map_err(|_| PythonParserError::InvalidPythonFile)?;

    // Parse the schema file into an AST
    let ast = ast::Suite::parse(&schema_file, "<embedded>")
        .map_err(|_| PythonParserError::InvalidPythonFile)?;

    Ok(ast)
}

/// ## Enum AST Nodes
/// This function extracts all the enum nodes from the AST
fn get_enum_ast_nodes(ast: &ast::Suite) -> Vec<&ast::Stmt> {
    ast.iter()
        .filter(|node| match node {
            Stmt::ClassDef(class_def) => {
                if class_def.bases.is_empty() {
                    false
                } else {
                    let enum_bases: Vec<_> = class_def
                        .bases
                        .iter()
                        .filter(|base| match base {
                            Expr::Name(name) => name.id == Identifier::new("Enum"),
                            _ => false,
                        })
                        .collect();

                    !enum_bases.is_empty()
                }
            }
            _ => false,
        })
        .collect()
}

/// ## Non Enum Class AST Nodes
/// This function extracts all the class nodes that are not enums from the AST
fn get_non_enum_class_ast_nodes(ast: &ast::Suite) -> Vec<&ast::Stmt> {
    ast.iter()
        .filter(|node| match node {
            Stmt::ClassDef(class_def) => {
                if class_def.bases.is_empty() {
                    true
                } else {
                    let enum_bases: Vec<_> = class_def
                        .bases
                        .iter()
                        .filter(|base| match base {
                            Expr::Name(name) => name.id == Identifier::new("Enum"),
                            _ => false,
                        })
                        .collect();

                    enum_bases.is_empty()
                }
            }
            _ => false,
        })
        .collect()
}

/// # Second pass processing: Turn classes and enums into framework data models and framework enums
/// These functions are responsible for turning the AST nodes into data model and enum objects that
/// can be used by the rest of the system.
///
/// ## Python Enum to Framework Enum
/// This function takes a python enum AST node and turns it into a framework enum object
fn python_enum_to_framework_enum(
    enum_node: &ast::Stmt,
) -> Result<FrameworkEnum, PythonParserError> {
    // Get the name of the enum
    let enum_name = match enum_node {
        Stmt::ClassDef(class_def) => class_def.name.to_string(),
        _ => {
            return Err(PythonParserError::EnumParseError {
                message: "Invalid enum node".to_string(),
            })
        }
    };

    Ok(FrameworkEnum {
        name: enum_name,
        values: vec![],
    })
}

/// ## Python Class to Framework Data Model
/// This function takes a python class AST node and turns it into a framework data model object
fn python_class_to_framework_datamodel(
    class_node: &ast::Stmt,
    enums: &[FrameworkEnum],
) -> Result<DataModel, PythonParserError> {
    let project = PROJECT.lock().unwrap();

    let class_name = match class_node {
        Stmt::ClassDef(class_def) => class_def.name.to_string(),
        _ => {
            return Err(PythonParserError::ClassParseError {
                message: "Invalid class node".to_string(),
            })
        }
    };

    let body_nodes = get_class_body_nodes(class_node)?;

    let columns = body_nodes
        .iter()
        .map(|body_node| body_node_to_column(body_node, enums))
        .collect::<Result<Vec<Column>, PythonParserError>>()?;

    Ok(DataModel {
        db_name: project.clickhouse_config.db_name.to_string(),
        columns,
        name: class_name,
    })
}

/// ### Get the body nodes for the class
/// This function gets the body nodes for a class node
fn get_class_body_nodes(class_node: &ast::Stmt) -> Result<&Vec<ast::Stmt>, PythonParserError> {
    match class_node {
        Stmt::ClassDef(class_def) => Ok(&class_def.body),
        _ => Err(PythonParserError::ClassParseError {
            message: "Invalid class node".to_string(),
        }),
    }
}

fn name_node_to_base_column_type(
    name_node: ast::ExprName,
) -> Result<ColumnType, PythonParserError> {
    match name_node.id.to_string().as_str() {
        "str" => Ok(ColumnType::String),
        "int" => Ok(ColumnType::Int),
        "float" => Ok(ColumnType::Float),
        "bool" => Ok(ColumnType::Boolean),
        "datetime" => Ok(ColumnType::DateTime),
        _ => Err(PythonParserError::UnsupportedDataTypeError {
            type_name: name_node.id.to_string(),
        }),
    }
}

fn class_attribute_node_to_column_builder(
    attribute_node: &ast::StmtAnnAssign,
    enums: &[FrameworkEnum],
) -> Result<ColumnBuilder, PythonParserError> {
    let mut column = ColumnBuilder::default();
    match *attribute_node.target.clone() {
        Expr::Name(name) => {
            column.name = Some(name.id.to_string());
        }
        _ => {
            return Err(PythonParserError::ClassParseError {
                message: "Name not found".to_string(),
            })
        }
    }
    match *attribute_node.annotation.clone() {
        Expr::Name(name) => {
            let col_type = match name_node_to_base_column_type(name.clone()) {
                Ok(col_type) => col_type,
                Err(e) => enums
                    .iter()
                    .find(|enum_item| name.id == enum_item.name)
                    .map(|enum_item| ColumnType::Enum(enum_item.clone()))
                    .ok_or(e)?,
            };
            column.required = Some(true);
            column.data_type = Some(col_type);
        }
        Expr::Subscript(subscript) => match &*subscript.value {
            Expr::Name(name) => match name.id.to_string().as_str() {
                "List" => {
                    let col_type = ColumnType::Array(match &*subscript.slice {
                        Expr::Name(name) => Box::new(name_node_to_base_column_type(name.clone())?),
                        _ => {
                            return Err(PythonParserError::UnsupportedDataTypeError {
                                type_name: "Unsupported data type".to_string(),
                            })
                        }
                    });
                    column.data_type = Some(col_type);
                }
                "Key" => match &*subscript.slice {
                    Expr::Name(name) => {
                        let col_type = name_node_to_base_column_type(name.clone())?;
                        column.data_type = Some(col_type);
                        column.required = Some(true);
                        column.primary_key = Some(true);
                    }
                    _ => {
                        return Err(PythonParserError::UnsupportedDataTypeError {
                            type_name: "Unsupported data type".to_string(),
                        })
                    }
                },
                "Optional" => match &*subscript.slice {
                    Expr::Name(name) => {
                        let col_type = name_node_to_base_column_type(name.clone())?;
                        column.data_type = Some(col_type);
                        column.required = Some(false);
                    }
                    _ => {
                        return Err(PythonParserError::UnsupportedDataTypeError {
                            type_name: "Unsupported data type".to_string(),
                        })
                    }
                },

                _ => {
                    return Err(PythonParserError::UnsupportedDataTypeError {
                        type_name: "Unsupported data type".to_string(),
                    })
                }
            },
            _ => {
                return Err(PythonParserError::UnsupportedDataTypeError {
                    type_name: "Unsupported data type".to_string(),
                })
            }
        },

        _ => {
            return Err(PythonParserError::UnsupportedDataTypeError {
                type_name: "Unsupported data type".to_string(),
            })
        }
    }

    Ok(column)
}

#[derive(Default)]
struct ColumnBuilder {
    name: Option<String>,
    data_type: Option<ColumnType>,
    required: Option<bool>,
    unique: Option<bool>,
    primary_key: Option<bool>,
    default: Option<ColumnDefaults>,
}

impl ColumnBuilder {
    fn build(self) -> Result<Column, PythonParserError> {
        let name = self.name.ok_or(PythonParserError::ClassParseError {
            message: "Class builder property, name, not set properly".to_string(),
        })?;

        let data_type = self
            .data_type
            .ok_or(PythonParserError::UnsupportedDataTypeError {
                type_name: "Class builder property, data_type, unsupported or not set properly"
                    .to_string(),
            })?;

        let required = self.required.unwrap_or(true);

        let unique = self.unique.unwrap_or(false);

        let primary_key = self.primary_key.unwrap_or(false);

        Ok(Column {
            name,
            data_type,
            required,
            unique,
            primary_key,
            default: self.default,
        })
    }
}

fn body_node_to_column(
    body_node: &ast::Stmt,
    enums: &[FrameworkEnum],
) -> Result<Column, PythonParserError> {
    match body_node {
        Stmt::AnnAssign(ann_assign) => {
            let column_builder = class_attribute_node_to_column_builder(ann_assign, enums);
            let column = column_builder?.build()?;

            Ok(column)
        }
        _ => {
            println!("failed parsing {:?} ", body_node);
            Err(PythonParserError::UnsupportedDataTypeError {
                type_name: "Unsupported data type".to_string(),
            })
        }
    }
}

pub fn extract_data_model_from_file(path: &PathBuf) -> Result<(), PythonParserError> {
    // todo!("Handle the enums in the class parsing");

    // Parse the schema file into an AST
    let ast = get_ast_from_file(path)?;

    // Get the enums from the AST
    let python_enums: Vec<&ast::Stmt> = get_enum_ast_nodes(&ast);

    // Get the non-enum classes from the AST
    let python_classes: Vec<&ast::Stmt> = get_non_enum_class_ast_nodes(&ast);

    let framework_enums = python_enums
        .iter()
        .map(|enum_node| python_enum_to_framework_enum(enum_node))
        .collect::<Result<Vec<FrameworkEnum>, PythonParserError>>()?;

    let data_models: Vec<DataModel> = python_classes
        .iter()
        .map(|class_node| python_class_to_framework_datamodel(class_node, &framework_enums))
        .collect::<Result<Vec<DataModel>, PythonParserError>>()?;

    // Process each of the class nodes
    println!("{:#?}", data_models);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_simple_python_file_path() -> std::path::PathBuf {
        let current_dir = std::env::current_dir().unwrap();
        current_dir.join("tests/python/simple.py")
    }

    #[test]
    fn test_parse_schema_file() {
        let test_file = get_simple_python_file_path();

        let result = extract_data_model_from_file(&test_file);

        println!("{:?}", result);

        assert!(result.is_ok());
    }

    #[test]
    fn has_right_number_of_enums() {
        let test_file = get_simple_python_file_path();

        let ast = get_ast_from_file(&test_file).unwrap();

        let enums = get_enum_ast_nodes(&ast);

        assert_eq!(enums.len(), 1);
    }

    #[test]
    fn has_right_number_of_non_enum_classes() {
        let test_file = get_simple_python_file_path();

        let ast = get_ast_from_file(&test_file).unwrap();

        let classes = get_non_enum_class_ast_nodes(&ast);

        assert_eq!(classes.len(), 1);
    }

    #[test]
    fn has_right_number_of_attributes() {
        let test_file = get_simple_python_file_path();

        let ast = get_ast_from_file(&test_file).unwrap();

        let classes = get_non_enum_class_ast_nodes(&ast);

        let class_node = classes.first().unwrap();

        let body_nodes = get_class_body_nodes(class_node).unwrap();

        assert_eq!(body_nodes.len(), 7);
    }
}

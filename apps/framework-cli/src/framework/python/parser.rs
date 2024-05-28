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
    ast::{self, Expr, ExprName, Identifier, Stmt, StmtClassDef},
    Parse,
};

use crate::framework::data_model::schema::{
    Column, ColumnType, DataEnum as FrameworkEnum, DataModel, Nested,
};

use crate::framework::python::utils::ColumnBuilder;

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
fn get_enum_ast_nodes(ast: &ast::Suite) -> Vec<&StmtClassDef> {
    ast.iter()
        .filter_map(|node| match node {
            Stmt::ClassDef(class_def) => {
                if class_def.bases.is_empty() {
                    None
                } else {
                    let enum_bases: Vec<_> = class_def
                        .bases
                        .iter()
                        .filter(|base| match base {
                            Expr::Name(name) => name.id == Identifier::new("Enum"),
                            _ => false,
                        })
                        .collect();

                    match enum_bases.is_empty() {
                        true => None,
                        false => Some(class_def),
                    }
                }
            }
            _ => None,
        })
        .collect()
}

/// ## Non Enum Class AST Nodes
/// This function extracts all the class nodes that are not enums from the AST
fn get_non_enum_class_ast_nodes(ast: &ast::Suite) -> Vec<&StmtClassDef> {
    ast.iter()
        .filter_map(|node| match node {
            Stmt::ClassDef(class_def) => {
                if class_def.bases.is_empty() {
                    Some(class_def)
                } else {
                    let enum_bases: Vec<_> = class_def
                        .bases
                        .iter()
                        .filter(|base| match base {
                            Expr::Name(name) => name.id == Identifier::new("Enum"),
                            _ => false,
                        })
                        .collect();

                    match enum_bases.is_empty() {
                        true => Some(class_def),
                        false => None,
                    }
                }
            }
            _ => None,
        })
        .collect()
}

/// # Recursively collect nested classes for a given class node
fn collect_nested_classes(
    class: &StmtClassDef,
    classes: &Vec<&StmtClassDef>,
    collector: &mut Vec<Identifier>,
) {
    let body_nodes = class.clone().body;

    for body_node in body_nodes {
        match body_node {
            Stmt::AnnAssign(assignment) => {
                let id = match *assignment.annotation.clone() {
                    Expr::Name(name) => name.id,
                    _ => Identifier::new(""),
                };

                let class_node = classes.iter().find(|class| class.name == id);

                match class_node {
                    Some(cn) => {
                        collector.push(id.clone());
                        collect_nested_classes(cn, classes, collector);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

pub fn get_nested_classes(python_classes: &Vec<&StmtClassDef>) -> Vec<Identifier> {
    let mut nested_classes_collector: Vec<Identifier> = Vec::new();

    for class_node in python_classes {
        collect_nested_classes(class_node, python_classes, &mut nested_classes_collector);
    }

    nested_classes_collector
}

/// # Second pass processing: Turn classes and enums into framework data models and framework enums
/// These functions are responsible for turning the AST nodes into data model and enum objects that
/// can be used by the rest of the system.
///
/// ## Python Enum to Framework Enum
/// This function takes a python enum AST node and turns it into a framework enum object
fn python_enum_to_framework_enum(
    enum_node: &StmtClassDef,
) -> Result<FrameworkEnum, PythonParserError> {
    Ok(FrameworkEnum {
        name: enum_node.name.to_string(),
        values: vec![],
    })
}

/// ## Python Class to Framework Data Model
/// This function takes a python class AST node and turns it into a framework data model object
fn python_class_to_framework_datamodel(
    class_node: &StmtClassDef,
    framework_enums: &[FrameworkEnum],
    python_classes: &[&StmtClassDef],
    nested_classes: &[Identifier],
) -> Result<DataModel, PythonParserError> {
    let class_name = class_node.clone().name.to_string();
    let body_nodes = class_node.clone().body;

    let columns = body_nodes
        .iter()
        .map(|body_node| {
            body_node_to_column(body_node, framework_enums, python_classes, nested_classes)
        })
        .collect::<Result<Vec<Column>, PythonParserError>>()?;

    Ok(DataModel {
        columns,
        name: class_name,
        config: Default::default(),
    })
}

/// ### Name Node to Base Column Type
/// This function converts an AST expr name to a base column type
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

/// # Attempt to turn a name into an enum or a nested class
fn attempt_enum(name: &ExprName, enums: &[FrameworkEnum]) -> Result<ColumnType, PythonParserError> {
    enums
        .iter()
        .find(|enum_item| name.id == enum_item.name)
        .map(|enum_item| ColumnType::Enum(enum_item.clone()))
        .ok_or(PythonParserError::ClassParseError {
            message: "Failed to parse enum type".to_string(),
        })
}

/// # Attempt to turn a nested class into a nested column type
fn attempt_nested_class(
    name: &ExprName,
    enums: &[FrameworkEnum],
    python_classes: &[&StmtClassDef],
    nested_classes: &[Identifier],
) -> Result<ColumnType, PythonParserError> {
    if let Some(class_node) = python_classes
        .iter()
        .find(|class_node| class_node.name == name.id)
    {
        let body_nodes = &class_node.body;
        let col_type = body_nodes
            .iter()
            .map(|body_node| body_node_to_column(body_node, enums, python_classes, nested_classes))
            .collect::<Result<Vec<Column>, PythonParserError>>()
            .map(|columns| {
                ColumnType::Nested(Nested {
                    name: name.id.to_string(),
                    columns,
                })
            });
        col_type
    } else {
        Err(PythonParserError::ClassParseError {
            message: "Failed to parse nested class type".to_string(),
        })
    }
}

fn handle_complex_named_type(
    name: ExprName,
    enums: &[FrameworkEnum],
    python_classes: &[&StmtClassDef],
    nested_classes: &[Identifier],
) -> Result<ColumnType, PythonParserError> {
    match attempt_enum(&name, enums) {
        Ok(col_type) => Ok(col_type),
        Err(_) => attempt_nested_class(&name, enums, python_classes, nested_classes),
    }
}

/// # Class Attribute Node to Column Builder
/// This function processes a class attribute node and turns it into a column builder
fn class_attribute_node_to_column_builder(
    attribute_node: &ast::StmtAnnAssign,
    enums: &[FrameworkEnum],
    python_classes: &[&StmtClassDef],
    nested_classes: &[Identifier],
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
        // Handles the case where the annotation is a straight name such as str, int, float, bool, datetime
        // this also includes enums and other custom types that are named in the schema, such as nested classes
        Expr::Name(name) => {
            process_name_node(name, enums, python_classes, nested_classes, &mut column)?
        }
        // Handles the case where the annotation is a subscript such as List[str], Optional[int], Key[str]
        Expr::Subscript(subscript) => process_subscript_node(subscript, &mut column)?,

        _ => {
            return Err(PythonParserError::UnsupportedDataTypeError {
                type_name: "Unsupported data type".to_string(),
            })
        }
    }

    Ok(column)
}

/// # Process subscript node
/// This function processes a subscript node and adds the relevant properties to the column builder
fn process_subscript_node(
    subscript: ast::ExprSubscript,
    column: &mut ColumnBuilder,
) -> Result<(), PythonParserError> {
    match &*subscript.value {
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
    };
    Ok(())
}

/// # Add the column builder properties to the column builder that are available in the name node
fn process_name_node(
    name: ExprName,
    enums: &[FrameworkEnum],
    python_classes: &[&StmtClassDef],
    nested_classes: &[Identifier],
    column: &mut ColumnBuilder,
) -> Result<(), PythonParserError> {
    let col_type = match name_node_to_base_column_type(name.clone()) {
        Ok(col_type) => col_type,
        Err(_e) => handle_complex_named_type(name, enums, python_classes, nested_classes)?,
    };
    column.required = Some(true);
    column.data_type = Some(col_type);
    Ok(())
}

/// # Construct and build a column from a class body node
/// These body nodes are statements that are part of a class definition and are used to define the
/// attributes of the class. This function is responsible for turning these body nodes into column
/// objects.
fn body_node_to_column(
    body_node: &ast::Stmt,
    enums: &[FrameworkEnum],
    python_classes: &[&StmtClassDef],
    nested_classes: &[Identifier],
) -> Result<Column, PythonParserError> {
    match body_node {
        Stmt::AnnAssign(ann_assign) => {
            let column_builder = class_attribute_node_to_column_builder(
                ann_assign,
                enums,
                python_classes,
                nested_classes,
            );
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

pub fn extract_data_model_from_file(path: &PathBuf) -> Result<Vec<DataModel>, PythonParserError> {
    // Parse the schema file into an AST
    let ast = get_ast_from_file(path)?;

    // Get the enums from the AST
    let python_enums: Vec<&StmtClassDef> = get_enum_ast_nodes(&ast);

    // Get the non-enum classes from the AST
    let python_classes: Vec<&StmtClassDef> = get_non_enum_class_ast_nodes(&ast);

    // Process the python enums into framework enums
    let framework_enums = python_enums
        .iter()
        .map(|enum_node| python_enum_to_framework_enum(enum_node))
        .collect::<Result<Vec<FrameworkEnum>, PythonParserError>>()?;

    // Get the nested classes found in the python file
    let nested_classes = get_nested_classes(&python_classes);

    // Process the python classes into framework data models (includes all declared classes as datamodels)
    let data_models: Vec<DataModel> = python_classes
        .iter()
        .map(|class_node| {
            python_class_to_framework_datamodel(
                class_node,
                &framework_enums,
                &python_classes,
                &nested_classes,
            )
        })
        .collect::<Result<Vec<DataModel>, PythonParserError>>()?;

    // Remove the nested classes from the top level data models
    let data_models = data_models
        .iter()
        .filter(|data_model| !nested_classes.contains(&Identifier::new(&data_model.name)))
        .cloned()
        .collect();

    Ok(data_models)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_simple_python_file_path() -> std::path::PathBuf {
        let current_dir = std::env::current_dir().unwrap();
        current_dir.join("tests/python/models/simple.py")
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

        assert_eq!(classes.len(), 2);
    }

    #[test]
    fn creates_right_number_if_data_models() {
        let test_file = get_simple_python_file_path();

        let data_models = extract_data_model_from_file(&test_file).unwrap();

        assert_eq!(data_models.len(), 1);
    }

    #[test]
    fn data_model_has_right_number_of_nested_objects() {
        // checks that the data model has one nested object column
        let test_file = get_simple_python_file_path();

        let data_models = extract_data_model_from_file(&test_file).unwrap();

        let data_model = data_models.first().unwrap();

        // get the nested object columns
        let nested_columns = data_model
            .columns
            .iter()
            .filter(|column| matches!(column.data_type, ColumnType::Nested(_)))
            .collect::<Vec<&Column>>();

        assert_eq!(nested_columns.len(), 1);
    }

    #[test]
    fn has_right_number_of_attributes() {
        // checks that all the parsed classes have the right number of attributes

        let test_file = get_simple_python_file_path();

        let ast = get_ast_from_file(&test_file).unwrap();

        let classes = get_non_enum_class_ast_nodes(&ast);

        // get number of attributes from all the classes
        let body_nodes_attribute_counts = classes
            .iter()
            .map(|class_node| class_node.body.clone().len())
            .collect::<Vec<usize>>();

        assert_eq!(body_nodes_attribute_counts, [2, 8]);
    }
}

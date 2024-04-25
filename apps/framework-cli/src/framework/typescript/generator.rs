use convert_case::{Case, Casing};
use log::debug;
use std::fs;
use std::{collections::HashMap, ffi::OsStr, fmt, path::PathBuf};

use super::templates::{self, IndexTemplate, PackageJsonTemplate, TsConfigTemplate};
use crate::{
    framework::{
        controller::FrameworkObject,
        schema::{ColumnType, Table},
    },
    project::Project,
    utilities::{constants::TS_INTERFACE_GENERATE_EXT, package_managers, system},
};

#[derive(Debug, thiserror::Error)]
#[error("Failed to generate Typescript code")]
#[non_exhaustive]
pub enum TypescriptGeneratorError {
    #[error("Typescript Code Generator - Unsupported data type: {type_name}")]
    UnsupportedDataTypeError { type_name: String },
}

#[derive(Debug, Clone)]
pub struct SendFunction {
    pub interface: TypescriptInterface,
    pub server_url: String,
    pub api_route_name: String,
}

impl SendFunction {
    pub fn new(interface: TypescriptInterface, server_url: String, api_route_name: String) -> Self {
        Self {
            interface,
            server_url,
            api_route_name,
        }
    }
    pub fn file_name(&self) -> String {
        self.interface.send_function_file_name().to_string()
    }

    pub fn file_name_with_extension(&self) -> String {
        format!("{}.ts", self.interface.send_function_file_name())
    }

    pub fn create_code(&self) -> String {
        templates::SendFunctionTemplate::build(
            &self.interface,
            self.server_url.clone(),
            self.api_route_name.clone(),
        )
    }
}

#[derive(Debug, Clone)]
pub struct TypescriptInterface {
    pub name: String,
    pub fields: Vec<InterfaceField>,
}

impl TypescriptInterface {
    pub fn new(name: String, fields: Vec<InterfaceField>) -> TypescriptInterface {
        TypescriptInterface { name, fields }
    }

    pub fn file_name(&self) -> String {
        //! Use when an interface is used in a file name. Does not include the .ts extension.
        self.name.to_case(Case::Pascal)
    }

    pub fn file_name_with_extension(&self) -> String {
        //! The interface's file name with the .ts extension.
        format!("{}.ts", self.file_name())
    }

    pub fn send_function_name(&self) -> String {
        format!("send{}", self.name.to_case(Case::Pascal))
    }

    pub fn send_function_file_name(&self) -> String {
        format!("Send{}", self.file_name())
    }

    pub fn send_function_file_name_with_extension(&self) -> String {
        format!("{}.ts", self.send_function_file_name())
    }

    pub fn var_name(&self) -> String {
        //! Use when an interface is used in a function, it is passed as a variable.
        self.name.to_case(Case::Camel)
    }

    pub fn create_code(&self) -> String {
        templates::InterfaceTemplate::build(self)
    }
}

#[derive(Debug, Clone)]
pub struct InterfaceField {
    pub name: String,
    pub comment: Option<String>,
    pub is_optional: bool,
    pub field_type: InterfaceFieldType,
}

impl InterfaceField {
    pub fn new(
        name: String,
        comment: Option<String>,
        is_optional: bool,
        field_type: InterfaceFieldType,
    ) -> InterfaceField {
        InterfaceField {
            name,
            comment,
            is_optional,
            field_type,
        }
    }
}

#[derive(Debug, Clone)]
pub enum InterfaceFieldType {
    String,
    Number,
    Boolean,
    Date,
    Array(Box<InterfaceFieldType>),
    Object(Box<TypescriptInterface>),
}

impl fmt::Display for InterfaceFieldType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InterfaceFieldType::String => write!(f, "string"),
            InterfaceFieldType::Number => write!(f, "number"),
            InterfaceFieldType::Boolean => write!(f, "boolean"),
            InterfaceFieldType::Date => write!(f, "Date"),
            InterfaceFieldType::Array(inner_type) => write!(f, "{}[]", inner_type),
            InterfaceFieldType::Object(inner_type) => write!(f, "{}", inner_type.name),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypescriptObjects {
    pub interface: TypescriptInterface,
    pub send_function: SendFunction,
}

impl TypescriptObjects {
    pub fn new(interface: TypescriptInterface, send_function: SendFunction) -> Self {
        Self {
            interface,
            send_function,
        }
    }
}

pub struct TypescriptPackage {
    pub name: String,
    // version: String,
    // description: String,
    // author: String,
}

impl TypescriptPackage {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn from_project(project: &Project) -> Self {
        Self {
            name: format!("{}-sdk", project.name().clone()),
        }
    }
}

fn std_field_type_to_typescript_field_mapper(
    field_type: ColumnType,
) -> Result<InterfaceFieldType, TypescriptGeneratorError> {
    match field_type {
        ColumnType::String => Ok(InterfaceFieldType::String),
        ColumnType::Boolean => Ok(InterfaceFieldType::Boolean),
        ColumnType::Int => Ok(InterfaceFieldType::Number),
        ColumnType::Float => Ok(InterfaceFieldType::Number),
        ColumnType::Decimal => Ok(InterfaceFieldType::Number),
        ColumnType::DateTime => Ok(InterfaceFieldType::Date),
        ColumnType::Array(inner) => {
            let inner_type = std_field_type_to_typescript_field_mapper(*inner)?;
            Ok(InterfaceFieldType::Array(Box::new(inner_type)))
        }
        ColumnType::Bytes => Err(TypescriptGeneratorError::UnsupportedDataTypeError {
            type_name: "Bytes".to_string(),
        }),
        ColumnType::Enum(_) => Err(TypescriptGeneratorError::UnsupportedDataTypeError {
            type_name: "Enum".to_string(),
        }),
        ColumnType::Json => Err(TypescriptGeneratorError::UnsupportedDataTypeError {
            type_name: "Json".to_string(),
        }),
        ColumnType::BigInt => Err(TypescriptGeneratorError::UnsupportedDataTypeError {
            type_name: "BigInt".to_string(),
        }),
    }
}

pub fn std_table_to_typescript_interface(
    table: Table,
    model_name: &str,
) -> Result<TypescriptInterface, TypescriptGeneratorError> {
    let mut fields: Vec<InterfaceField> = Vec::new();

    for column in table.columns {
        let typescript_interface_type =
            std_field_type_to_typescript_field_mapper(column.data_type.clone())?;

        fields.push(InterfaceField {
            name: column.name,
            field_type: typescript_interface_type,
            is_optional: !column.required,
            comment: Some(format!(
                "db_type:{} | isPrimary:{}",
                column.data_type, column.primary_key
            )),
        });
    }

    Ok(TypescriptInterface {
        name: model_name.to_string(),
        fields,
    })
}

pub fn generate_sdk(
    project: &Project,
    ts_objects: &HashMap<String, TypescriptObjects>,
) -> Result<PathBuf, std::io::Error> {
    //! Generates a Typescript SDK for the given project and returns the path where the SDK was generated.
    //!
    //! # Arguments
    //! - `project` - The project to generate the SDK for.
    //! - `ts_objects` - The objects to generate the SDK for.
    //!
    //!
    //! # Returns
    //! - `Result<PathBuf, std::io::Error>` - A result containing the path where the SDK was generated.
    //!
    let internal_dir = project.internal_dir()?;

    let package = TypescriptPackage::from_project(project);
    let package_json_code = PackageJsonTemplate::build(&package);
    let ts_config_code = TsConfigTemplate::build();
    let index_code = IndexTemplate::build(ts_objects);

    // This needs to write to the root of the NPM folder... creating in the current project location for now
    let sdk_dir = internal_dir.join(package.name);

    std::fs::remove_dir_all(sdk_dir.clone()).or_else(|err| match err.kind() {
        std::io::ErrorKind::NotFound => Ok(()),
        _ => Err(err),
    })?;

    std::fs::create_dir_all(sdk_dir.clone())?;

    fs::write(sdk_dir.join("package.json"), package_json_code)?;
    fs::write(sdk_dir.join("tsconfig.json"), ts_config_code)?;
    fs::write(sdk_dir.join("index.ts"), index_code)?;

    for obj in ts_objects.values() {
        let interface_code = obj.interface.create_code();
        let send_function_code = obj.send_function.create_code();

        fs::write(
            sdk_dir.join(obj.interface.file_name_with_extension()),
            interface_code,
        )?;
        fs::write(
            sdk_dir.join(obj.send_function.file_name_with_extension()),
            send_function_code,
        )?;
    }

    Ok(sdk_dir)
}

pub fn generate_temp_data_model(
    project: &Project,
    fo: &FrameworkObject,
) -> Result<(), std::io::Error> {
    // TODO remove this when we move away from prisma
    if fo.original_file_path.extension() == Some(OsStr::new("prisma")) {
        let schemas_dir = project.schemas_dir();
        let ts_interface_code = fo.ts_interface.create_code();

        debug!(
            "Prisma model {:?} detected, generating typescript in the datamodels folder {:?}",
            fo.original_file_path, schemas_dir
        );

        fs::write(
            schemas_dir.join(format!(
                "{}{}",
                fo.ts_interface.file_name(),
                TS_INTERFACE_GENERATE_EXT
            )),
            ts_interface_code,
        )?;
    }

    Ok(())
}

pub fn move_to_npm_global_dir(sdk_location: &PathBuf) -> Result<PathBuf, std::io::Error> {
    //! Moves the generated SDK to the NPM global directory.
    //!
    //! *** Note *** This here doesn't work for typescript due to package resolution issues.
    //!
    //! # Arguments
    //! - `sdk_location` - The location of the generated SDK.
    //!
    //! # Returns
    //! - `Result<PathBuf, std::io::Error>` - A result containing the path where the SDK was moved to.
    //!
    let global_node_modules = package_managers::get_or_create_global_folder()?;

    system::copy_directory(sdk_location, &global_node_modules)?;

    Ok(global_node_modules)
}

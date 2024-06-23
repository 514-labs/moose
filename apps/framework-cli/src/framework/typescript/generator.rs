use convert_case::{Case, Casing};
use serde::Serialize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::{fmt, path::PathBuf};

use crate::framework::core::code_loader::{FrameworkObjectVersions, SchemaVersion};
use crate::framework::typescript;
use crate::{
    project::Project,
    utilities::{package_managers, system},
};

use crate::framework::core::infrastructure::table::{ColumnType, DataEnum, EnumValue, Table};

use super::templates::TypescriptRenderingError;

#[derive(Debug, thiserror::Error)]
#[error("Failed to generate Typescript code")]
#[non_exhaustive]
pub enum TypescriptGeneratorError {
    #[error("Typescript Code Generator - Unsupported data type: {type_name}")]
    UnsupportedDataTypeError {
        type_name: String,
    },
    FileWritingError(#[from] std::io::Error),
    RenderingError(#[from] typescript::templates::TypescriptRenderingError),
    ProjectFile(#[from] crate::project::ProjectFileError),
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

    pub fn create_code(&self) -> Result<String, TypescriptRenderingError> {
        typescript::templates::render_interface(self)
    }

    pub fn enums(&self) -> HashSet<String> {
        self.fields
            .iter()
            .filter_map(|field| {
                if let InterfaceFieldType::Enum(e) = &field.field_type {
                    Some(e.name.clone())
                } else {
                    None
                }
            })
            .collect()
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
    Enum(TSEnum),
}

#[derive(Debug, Clone, Serialize, Eq, PartialEq, Hash)]
pub struct TSEnum {
    pub name: String,
    pub values: Vec<TSEnumMember>,
}

#[derive(Debug, Clone, Serialize, Eq, PartialEq, Hash)]
pub struct TSEnumMember {
    pub name: String,
    pub value: TSEnumValue,
}

#[derive(Debug, Clone, Serialize, Eq, PartialEq, Hash)]
pub enum TSEnumValue {
    String(String),
    Number(u8),
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
            InterfaceFieldType::Enum(e) => write!(f, "{}", e.name),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypescriptObjects {
    pub interface: TypescriptInterface,
}

impl TypescriptObjects {
    pub fn new(interface: TypescriptInterface) -> Self {
        Self { interface }
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
        ColumnType::Enum(enum_type) => Ok(InterfaceFieldType::Enum(map_std_enum_to_ts(enum_type))),
        ColumnType::Json => Err(TypescriptGeneratorError::UnsupportedDataTypeError {
            type_name: "Json".to_string(),
        }),
        ColumnType::BigInt => Err(TypescriptGeneratorError::UnsupportedDataTypeError {
            type_name: "BigInt".to_string(),
        }),
        ColumnType::Nested(inner) => {
            Ok(InterfaceFieldType::Object(Box::new(TypescriptInterface {
                name: inner.name,
                fields: inner
                    .columns
                    .iter()
                    .map(|c| {
                        Ok(InterfaceField {
                            name: c.name.clone(),
                            comment: None,
                            is_optional: !c.required,
                            field_type: std_field_type_to_typescript_field_mapper(
                                c.data_type.clone(),
                            )?,
                        })
                    })
                    .collect::<Result<Vec<InterfaceField>, TypescriptGeneratorError>>()?,
            })))
        }
    }
}

fn map_std_enum_to_ts(enum_type: DataEnum) -> TSEnum {
    let mut values: Vec<TSEnumMember> = Vec::new();

    for enum_member in enum_type.values {
        let enum_value = match enum_member.value {
            EnumValue::String(value) => TSEnumValue::String(value),
            EnumValue::Int(value) => TSEnumValue::Number(value),
        };

        values.push(TSEnumMember {
            name: enum_member.name,
            value: enum_value,
        });
    }

    TSEnum {
        name: enum_type.name,
        values,
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

fn collect_ts_objects(
    framework_objects: &SchemaVersion,
) -> Result<Vec<TypescriptObjects>, TypescriptGeneratorError> {
    let mut ts_objects: Vec<TypescriptObjects> = Vec::new();

    for model in framework_objects.get_all_models() {
        let interface = std_table_to_typescript_interface(model.to_table(), &model.name)?;
        ts_objects.push(TypescriptObjects::new(interface));
    }

    Ok(ts_objects)
}

fn collect_enums(framework_objects: &SchemaVersion) -> HashSet<TSEnum> {
    let mut enums: HashSet<TSEnum> = HashSet::new();

    for enum_type in framework_objects.get_all_enums() {
        enums.insert(map_std_enum_to_ts(enum_type.clone()));
    }

    enums
}

pub fn generate_sdk(
    project: &Project,
    framework_object_versions: &FrameworkObjectVersions,
    sdk_dir: &Path,
    packaged: &bool,
) -> Result<(), TypescriptGeneratorError> {
    //! Generates a Typescript SDK for the given project and returns the path where the SDK was generated.
    //!
    //! # Arguments
    //! - `project` - The project to generate the SDK for.
    //! - `framework_object_versions` - The objects to generate the SDK for.
    //! - `sdk_dir` - Where to write the generated SDK.
    //! - `packaged` - Whether or not to generate a full fledged package or just the source files in the
    //!                language of choice.
    //!
    //! # Returns
    //! - `Result<PathBuf, std::io::Error>` - A result containing the path where the SDK was generated.

    let current_version_ts_objects = collect_ts_objects(&framework_object_versions.current_models)?;

    let enums: HashSet<TSEnum> = collect_enums(&framework_object_versions.current_models);

    std::fs::remove_dir_all(sdk_dir).or_else(|err| match err.kind() {
        std::io::ErrorKind::NotFound => Ok(()),
        _ => Err(err),
    })?;
    std::fs::create_dir_all(sdk_dir)?;

    if *packaged {
        let package = TypescriptPackage::from_project(project);
        let package_json_code = typescript::templates::render_package_json(&package.name)?;
        fs::write(sdk_dir.join("package.json"), package_json_code)?;
        let ts_config_code = typescript::templates::render_ts_config()?;
        fs::write(sdk_dir.join("tsconfig.json"), ts_config_code)?;
    }

    let index_code = typescript::templates::render_ingest_client(
        &framework_object_versions.current_version,
        &current_version_ts_objects,
    )?;
    fs::write(sdk_dir.join("index.ts"), index_code)?;

    if !enums.is_empty() {
        let current_enum_code = typescript::templates::render_enums(enums)?;
        fs::write(sdk_dir.join("enums.ts"), current_enum_code)?;
    }

    for obj in current_version_ts_objects.iter() {
        let interface_code = obj.interface.create_code()?;
        fs::write(
            sdk_dir.join(obj.interface.file_name_with_extension()),
            interface_code,
        )?;
    }

    let versions = framework_object_versions.previous_version_models.iter();

    for (version, models) in versions {
        let version_dir = sdk_dir.join(version);
        fs::create_dir_all(&version_dir)?;

        let ts_objects = collect_ts_objects(models)?;
        let version_enums = collect_enums(models);

        if !version_enums.is_empty() {
            let enums_code = typescript::templates::render_enums(version_enums)?;
            fs::write(version_dir.join("enums.ts"), enums_code)?;
        }

        let client_code = typescript::templates::render_ingest_client(version, &ts_objects)?;
        fs::write(version_dir.join("index.ts"), client_code)?;

        for obj in ts_objects.iter() {
            let interface_code = typescript::templates::render_interface(&obj.interface)?;
            fs::write(
                version_dir.join(obj.interface.file_name_with_extension()),
                interface_code,
            )?;
        }
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

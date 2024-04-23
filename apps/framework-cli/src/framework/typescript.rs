use std::{fmt, path::PathBuf};

use convert_case::{Case, Casing};

use crate::project::Project;

use super::{
    languages::{get_models_dir, CodeGenerator},
    schema::UnsupportedDataTypeError,
};

pub mod mapper;
pub mod templates;

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
}

impl CodeGenerator for TypescriptInterface {
    fn create_code(&self) -> Result<String, UnsupportedDataTypeError> {
        Ok(templates::InterfaceTemplate::build(self))
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
    Unsupported,
}

impl fmt::Display for InterfaceFieldType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InterfaceFieldType::String => write!(f, "string"),
            InterfaceFieldType::Number => write!(f, "number"),
            InterfaceFieldType::Boolean => write!(f, "boolean"),
            InterfaceFieldType::Date => write!(f, "Date"),
            InterfaceFieldType::Array(inner_type) => write!(f, "List<{}>", inner_type),
            InterfaceFieldType::Object(inner_type) => write!(f, "{}", inner_type.name),
            InterfaceFieldType::Unsupported => write!(f, "Unsupported Interface field type"),
        }
    }
}
#[derive(Debug, Clone)]
pub struct SendFunction {
    pub interface: TypescriptInterface,
    server_url: String,
    api_route_name: String,
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
}

impl CodeGenerator for SendFunction {
    fn create_code(&self) -> Result<String, UnsupportedDataTypeError> {
        Ok(templates::SendFunctionTemplate::build(
            &self.interface,
            self.server_url.clone(),
            self.api_route_name.clone(),
        ))
    }
}

pub fn create_typescript_models_dir(project: &Project) -> Result<PathBuf, std::io::Error> {
    let models_dir = get_models_dir(project);
    match models_dir {
        Ok(dir) => {
            std::fs::create_dir_all(dir.join("typescript"))?;
            Ok(dir)
        }
        Err(err) => Err(err),
    }
}

pub fn get_typescript_models_dir(project: &Project) -> Result<PathBuf, std::io::Error> {
    let models_dir = get_models_dir(project)?;
    let typescript_dir = models_dir.clone().join("typescript");
    if typescript_dir.exists() {
        Ok(typescript_dir)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Typescript models directory not found",
        ))
    }
}

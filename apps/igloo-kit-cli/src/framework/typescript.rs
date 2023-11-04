use std::{fmt, path::PathBuf};

use super::{languages::{CodeGenerator, get_models_dir}, schema::UnsupportedDataTypeError};

mod templates;
pub mod mapper;


#[derive(Debug, Clone)]
pub struct TypescriptInterface {
    pub name: String,
    pub fields: Vec<InterfaceField>,
}

impl TypescriptInterface {
    pub fn new(name: String, fields: Vec<InterfaceField>) -> TypescriptInterface {
        TypescriptInterface {
            name,
            fields,
        }
    }
}

impl CodeGenerator for TypescriptInterface {
    fn create_code(&self) -> Result<String, UnsupportedDataTypeError> {
        Ok(templates::InterfaceTemplate::new(self.clone()))
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
    pub fn new(name: String, comment: Option<String>, is_optional: bool, field_type: InterfaceFieldType) -> InterfaceField {
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
            InterfaceFieldType::Unsupported => write!(f, "Unsupported"),
        }
    }
}


pub fn create_typescript_models_dir() -> Result<PathBuf, std::io::Error> {
    let models_dir = get_models_dir();
    match models_dir {
        Ok(dir) => {
            std::fs::create_dir_all(dir.join("typescript"))?;
            Ok(dir)
        },
        Err(err) => {
            Err(err)
        }
    }
}

pub fn get_typescript_models_dir() -> Result<PathBuf, std::io::Error> {
    let models_dir = get_models_dir()?;
    let typescript_dir = models_dir.clone().join("typescript");
    if typescript_dir.exists() {
        Ok(typescript_dir)
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Typescript models directory not found"))
    }
}
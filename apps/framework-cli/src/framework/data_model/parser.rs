use serde::Deserialize;
use std::path::Path;

use super::model::DataModel;
use crate::utilities::constants;
use crate::{
    framework::{
        core::{code_loader::MappingError, infrastructure::table::DataEnum},
        prisma, python, typescript,
    },
    project::Project,
};

#[derive(Debug, thiserror::Error)]
#[error("Failed to parse the data model file")]
#[non_exhaustive]
pub enum DataModelParsingError {
    PrismaParsingError(#[from] prisma::parser::PrismaParsingError),
    TypescriptParsingError(#[from] typescript::parser::TypescriptParsingError),
    PythonParsingError(#[from] python::parser::PythonParserError),
    MappingError(#[from] MappingError),
    #[error("Unsupported file: {file_name}")]
    UnsupportedFileType {
        file_name: String,
    },
}

#[derive(Deserialize, Debug, Default)]
pub struct FileObjects {
    pub models: Vec<DataModel>,
    pub enums: Vec<DataEnum>,
}

impl FileObjects {
    pub fn new(models: Vec<DataModel>, enums: Vec<DataEnum>) -> FileObjects {
        FileObjects { models, enums }
    }
}

pub fn parse_data_model_file(
    path: &Path,
    version: &str,
    project: &Project,
) -> Result<FileObjects, DataModelParsingError> {
    // TODO - Remove this if when we have deprecated v0.3 of the internal deployment
    if !path
        .file_name()
        .is_some_and(|file_name| file_name.to_str().unwrap().contains("generated"))
    {
        if let Some(ext) = path.extension() {
            match ext.to_str() {
                Some("prisma") => Ok(prisma::parser::extract_data_model_from_file(path, version)?),
                Some(constants::TYPESCRIPT_FILE_EXTENSION) => Ok(
                    typescript::parser::extract_data_model_from_file(path, project, version)?,
                ),
                Some(constants::PYTHON_FILE_EXTENSION) => {
                    Ok(python::parser::extract_data_model_from_file(path, version)?)
                }
                Some(constants::PYTHON_CACHE_EXTENSION) => Ok(FileObjects::default()), // __pycache__
                _ => unsupported_file_type(path),
            }
        } else {
            unsupported_file_type(path)
        }
    } else {
        Ok(FileObjects::default())
    }
}

fn unsupported_file_type<T>(path: &Path) -> Result<T, DataModelParsingError> {
    Err(DataModelParsingError::UnsupportedFileType {
        file_name: match path.file_name() {
            None => "".to_string(),
            Some(f) => f.to_string_lossy().to_string(),
        },
    })
}

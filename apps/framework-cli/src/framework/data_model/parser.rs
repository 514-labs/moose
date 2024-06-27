use serde::Deserialize;
use std::path::Path;

use crate::{
    framework::{
        core::{code_loader::MappingError, infrastructure::table::DataEnum},
        prisma, python, typescript,
    },
    project::Project,
};

use super::schema::DataModel;

#[derive(Debug, thiserror::Error)]
#[error("Failed to parse the data model file")]
#[non_exhaustive]
pub enum DataModelParsingError {
    PrismaParsingError(#[from] prisma::parser::PrismaParsingError),
    TypescriptParsingError(#[from] typescript::parser::TypescriptParsingError),
    PythonParsingError(#[from] python::parser::PythonParserError),
    MappingError(#[from] MappingError),
    UnsupportedFileType,
}

#[derive(Deserialize, Debug)]
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
    if let Some(ext) = path.extension() {
        match ext.to_str() {
            Some("prisma") => Ok(prisma::parser::extract_data_model_from_file(path, version)?),
            Some("ts") => Ok(typescript::parser::extract_data_model_from_file(
                path, project, version,
            )?),
            Some("py") => Ok(python::parser::extract_data_model_from_file(path, version)?),
            _ => Err(DataModelParsingError::UnsupportedFileType),
        }
    } else {
        Err(DataModelParsingError::UnsupportedFileType)
    }
}

use log::debug;
use std::path::Path;

use crate::framework::{controller::MappingError, prisma, typescript};

use super::schema::{DataEnum, DataModel};

#[derive(Debug, thiserror::Error)]
#[error("Failed to parse the data model file")]
#[non_exhaustive]
pub enum DataModelParsingError {
    PrismaParsingError(#[from] prisma::parser::PrismaParsingError),
    TypescriptParsingError(#[from] typescript::parser::TypescriptParsingError),
    MappingError(#[from] MappingError),
}

pub struct FileObjects {
    pub models: Vec<DataModel>,
    pub enums: Vec<DataEnum>,
}

impl FileObjects {
    pub fn new(models: Vec<DataModel>, enums: Vec<DataEnum>) -> FileObjects {
        FileObjects { models, enums }
    }
}

pub fn parse_data_model_file(path: &Path) -> Result<FileObjects, DataModelParsingError> {
    let is_prisma_schema = path.extension().map(|e| e == "prisma").unwrap_or(false);

    if is_prisma_schema {
        debug!("Parsing prisma file {:?}", path);
        Ok(prisma::parser::extract_data_model_from_file(path)?)
    } else {
        debug!("Parsing typescript file {:?}", path);
        Ok(typescript::parser::extract_data_model_from_file(path)?)
    }
}

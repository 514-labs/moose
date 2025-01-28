use crate::framework::core::primitive_map::PrimitiveMap;
use crate::{
    framework::{
        languages::SupportedLanguages,
        typescript::{self, generator::TypescriptGeneratorError},
    },
    project::Project,
};
use std::path::Path;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SDKGenerationError {
    #[error("Failed to generate Typescript SDK")]
    TypescriptError(#[from] TypescriptGeneratorError),
}

pub fn generate_sdk(
    language: &SupportedLanguages,
    project: &Project,
    primitive_map: &PrimitiveMap,
    destination: &Path,
    packaged: &bool,
) -> Result<(), SDKGenerationError> {
    match language {
        SupportedLanguages::Typescript => Ok(typescript::generator::generate_sdk(
            project,
            primitive_map,
            destination,
            packaged,
        )?),
        SupportedLanguages::Python => {
            todo!("Python SDK generation is not yet supported");
        }
    }
}

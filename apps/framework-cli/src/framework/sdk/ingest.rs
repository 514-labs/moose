use std::path::Path;

use crate::{
    framework::{
        core::code_loader::FrameworkObjectVersions,
        languages::SupportedLanguages,
        typescript::{self, generator::TypescriptGeneratorError},
    },
    project::Project,
};

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SDKGenerationError {
    #[error("Failed to generate Typescript SDK")]
    TypescriptError(#[from] TypescriptGeneratorError),
}

pub fn generate_sdk(
    language: &SupportedLanguages,
    project: &Project,
    framework_objects: &FrameworkObjectVersions,
    destination: &Path,
    packaged: &bool,
) -> Result<(), SDKGenerationError> {
    match language {
        SupportedLanguages::Typescript => Ok(typescript::generator::generate_sdk(
            project,
            framework_objects,
            destination,
            packaged,
        )?),
        SupportedLanguages::Python => {
            todo!("Python SDK generation is not yet supported");
        }
    }
}

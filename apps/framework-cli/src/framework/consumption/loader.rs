use super::model::{Consumption, ConsumptionQueryParam, EndpointFile};
use crate::framework::languages::SupportedLanguages;
use crate::framework::python::consumption::load_python_query_param;
use crate::framework::typescript;
use crate::framework::typescript::export_collectors::ExportCollectorError;
use crate::project::Project;
use crate::utilities::PathExt;
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ConsumptionLoaderError {
    #[error("Failed to open file: {0}")]
    FailedToOpenFile(std::io::Error),
    #[error("Failed to load query params: {0}")]
    FailedToLoadPythonParams(std::io::Error),
    #[error("Failed to load query params: {0}")]
    FailedToLoadTypescriptParams(ExportCollectorError),
}

#[derive(Debug, Deserialize)]
pub struct QueryParamOutput {
    pub params: Vec<ConsumptionQueryParam>,
}

pub async fn load_consumption(project: &Project) -> Result<Consumption, ConsumptionLoaderError> {
    let mut endpoint_files = Vec::new();
    for f in walkdir::WalkDir::new(project.consumption_dir())
        .into_iter()
        // drop Err cases
        .flatten()
    {
        if f.file_type().is_file() && f.path().ext_is_supported_lang() {
            let result = build_endpoint_file(project, f.path()).await;
            log::debug!("build_endpoint_file result: {:?}", result);
            if let Some(file) = result.ok().flatten() {
                endpoint_files.push(file);
            }
        }
    }

    Ok(Consumption { endpoint_files })
}

async fn build_endpoint_file(
    project: &Project,
    file_path: &Path,
) -> Result<Option<EndpointFile>, ConsumptionLoaderError> {
    if let Ok(path) = file_path.strip_prefix(project.consumption_dir()) {
        let mut file =
            fs::File::open(file_path).map_err(ConsumptionLoaderError::FailedToOpenFile)?;
        let mut hasher = Sha256::new();
        std::io::copy(&mut file, &mut hasher).map_err(ConsumptionLoaderError::FailedToOpenFile)?;
        let hash = hasher.finalize();

        let mut path = path.to_path_buf();
        path.set_extension("");

        let (query_params, output_schema) = match project.language {
            SupportedLanguages::Typescript => {
                typescript::export_collectors::get_func_types(&path, &project.project_location)
                    .await
                    .map_err(ConsumptionLoaderError::FailedToLoadTypescriptParams)?
            }
            SupportedLanguages::Python => {
                let params = load_python_query_param(&project.project_location, &path)
                    .await
                    .map_err(ConsumptionLoaderError::FailedToLoadPythonParams)?;
                (params, Value::Null)
            }
        };

        Ok(Some(EndpointFile {
            path,
            hash,
            query_params,
            output_schema,
        }))
    } else {
        Ok(None)
    }
}

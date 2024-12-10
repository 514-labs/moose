use super::model::{Consumption, ConsumptionQueryParam, EndpointFile};
use crate::framework::languages::SupportedLanguages;
use crate::framework::python::executor::{run_python_program, PythonProgram};
use crate::project::Project;
use crate::utilities::PathExt;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ConsumptionLoaderError {
    #[error("Failed to open file: {0}")]
    FailedToOpenFile(std::io::Error),
    #[error("Failed to load query params: {0}")]
    FailedToLoadParams(std::io::Error),
}

#[derive(Debug, Deserialize)]
struct QueryParamOutput {
    params: Vec<ConsumptionQueryParam>,
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

async fn load_python_query_param(
    path: &Path,
) -> Result<Vec<ConsumptionQueryParam>, std::io::Error> {
    let args = vec![path.file_name().unwrap().to_str().unwrap().to_string()];
    let process = run_python_program(PythonProgram::LoadApiParam { args })?;
    let output = process.wait_with_output().await?;

    if !output.status.success() {
        return Err(std::io::Error::other(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }
    let raw_string_stdout = String::from_utf8_lossy(&output.stdout);

    let config = serde_json::from_str::<QueryParamOutput>(&raw_string_stdout)
        .map_err(std::io::Error::other)?;
    Ok(config.params)
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

        let query_params = match project.language {
            SupportedLanguages::Typescript => Vec::new(),
            SupportedLanguages::Python => load_python_query_param(&path)
                .await
                .map_err(ConsumptionLoaderError::FailedToLoadParams)?,
        };

        Ok(Some(EndpointFile {
            path,
            hash,
            query_params,
        }))
    } else {
        Ok(None)
    }
}

use crate::project::Project;
use sha2::{Digest, Sha256};
use std::{fs, io, path::Path};

use super::model::{Consumption, EndpointFile};
use crate::utilities::PathExt;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ConsumptionLoaderError {
    #[error("Failed to open file: {0}")]
    FailedToOpenFile(#[from] std::io::Error),
}

pub fn load_consumption(project: &Project) -> Result<Consumption, ConsumptionLoaderError> {
    let endpoint_files = walkdir::WalkDir::new(project.consumption_dir())
        .into_iter()
        .filter_map::<EndpointFile, _>(|f| {
            if let Ok(f) = f {
                if f.file_type().is_file() && f.path().ext_is_supported_lang() {
                    build_endpoint_file(project, f.path()).ok().flatten()
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    Ok(Consumption { endpoint_files })
}

fn build_endpoint_file(
    project: &Project,
    file_path: &Path,
) -> Result<Option<EndpointFile>, ConsumptionLoaderError> {
    if let Ok(path) = file_path.strip_prefix(project.consumption_dir()) {
        let mut file = fs::File::open(file_path)?;
        let mut hasher = Sha256::new();
        io::copy(&mut file, &mut hasher)?;
        let hash = hasher.finalize();

        let mut path = path.to_path_buf();
        path.set_extension("");

        Ok(Some(EndpointFile { path, hash }))
    } else {
        Ok(None)
    }
}

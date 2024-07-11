pub mod config;
pub mod model;
pub mod parser;

use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::{
    fmt,
    path::{Path, PathBuf},
};

use crate::utilities::system::file_name_contains;

use super::core::code_loader::FrameworkObject;

#[derive(Debug, Clone)]
pub struct DuplicateModelError {
    pub model_name: String,
    pub file_path: PathBuf,
    pub other_file_path: PathBuf,
}

impl DuplicateModelError {
    pub fn try_insert(
        map: &mut HashMap<String, FrameworkObject>,
        fo: FrameworkObject,
        current_path: &Path,
    ) -> Result<(), Self> {
        let maybe_existing = map.insert(fo.data_model.name.clone(), fo);
        match maybe_existing {
            None => Ok(()),
            Some(other_fo) => Err(DuplicateModelError {
                model_name: other_fo.data_model.name,
                file_path: current_path.to_path_buf(),
                other_file_path: other_fo.original_file_path,
            }),
        }
    }
}

impl Display for DuplicateModelError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Duplicate model {} in files: {}, {}",
            self.model_name,
            self.file_path.display(),
            self.other_file_path.display()
        )
    }
}

impl std::error::Error for DuplicateModelError {}

pub fn is_schema_file(path: &Path) -> bool {
    path.extension()
        .map(|extension| extension == "prisma" || extension == "ts" || extension == "py")
        .unwrap_or(false)
        // TODO: There's logic that looks at version history which may have
        // .generated.ts files. Those files need to be ignored. We don't have
        // .generated.ts files anymore, so we can remove this when we can deprecate older versions
        && !file_name_contains(path, ".generated.ts")
}

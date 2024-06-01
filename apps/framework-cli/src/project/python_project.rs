use std::collections::HashMap;
use std::path::PathBuf;

use config::ConfigError;
use serde::{Deserialize, Serialize};

use crate::{
    framework::python::{
        executor::{serialize_contents, PythonSerializers},
        parser::{get_project_from_file, PythonParserError},
        templates::{render_setup_py, PythonRenderingError},
    },
    utilities::constants::SETUP_PY,
};

#[derive(Debug, thiserror::Error)]
#[error("Failed to create or delete project files")]
#[non_exhaustive]
pub enum PythonProjectError {
    IO(#[from] std::io::Error),
    JSONSerde(#[from] serde_json::Error),
    PythonRenderingError(#[from] PythonRenderingError),
    PythonParserError(#[from] PythonParserError),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PythonProject {
    pub name: String,
    pub version: String,

    pub dependencies: Vec<String>,
}

impl Default for PythonProject {
    fn default() -> Self {
        Self {
            name: "new_project".to_string(),
            version: "0.0".to_string(),
            dependencies: vec!["kafka-python==2.0.2".to_string()],
        }
    }
}

impl PythonProject {
    pub fn new(name: String) -> Self {
        PythonProject {
            name,
            ..Default::default()
        }
    }

    pub fn load(directory: PathBuf) -> Result<Self, ConfigError> {
        let mut location = directory.clone();
        location.push(SETUP_PY);

        get_project_from_file(&location)
            .map_err(|_| ConfigError::Message("Failed to load Python project".to_string()))
    }

    pub fn write_to_disk(&self, project_location: PathBuf) -> Result<(), PythonProjectError> {
        let mut setup_py_location = project_location.clone();
        setup_py_location.push("setup.py");

        let setup_py = render_setup_py(self.clone())?;
        std::fs::write(setup_py_location, setup_py)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_project_abs_dir_path() -> PathBuf {
        let test_project_location = PathBuf::from("tests/python/project");
        let abs_test_project_location = std::fs::canonicalize(test_project_location).unwrap();
        abs_test_project_location
    }

    #[test]
    fn test_python_load() {
        let test_project_dir = get_test_project_abs_dir_path();
        println!("Test Project Dir: {:?}", test_project_dir);
        let project = PythonProject::load(test_project_dir).unwrap();

        assert_eq!(project.name, "test_project");
        assert_eq!(project.version, "0.0");
        assert_eq!(
            project.dependencies,
            vec!["kafka-python==2.0.2".to_string()]
        );
    }
}

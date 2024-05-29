use std::collections::HashMap;
use std::path::PathBuf;

use config::ConfigError;
use serde::{Deserialize, Serialize};

use crate::utilities::constants::REQUIREMENTS_TXT;

#[derive(Debug, thiserror::Error)]
#[error("Failed to create or delete project files")]
#[non_exhaustive]
pub enum PythonProjectError {
    IO(#[from] std::io::Error),
    JSONSerde(#[from] serde_json::Error),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PythonProject {
    pub name: String,
    pub version: String,

    pub dependencies: HashMap<String, String>,
}

impl Default for PythonProject {
    fn default() -> Self {
        Self {
            name: "new_project".to_string(),
            version: "0.0".to_string(),
            dependencies: HashMap::from([("kafka-python".to_string(), "^2.0.2".to_string())]),
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
        unimplemented!("Write to disk for PythonProject")
    }

    pub fn write_to_disk(&self, project_location: PathBuf) -> Result<(), PythonProjectError> {
        unimplemented!("Write to disk for PythonProject")
    }
}

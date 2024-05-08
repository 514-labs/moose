use std::collections::HashMap;
use std::path::PathBuf;

use config::{Config, ConfigError, File};
use serde::{Deserialize, Serialize};

use crate::utilities::constants::PACKAGE_JSON;

#[derive(Debug, thiserror::Error)]
#[error("Failed to create or delete project files")]
#[non_exhaustive]
pub enum TSProjectFileError {
    IO(#[from] std::io::Error),
    JSONSerde(#[from] serde_json::Error),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TypescriptProject {
    pub name: String,
    pub version: String,
    pub scripts: HashMap<String, String>,
    pub dependencies: HashMap<String, String>,
    pub dev_dependencies: HashMap<String, String>,
}

impl Default for TypescriptProject {
    fn default() -> Self {
        Self {
            name: "new_project".to_string(),
            version: "0.0".to_string(),
            // For local development of the CLI,
            // change `moose-cli` to `<REPO_PATH>/apps/framework-cli/target/debug/moose-cli`
            scripts: HashMap::from([
                ("dev".to_string(), "moose-cli dev".to_string()),
                ("moose".to_string(), "moose-cli".to_string()),
                ("build".to_string(), "moose-cli build --docker".to_string()),
            ]),
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::from([
                ("@514labs/moose-cli".to_string(), "latest".to_string()),
                ("typescript".to_string(), "^5.4.0".to_string()),
                ("ts-node".to_string(), "^10.9.2".to_string()),
            ]),
        }
    }
}

impl TypescriptProject {
    pub fn new(name: String) -> Self {
        TypescriptProject {
            name,
            ..Default::default()
        }
    }

    pub fn load(directory: PathBuf) -> Result<Self, ConfigError> {
        let mut package_json_location = directory.clone();
        package_json_location.push(PACKAGE_JSON);

        Config::builder()
            .add_source(File::from(package_json_location).required(true))
            .build()?
            .try_deserialize()
    }

    pub fn write_to_disk(&self, project_location: PathBuf) -> Result<(), TSProjectFileError> {
        let mut package_json_location = project_location.clone();
        package_json_location.push(PACKAGE_JSON);

        let json = serde_json::to_string_pretty(&self)?;
        std::fs::write(&package_json_location, json)?;

        Ok(())
    }
}

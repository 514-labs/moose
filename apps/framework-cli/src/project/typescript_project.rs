use std::collections::HashMap;
use std::path::PathBuf;

use config::{Config, ConfigError, File};
use serde::{Deserialize, Serialize};

use crate::utilities::constants::PACKAGE_JSON;

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
            scripts: HashMap::from([("dev".to_string(), "igloo-cli dev".to_string())]),
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::from([(
                "@514labs/moose-cli".to_string(),
                "latest".to_string(),
            )]),
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

    pub fn write_to_disk(&self, project_location: PathBuf) -> Result<(), anyhow::Error> {
        let mut package_json_location = project_location.clone();
        package_json_location.push(PACKAGE_JSON);

        let json = serde_json::to_string_pretty(&self)?;
        std::fs::write(&package_json_location, json)?;

        Ok(())
    }
}

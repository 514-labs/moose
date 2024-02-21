use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

use config::{Config, ConfigError, File};
use serde::{Deserialize, Serialize};

use crate::cli::local_webserver::LocalWebserverConfig;
use crate::infrastructure::console::ConsoleConfig;
use crate::infrastructure::olap::clickhouse::config::ClickhouseConfig;
use crate::infrastructure::stream::redpanda::RedpandaConfig;
use crate::project::project_config_file::ProjectConfigFile;
use crate::utilities::constants::PACKAGE_JSON;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct PackageJsonFile {
    pub name: String,
    pub version: String,
    pub scripts: HashMap<String, String>,
    pub dependencies: HashMap<String, String>,
    pub dev_dependencies: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypescriptProject {
    pub name: String,
    pub project_location: PathBuf,
    pub redpanda_config: RedpandaConfig,
    pub clickhouse_config: ClickhouseConfig,
    pub http_server_config: LocalWebserverConfig,
    pub console_config: ConsoleConfig,
}

impl TypescriptProject {
    pub fn new(dir_location: &Path, name: String) -> Self {
        Self {
            name,
            project_location: dir_location.to_path_buf(),
            redpanda_config: RedpandaConfig::default(), // TODO: Add the ability for the developer to configure this
            clickhouse_config: ClickhouseConfig::default(), // TODO: Add the ability for the developer to configure this
            http_server_config: LocalWebserverConfig::default(), // TODO: Add the ability for the developer to configure this
            console_config: ConsoleConfig::default(), // TODO: Add the ability for the developer to configure this
        }
    }

    pub fn load(
        project_config: ProjectConfigFile,
        directory: PathBuf,
    ) -> Result<Self, ConfigError> {
        let mut package_json_location = directory.clone();
        package_json_location.push(PACKAGE_JSON);

        let package_json: PackageJsonFile = Config::builder()
            .add_source(File::from(package_json_location).required(true))
            .build()?
            .try_deserialize()?;

        return Ok(TypescriptProject {
            name: package_json.name,
            project_location: directory,
            redpanda_config: project_config.redpanda,
            clickhouse_config: project_config.clickhouse,
            http_server_config: project_config.http_server,
            console_config: project_config.console,
        });
    }

    pub fn write_to_disk(&self) -> Result<(), anyhow::Error> {
        let package_json = PackageJsonFile {
            name: self.name.clone(),
            version: "0.0".to_string(),
            // For local development of the CLI
            // change `igloo-cli` to `<REPO_PATH>/apps/igloo-kit-cli/target/debug/igloo-cli`
            scripts: HashMap::from([("dev".to_string(), "igloo-cli dev".to_string())]),
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::from([(
                "@514labs/moose-cli".to_string(),
                "latest".to_string(),
            )]),
        };

        let mut package_json_location = self.project_location.clone();
        package_json_location.push(PACKAGE_JSON);

        let json = serde_json::to_string_pretty(&package_json)?;
        std::fs::write(&package_json_location, json)?;

        Ok(())
    }
}

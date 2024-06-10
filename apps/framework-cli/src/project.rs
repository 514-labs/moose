//! # Project
//!
//! This module contains the `Project` struct, which represents a users project.
//! These projects are data-intensive applications or services.
//! A project is initialized using the `moose init` command and is stored
//!  in the `$PROJECT_PATH/.moose` directory.
//!
//! The `Project` struct contains the following fields:
//! - `name` - The name of the project
//! - `language` - The language of the project
//! - `project_file_location` - The location of the project file on disk
//! ```

use std::collections::HashMap;
use std::collections::HashSet;
use std::io::Write;
pub mod python_project;
pub mod typescript_project;

use std::fmt::Debug;
use std::path::Path;
use std::path::PathBuf;

use config::{Config, ConfigError, Environment, File};
use log::debug;
use python_project::PythonProject;
use serde::Deserialize;
use serde::Serialize;

use crate::cli::local_webserver::LocalWebserverConfig;
use crate::framework::languages::SupportedLanguages;
use crate::framework::python::templates::PTYHON_BASE_AGG_SAMPLE_TEMPLATE;
use crate::framework::python::templates::PYTHON_BASE_FLOW_TEMPLATE;
use crate::framework::python::templates::PYTHON_BASE_MODEL_TEMPLATE;
use crate::framework::typescript::templates::BASE_APIS_SAMPLE_TEMPLATE;
use crate::framework::typescript::templates::TS_BASE_MODEL_TEMPLATE;
use crate::framework::typescript::templates::{
    TS_BASE_AGGREGATION_SAMPLE_TEMPLATE, TS_BASE_FLOW_SAMPLE_TEMPLATE,
};
use crate::framework::typescript::templates::{
    VSCODE_EXTENSIONS_TEMPLATE, VSCODE_SETTINGS_TEMPLATE,
};
use crate::infrastructure::console::ConsoleConfig;
use crate::infrastructure::olap::clickhouse::config::ClickHouseConfig;
use crate::infrastructure::olap::clickhouse::version_sync::{parse_version, version_to_string};
use crate::infrastructure::stream::redpanda::RedpandaConfig;
use crate::project::typescript_project::TypescriptProject;

use crate::utilities::constants::API_FILE;
use crate::utilities::constants::CLI_DEV_CLICKHOUSE_VOLUME_DIR_CONFIG_SCRIPTS;
use crate::utilities::constants::CLI_DEV_CLICKHOUSE_VOLUME_DIR_CONFIG_USERS;
use crate::utilities::constants::CLI_DEV_CLICKHOUSE_VOLUME_DIR_DATA;
use crate::utilities::constants::CLI_DEV_CLICKHOUSE_VOLUME_DIR_LOGS;
use crate::utilities::constants::CLI_DEV_REDPANDA_VOLUME_DIR;
use crate::utilities::constants::CLI_INTERNAL_VERSIONS_DIR;
use crate::utilities::constants::PY_AGGREGATIONS_FILE;
use crate::utilities::constants::PY_FLOW_FILE;
use crate::utilities::constants::README_PREFIX;
use crate::utilities::constants::TS_AGGREGATIONS_FILE;
use crate::utilities::constants::{
    AGGREGATIONS_DIR, CONSUMPTION_DIR, FLOWS_DIR, PROJECT_CONFIG_FILE, SAMPLE_FLOWS_DEST,
    SAMPLE_FLOWS_SOURCE, TS_FLOW_FILE,
};
use crate::utilities::constants::{APP_DIR, APP_DIR_LAYOUT, CLI_PROJECT_INTERNAL_DIR, SCHEMAS_DIR};
use crate::utilities::constants::{VSCODE_DIR, VSCODE_EXT_FILE, VSCODE_SETTINGS_FILE};

#[derive(Debug, thiserror::Error)]
#[error("Failed to create or delete project files")]
#[non_exhaustive]
pub enum ProjectFileError {
    InternalDirCreationFailed(std::io::Error),
    #[error("Failed to create project files: {message}")]
    Other {
        message: String,
    },
    IO(#[from] std::io::Error),
    TSProjectFileError(#[from] typescript_project::TSProjectFileError),
    PythonProjectError(#[from] python_project::PythonProjectError),
    JSONSerde(#[from] serde_json::Error),
    TOMLSerde(#[from] toml::ser::Error),
}

// We have explored using a Generic associated Types as well as
// Dynamic Dispatch to handle the different types of projects
// the approach with enums is the one that is the simplest to put into practice and
// maintain. With Copilot - it also has the advaantage that the boiler plate is really fast to write
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub language: SupportedLanguages,
    pub redpanda_config: RedpandaConfig,
    pub clickhouse_config: ClickHouseConfig,
    pub http_server_config: LocalWebserverConfig,
    pub console_config: ConsoleConfig,

    // This part of the configuration for the project is dynamic and not saved
    // to disk. It is loaded from the language specific configuration file or the currently
    // running command
    #[serde(skip)]
    pub language_project_config: LanguageProjectConfig,
    #[serde(skip)]
    pub project_location: PathBuf,
    #[serde(skip, default = "Project::default_production")]
    pub is_production: bool,

    #[serde(default = "HashMap::new")]
    pub supported_old_versions: HashMap<String, String>,
}

pub struct AggregationSet {
    pub current_version: String,
    pub names: HashSet<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum LanguageProjectConfig {
    Typescript(TypescriptProject),
    Python(PythonProject),
}

impl Default for LanguageProjectConfig {
    fn default() -> Self {
        LanguageProjectConfig::Typescript(TypescriptProject::default())
    }
}

impl Project {
    pub fn default_production() -> bool {
        false
    }

    pub fn name(&self) -> String {
        match &self.language_project_config {
            LanguageProjectConfig::Typescript(p) => p.name.clone(),
            LanguageProjectConfig::Python(p) => p.name.clone(),
        }
    }

    pub fn new(dir_location: &Path, name: String, language: SupportedLanguages) -> Project {
        let mut location = dir_location.to_path_buf();

        location = location
            .canonicalize()
            .expect("The directory provided does not exist");

        debug!("Package.json file location: {:?}", location);

        match language {
            SupportedLanguages::Typescript => Project {
                language: SupportedLanguages::Typescript,
                is_production: false,
                project_location: location.clone(),
                redpanda_config: RedpandaConfig::default(),
                clickhouse_config: ClickHouseConfig::default(),
                http_server_config: LocalWebserverConfig::default(),
                console_config: ConsoleConfig::default(),
                language_project_config: LanguageProjectConfig::Typescript(TypescriptProject::new(
                    name,
                )),
                supported_old_versions: HashMap::new(),
            },
            SupportedLanguages::Python => Project {
                language: SupportedLanguages::Python,
                is_production: false,
                project_location: location.clone(),
                redpanda_config: RedpandaConfig::default(),
                clickhouse_config: ClickHouseConfig::default(),
                http_server_config: LocalWebserverConfig::default(),
                console_config: ConsoleConfig::default(),
                language_project_config: LanguageProjectConfig::Python(PythonProject::new(name)),
                supported_old_versions: HashMap::new(),
            },
        }
    }

    pub fn set_is_production_env(&mut self, is_production: bool) {
        self.is_production = is_production;
    }

    pub fn load(directory: &PathBuf) -> Result<Project, ConfigError> {
        let mut project_file = directory.clone();
        project_file.push(PROJECT_CONFIG_FILE);

        let mut project_config: Project = Config::builder()
            // TODO: consider putting the defaults into a source (e.g. include_str a toml file)
            .set_default(
                "clickhouse_config.native_port",
                ClickHouseConfig::default().native_port,
            )
            .unwrap() // key in set_default is a static string, this never fails
            .add_source(File::from(project_file).required(true))
            .add_source(
                Environment::with_prefix("MOOSE")
                    .prefix_separator("_")
                    .separator("__"),
            )
            .build()?
            .try_deserialize()?;

        project_config.project_location.clone_from(directory);

        match project_config.language {
            SupportedLanguages::Typescript => {
                let ts_config = TypescriptProject::load(directory)?;
                project_config.language_project_config =
                    LanguageProjectConfig::Typescript(ts_config);
            }
            SupportedLanguages::Python => {
                let py_config = PythonProject::load(directory)?;
                project_config.language_project_config = LanguageProjectConfig::Python(py_config);
            }
        }

        Ok(project_config)
    }

    pub fn load_from_current_dir() -> Result<Project, ConfigError> {
        let current_dir = std::env::current_dir().expect("Failed to get the current directory");
        Project::load(&current_dir)
    }

    pub fn write_to_disk(&self) -> Result<(), ProjectFileError> {
        // Write to disk what is common to all project types, the project.toml
        let project_file = self.project_location.join(PROJECT_CONFIG_FILE);
        let toml_project = toml::to_string(&self)?;

        std::fs::write(project_file, toml_project)?;

        // Write language specific files to disk
        match &self.language_project_config {
            LanguageProjectConfig::Typescript(p) => Ok(p.write_to_disk(&self.project_location)?),
            LanguageProjectConfig::Python(p) => Ok(p.write_to_disk(&self.project_location)?),
        }
    }

    pub fn setup_app_dir(&self) -> Result<(), ProjectFileError> {
        let app_dir = self.app_dir();
        std::fs::create_dir_all(&app_dir)?;

        for dir in APP_DIR_LAYOUT.iter() {
            let to_create = app_dir.join(dir);
            std::fs::create_dir_all(to_create)?;
        }

        Ok(())
    }

    pub fn create_base_app_files(&self) -> Result<(), std::io::Error> {
        let readme_file_path = self.project_location.join("README.md");
        let base_model_file_path = match self.language {
            SupportedLanguages::Typescript => self.schemas_dir().join("models.ts"),
            SupportedLanguages::Python => self.schemas_dir().join("models.py"),
        };

        let flow_file_name = match self.language {
            SupportedLanguages::Typescript => TS_FLOW_FILE,
            SupportedLanguages::Python => PY_FLOW_FILE,
        };

        let agg_file_name: &str = match self.language {
            SupportedLanguages::Typescript => TS_AGGREGATIONS_FILE,
            SupportedLanguages::Python => PY_AGGREGATIONS_FILE,
        };

        let flow_file_dir = self
            .flows_dir()
            .join(SAMPLE_FLOWS_SOURCE)
            .join(SAMPLE_FLOWS_DEST);

        let flow_file_path = flow_file_dir.join(flow_file_name);
        let aggregations_file_path = self.aggregations_dir().join(agg_file_name);
        let apis_file_path = self.consumption_dir().join(API_FILE);

        let mut readme_file = std::fs::File::create(readme_file_path)?;
        let mut base_model_file = std::fs::File::create(base_model_file_path)?;
        let mut flow_file = std::fs::File::create(flow_file_path)?;
        let mut aggregations_file = std::fs::File::create(aggregations_file_path)?;
        let mut apis_file = std::fs::File::create(apis_file_path)?;

        let mut readme = include_str!("../../../README.md").to_string();
        readme.insert_str(0, README_PREFIX);

        readme_file.write_all(readme.as_bytes())?;

        match self.language {
            SupportedLanguages::Typescript => {
                base_model_file.write_all(TS_BASE_MODEL_TEMPLATE.as_bytes())?;

                // Unclear whether the replace is still needed here?!
                flow_file.write_all(
                    TS_BASE_FLOW_SAMPLE_TEMPLATE
                        .to_string()
                        .replace("{{project_name}}", &self.name())
                        .as_bytes(),
                )?;

                aggregations_file.write_all(TS_BASE_AGGREGATION_SAMPLE_TEMPLATE.as_bytes())?;
            }
            SupportedLanguages::Python => {
                base_model_file.write_all(PYTHON_BASE_MODEL_TEMPLATE.as_bytes())?;

                // Create __init__.py file in the app directory tree
                std::fs::File::create(self.app_dir().join("__init__.py"))?;

                std::fs::File::create(self.schemas_dir().join("__init__.py"))?;
                std::fs::File::create(self.aggregations_dir().join("__init__.py"))?;
                std::fs::File::create(self.consumption_dir().join("__init__.py"))?;
                std::fs::File::create(self.flows_dir().join("__init__.py"))?;
                std::fs::File::create(
                    flow_file_dir
                        .parent()
                        .unwrap_or(&flow_file_dir)
                        .join("__init__.py"),
                )?;
                std::fs::File::create(flow_file_dir.join("__init__.py"))?;

                flow_file.write_all(PYTHON_BASE_FLOW_TEMPLATE.as_bytes())?;
                aggregations_file.write_all(PTYHON_BASE_AGG_SAMPLE_TEMPLATE.as_bytes())?;
            }
        }

        apis_file.write_all(BASE_APIS_SAMPLE_TEMPLATE.as_bytes())?;

        Ok(())
    }

    pub fn create_vscode_files(&self) -> Result<(), ProjectFileError> {
        let vscode_dir = self.vscode_dir();

        let ext_file_path = vscode_dir.join(VSCODE_EXT_FILE);
        let settings_file_path = vscode_dir.join(VSCODE_SETTINGS_FILE);

        let mut ext_file = std::fs::File::create(ext_file_path)?;
        let mut settings_file = std::fs::File::create(settings_file_path)?;

        ext_file.write_all(VSCODE_EXTENSIONS_TEMPLATE.as_bytes())?;
        settings_file.write_all(VSCODE_SETTINGS_TEMPLATE.as_bytes())?;

        Ok(())
    }

    pub fn app_dir(&self) -> PathBuf {
        let mut app_dir = self.project_location.clone();
        app_dir.push(APP_DIR);

        debug!("App dir: {:?}", app_dir);

        if !app_dir.exists() {
            std::fs::create_dir_all(&app_dir).expect("Failed to create app directory");
        }
        app_dir
    }

    pub fn schemas_dir(&self) -> PathBuf {
        let mut schemas_dir = self.app_dir();
        schemas_dir.push(SCHEMAS_DIR);

        if !schemas_dir.exists() {
            std::fs::create_dir_all(&schemas_dir).expect("Failed to create schemas directory");
        }

        debug!("Schemas dir: {:?}", schemas_dir);
        schemas_dir
    }

    pub fn flows_dir(&self) -> PathBuf {
        let flows_dir = self.app_dir().join(FLOWS_DIR);

        if !flows_dir.exists() {
            std::fs::create_dir_all(&flows_dir).expect("Failed to create flows directory");
        }

        debug!("Flows dir: {:?}", flows_dir);
        flows_dir
    }

    pub fn aggregations_dir(&self) -> PathBuf {
        let aggregations_dir = self.app_dir().join(AGGREGATIONS_DIR);

        if !aggregations_dir.exists() {
            std::fs::create_dir_all(&aggregations_dir)
                .expect("Failed to create aggregations directory");
        }

        debug!("Aggregations dir: {:?}", aggregations_dir);
        aggregations_dir
    }

    pub fn consumption_dir(&self) -> PathBuf {
        let apis_dir = self.app_dir().join(CONSUMPTION_DIR);

        if !apis_dir.exists() {
            std::fs::create_dir_all(&apis_dir).expect("Failed to create consumption directory");
        }

        debug!("Consumptions dir: {:?}", apis_dir);
        apis_dir
    }

    pub fn vscode_dir(&self) -> PathBuf {
        let mut vscode_dir = self.project_location.clone();
        vscode_dir.push(VSCODE_DIR);

        if !vscode_dir.exists() {
            std::fs::create_dir_all(&vscode_dir).expect("Failed to create .vscode directory");
        }

        debug!(".vscode dir: {:?}", vscode_dir);
        vscode_dir
    }

    // This is a Result of io::Error because the caller
    // can be returning a Result of io::Error or a Routine Failure
    pub fn internal_dir(&self) -> Result<PathBuf, ProjectFileError> {
        let mut internal_dir = self.project_location.clone();
        internal_dir.push(CLI_PROJECT_INTERNAL_DIR);

        if !internal_dir.is_dir() {
            if internal_dir.exists() {
                debug!("Internal dir exists as a file: {:?}", internal_dir);
                return Err(ProjectFileError::Other {
                    message: format!(
                        "The {} file exists but is not a directory",
                        CLI_PROJECT_INTERNAL_DIR
                    ),
                });
            } else {
                debug!("Creating internal dir: {:?}", internal_dir);
                std::fs::create_dir_all(&internal_dir)?;
            }
        } else {
            debug!("Internal directory Exists: {:?}", internal_dir);
        }

        Ok(internal_dir)
    }

    pub fn delete_internal_dir(&self) -> Result<(), ProjectFileError> {
        let internal_dir = self.internal_dir()?;
        Ok(std::fs::remove_dir_all(internal_dir)?)
    }

    pub fn create_internal_clickhouse_volume(&self) -> anyhow::Result<()> {
        let clikhouse_dir = self.internal_dir()?;

        std::fs::create_dir_all(clikhouse_dir.join(CLI_DEV_CLICKHOUSE_VOLUME_DIR_LOGS))?;
        std::fs::create_dir_all(clikhouse_dir.join(CLI_DEV_CLICKHOUSE_VOLUME_DIR_DATA))?;
        std::fs::create_dir_all(clikhouse_dir.join(CLI_DEV_CLICKHOUSE_VOLUME_DIR_CONFIG_SCRIPTS))?;
        std::fs::create_dir_all(clikhouse_dir.join(CLI_DEV_CLICKHOUSE_VOLUME_DIR_CONFIG_USERS))?;

        Ok(())
    }

    pub fn create_internal_redpanda_volume(&self) -> anyhow::Result<()> {
        let redpanda_dir = self.internal_dir()?;
        std::fs::create_dir_all(redpanda_dir.join(CLI_DEV_REDPANDA_VOLUME_DIR))?;
        Ok(())
    }

    pub fn version(&self) -> &str {
        match &self.language_project_config {
            LanguageProjectConfig::Typescript(package_json) => &package_json.version,
            LanguageProjectConfig::Python(package_json) => &package_json.version,
        }
    }

    pub fn old_version_location(&self, version: &str) -> Result<PathBuf, ProjectFileError> {
        let mut old_base_path = self.internal_dir()?;
        old_base_path.push(CLI_INTERNAL_VERSIONS_DIR);
        old_base_path.push(version);

        Ok(old_base_path)
    }

    pub fn delete_old_versions(&self) -> Result<(), ProjectFileError> {
        let mut old_versions = self.internal_dir()?;
        old_versions.push(CLI_INTERNAL_VERSIONS_DIR);

        if old_versions.exists() {
            std::fs::remove_dir_all(old_versions)?;
        }

        Ok(())
    }

    pub fn old_versions_sorted(&self) -> Vec<String> {
        let mut old_versions = self
            .supported_old_versions
            .keys()
            .map(|v| parse_version(v))
            .collect::<Vec<Vec<i32>>>();
        old_versions.sort();

        old_versions
            .into_iter()
            .map(|v| version_to_string(&v))
            .collect::<Vec<String>>()
    }

    pub fn get_flows(&self) -> HashMap<String, Vec<String>> {
        let mut flows_map = HashMap::new();

        if let Ok(entries) = std::fs::read_dir(self.flows_dir()) {
            for entry in entries.flatten() {
                if let Some((input_model, output_models)) = self.process_flow_input(&entry) {
                    flows_map.insert(input_model, output_models);
                }
            }
        }

        flows_map
    }

    pub fn get_aggregations(&self) -> HashSet<String> {
        match std::fs::read_dir(self.aggregations_dir()) {
            Ok(files) => files
                .filter_map(Result::ok)
                .filter_map(|entry| entry.file_name().to_str().map(String::from))
                .filter(|file_name| file_name.ends_with(".ts"))
                .map(|file_name| file_name.trim_end_matches(".ts").to_string())
                .collect::<HashSet<_>>(),
            Err(_) => HashSet::new(),
        }
    }

    fn process_flow_input(&self, entry: &std::fs::DirEntry) -> Option<(String, Vec<String>)> {
        let input_model = entry.file_name().to_string_lossy().into_owned();
        let mut output_models = Vec::new();

        if let Ok(output_entries) = std::fs::read_dir(entry.path()) {
            for output_entry in output_entries.flatten() {
                if let Some(output_model) = self.process_flow_output(&output_entry) {
                    output_models.push(output_model);
                }
            }
        }

        if !output_models.is_empty() {
            Some((input_model, output_models))
        } else {
            None
        }
    }

    fn process_flow_output(&self, entry: &std::fs::DirEntry) -> Option<String> {
        if let Ok(file_type) = entry.file_type() {
            if file_type.is_dir() {
                let flow_path = entry.path().join(TS_FLOW_FILE);
                if flow_path.exists() {
                    return Some(entry.file_name().to_string_lossy().into_owned());
                }
            }
        }
        None
    }
}

// Tests
#[cfg(test)]
pub mod tests {
    use super::*;

    fn create_python_project() -> Project {
        Project::new(
            Path::new("tests/python/project"),
            "test_project".to_string(),
            SupportedLanguages::Python,
        )
    }

    fn remove_python_project() {
        let project_files = vec![PROJECT_CONFIG_FILE];
        let project = create_python_project();
        for file in project_files {
            let file_path = project.project_location.join(file);
            if file_path.exists() {
                std::fs::remove_file(file_path).unwrap();
            }
        }
    }

    #[test]
    fn test_new_python_project() {
        let project = create_python_project();

        assert_eq!(project.language, SupportedLanguages::Python);
        assert_eq!(project.name(), "test_project");
    }

    #[test]
    fn test_write_to_disk() {
        let project = create_python_project();
        project.write_to_disk().unwrap();

        assert!(project.project_location.join(PROJECT_CONFIG_FILE).exists());

        remove_python_project();
    }

    #[test]
    fn test_new_python_project_from_file() {
        let project = create_python_project();
        project.write_to_disk().unwrap();

        assert_eq!(project.language, SupportedLanguages::Python);
        assert_eq!(project.name(), "test_project");
    }
}

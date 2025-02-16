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
use std::io::Write;
pub mod python_project;
pub mod typescript_project;

use std::fmt::Debug;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Once;

use crate::cli::local_webserver::LocalWebserverConfig;
use crate::framework::languages::SupportedLanguages;
use crate::framework::python::templates::PYTHON_BASE_MODEL_TEMPLATE;
use crate::framework::python::templates::PYTHON_BASE_STREAMING_FUNCTION_SAMPLE;
use crate::framework::python::templates::{PYTHON_BASE_API_SAMPLE, PYTHON_BASE_BLOCKS_SAMPLE};
use crate::framework::streaming::loader::parse_streaming_function;
use crate::framework::typescript::templates::TS_BASE_MODEL_TEMPLATE;
use crate::framework::typescript::templates::{TS_BASE_APIS_SAMPLE, VS_CODE_PYTHON_SETTINGS};
use crate::framework::typescript::templates::{
    TS_BASE_BLOCKS_SAMPLE, TS_BASE_STREAMING_FUNCTION_SAMPLE,
};
use crate::framework::typescript::templates::{
    VSCODE_EXTENSIONS_TEMPLATE, VSCODE_SETTINGS_TEMPLATE,
};
use crate::framework::versions::Version;
use crate::infrastructure::olap::clickhouse::config::ClickHouseConfig;
use crate::infrastructure::orchestration::temporal::TemporalConfig;
use crate::infrastructure::processes::cron_registry::CronJob;
use crate::infrastructure::redis::redis_client::RedisConfig;
use crate::infrastructure::stream::redpanda::RedpandaConfig;

use crate::project::typescript_project::TypescriptProject;
use crate::utilities::constants::SCRIPTS_DIR;
use config::{Config, ConfigError, Environment, File};
use itertools::Itertools;
use log::debug;
use python_project::PythonProject;
use serde::Deserialize;
use serde::Serialize;

use crate::utilities::constants::CLI_INTERNAL_VERSIONS_DIR;
use crate::utilities::constants::ENVIRONMENT_VARIABLE_PREFIX;
use crate::utilities::constants::PROJECT_CONFIG_FILE;
use crate::utilities::constants::PY_BLOCKS_FILE;
use crate::utilities::constants::README_PREFIX;
use crate::utilities::constants::TSCONFIG_JSON;
use crate::utilities::constants::{APP_DIR, APP_DIR_LAYOUT, CLI_PROJECT_INTERNAL_DIR, SCHEMAS_DIR};
use crate::utilities::constants::{BLOCKS_DIR, TS_BLOCKS_FILE};
use crate::utilities::constants::{
    CONSUMPTION_DIR, FUNCTIONS_DIR, OLD_PROJECT_CONFIG_FILE, SAMPLE_STREAMING_FUNCTION_DEST,
    SAMPLE_STREAMING_FUNCTION_SOURCE, TS_FLOW_FILE,
};
use crate::utilities::constants::{PYTHON_INIT_FILE, PY_API_FILE, TS_API_FILE};
use crate::utilities::constants::{VSCODE_DIR, VSCODE_EXT_FILE, VSCODE_SETTINGS_FILE};
use crate::utilities::git::GitConfig;
use crate::utilities::PathExt;
use crate::utilities::_true;

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
// maintain. With Copilot - it also has the advantage that the boiler plate is really fast to write
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub language: SupportedLanguages,
    #[serde(default)]
    pub redpanda_config: RedpandaConfig,
    pub clickhouse_config: ClickHouseConfig,
    pub http_server_config: LocalWebserverConfig,
    #[serde(default)]
    pub redis_config: RedisConfig,
    #[serde(default)]
    pub git_config: GitConfig,

    #[serde(default)]
    pub temporal_config: TemporalConfig,

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
    pub supported_old_versions: HashMap<Version, String>,
    #[serde(default)]
    pub jwt: Option<JwtConfig>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cron_jobs: Vec<CronJob>,

    #[serde(default)]
    pub features: ProjectFeatures,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectFeatures {
    #[serde(default = "_true")]
    pub streaming_engine: bool,

    #[serde(default)]
    pub workflows: bool,
}

impl Default for ProjectFeatures {
    fn default() -> Self {
        ProjectFeatures {
            streaming_engine: true,
            workflows: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JwtConfig {
    #[serde(default)]
    pub enforce_on_all_consumptions_apis: bool,
    #[serde(default)]
    pub enforce_on_all_ingest_apis: bool,
    pub secret: String,
    pub issuer: String,
    pub audience: String,
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

static STREAMING_FUNCTION_RENAME_WARNING: Once = Once::new();

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

        let language_project_config = match language {
            SupportedLanguages::Typescript => {
                LanguageProjectConfig::Typescript(TypescriptProject::new(name))
            }
            SupportedLanguages::Python => LanguageProjectConfig::Python(PythonProject::new(name)),
        };

        Project {
            language,
            is_production: false,
            project_location: location.clone(),
            redpanda_config: RedpandaConfig::default(),
            clickhouse_config: ClickHouseConfig::default(),
            redis_config: RedisConfig::default(),
            http_server_config: LocalWebserverConfig::default(),
            temporal_config: TemporalConfig::default(),
            language_project_config,
            supported_old_versions: HashMap::new(),
            git_config: GitConfig::default(),
            jwt: None,
            cron_jobs: Vec::new(),
            features: Default::default(),
        }
    }

    pub fn set_is_production_env(&mut self, is_production: bool) {
        self.is_production = is_production;
    }

    pub fn load(directory: &PathBuf) -> Result<Project, ConfigError> {
        let mut project_file = directory.clone();

        // Prioritize the new project file name
        if directory.clone().join(PROJECT_CONFIG_FILE).exists() {
            project_file.push(PROJECT_CONFIG_FILE);
        } else {
            project_file.push(OLD_PROJECT_CONFIG_FILE);
        }

        let mut project_config: Project = Config::builder()
            .add_source(File::from(project_file).required(true))
            .add_source(
                Environment::with_prefix(ENVIRONMENT_VARIABLE_PREFIX)
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
        // Write to disk what is common to all project types, the moose.config.toml
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

        // TODO probably move this to the respective project language modules
        if self.language == SupportedLanguages::Python {
            std::fs::File::create(app_dir.join(PYTHON_INIT_FILE))?;
        }

        for dir in APP_DIR_LAYOUT.iter() {
            let to_create = app_dir.join(dir);
            std::fs::create_dir_all(&to_create)?;

            // TODO probably move this to the respective project language modules
            if self.language == SupportedLanguages::Python {
                std::fs::File::create(to_create.join(PYTHON_INIT_FILE))?;
            }
        }

        Ok(())
    }

    pub fn create_base_app_files(&self, no_samples: bool) -> Result<(), std::io::Error> {
        // Common file paths
        let readme_file_path = self.project_location.join("README.md");
        let blocks_dir = self.blocks_dir();

        if !no_samples {
            self.write_file(
                &readme_file_path,
                README_PREFIX.to_owned() + include_str!("../../../README.md"),
            )?;
        }
        match self.language {
            // TODO move the templates to the respective project language modules
            SupportedLanguages::Typescript => {
                let tsconfig = self.project_location.join(TSCONFIG_JSON);
                self.write_file(
                    &tsconfig,
                    serde_json::to_string_pretty(&serde_json::json!(
                        {
                            "compilerOptions": {
                                "outDir": "dist",
                                "esModuleInterop": true,
                                "paths": {
                                  "datamodels/*": ["./app/datamodels/*"],
                                  "versions/*": ["./.moose/versions/*"]
                                },
                                "plugins": [
                                    {
                                        "transform": "./node_modules/@514labs/moose-lib/dist/consumption-apis/insertTypiaValidation.js",
                                        "transformProgram": true
                                    },
                                    {
                                        "transform": "typia/lib/transform"
                                    }
                                ],
                                "strictNullChecks": true
                            }
                        }
                    ))
                    .expect("formatting `serde_json::Value` with string keys never fails"),
                )?;

                if !no_samples {
                    let apis_file_path = self.consumption_dir().join(TS_API_FILE);
                    let base_model_file_path = self.data_models_dir().join("models.ts");
                    let function_file_path = self.streaming_func_dir().join(format!(
                        "{}__{}.ts",
                        SAMPLE_STREAMING_FUNCTION_SOURCE, SAMPLE_STREAMING_FUNCTION_DEST
                    ));
                    let blocks_file_path = blocks_dir.join(TS_BLOCKS_FILE);

                    self.write_file(&apis_file_path, TS_BASE_APIS_SAMPLE.to_string())?;
                    self.write_file(&base_model_file_path, TS_BASE_MODEL_TEMPLATE.to_string())?;
                    self.write_file(
                        &function_file_path,
                        TS_BASE_STREAMING_FUNCTION_SAMPLE.to_string(),
                    )?;

                    self.write_file(&blocks_file_path, TS_BASE_BLOCKS_SAMPLE.to_string())?;
                }
            }
            SupportedLanguages::Python if !no_samples => {
                let apis_file_path = self.consumption_dir().join(PY_API_FILE);
                let base_model_file_path = self.data_models_dir().join("models.py");
                let function_file_path = self.streaming_func_dir().join(format!(
                    "{}__{}.py",
                    SAMPLE_STREAMING_FUNCTION_SOURCE, SAMPLE_STREAMING_FUNCTION_DEST
                ));
                let blocks_file_path = blocks_dir.join(PY_BLOCKS_FILE);

                // Write Python specific templates
                self.write_file(&apis_file_path, PYTHON_BASE_API_SAMPLE.to_string())?;
                self.write_file(
                    &base_model_file_path,
                    PYTHON_BASE_MODEL_TEMPLATE.to_string(),
                )?;
                self.write_file(
                    &function_file_path,
                    PYTHON_BASE_STREAMING_FUNCTION_SAMPLE.to_string(),
                )?;
                self.write_file(&blocks_file_path, PYTHON_BASE_BLOCKS_SAMPLE.to_string())?;
            }
            SupportedLanguages::Python => {} // nothing to do
        }

        Ok(())
    }

    fn write_file(&self, path: &PathBuf, content: String) -> Result<(), std::io::Error> {
        let mut file = std::fs::File::create(path)?;
        let content = if let Some(without_starting_empty_line) = content.strip_prefix('\n') {
            without_starting_empty_line
        } else {
            &content
        };
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    pub fn create_vscode_files(&self) -> Result<(), ProjectFileError> {
        let vscode_dir = self.vscode_dir();

        let ext_file_path = vscode_dir.join(VSCODE_EXT_FILE);
        let settings_file_path = vscode_dir.join(VSCODE_SETTINGS_FILE);

        let mut ext_file = std::fs::File::create(ext_file_path)?;
        let mut settings_file = std::fs::File::create(settings_file_path)?;

        ext_file.write_all(VSCODE_EXTENSIONS_TEMPLATE.as_bytes())?;
        settings_file.write_all(
            VSCODE_SETTINGS_TEMPLATE
                .replace(
                    "{language_specific_settings}",
                    match self.language {
                        SupportedLanguages::Typescript => "",
                        SupportedLanguages::Python => VS_CODE_PYTHON_SETTINGS,
                    },
                )
                .as_bytes(),
        )?;

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

    pub fn scripts_dir(&self) -> PathBuf {
        let mut scripts_dir = self.app_dir();
        scripts_dir.push(SCRIPTS_DIR);

        if !scripts_dir.exists() {
            std::fs::create_dir_all(&scripts_dir).expect("Failed to create scripts directory");
        }

        scripts_dir
    }

    pub fn data_models_dir(&self) -> PathBuf {
        let mut schemas_dir = self.app_dir();
        schemas_dir.push(SCHEMAS_DIR);

        if !schemas_dir.exists() {
            std::fs::create_dir_all(&schemas_dir).expect("Failed to create schemas directory");
        }

        debug!("Schemas dir: {:?}", schemas_dir);
        schemas_dir
    }

    // Will start to be more useful when we version more than just the data models
    pub fn versioned_data_model_dir(&self, version: &str) -> Result<PathBuf, ProjectFileError> {
        if version == self.cur_version().as_str() {
            Ok(self.data_models_dir())
        } else {
            Ok(self.old_version_location(version)?)
        }
    }

    pub fn streaming_func_dir(&self) -> PathBuf {
        let functions_dir = self.app_dir().join(FUNCTIONS_DIR);

        if !functions_dir.exists() {
            let flows_dir = self.app_dir().join("flows");
            if flows_dir.exists() {
                STREAMING_FUNCTION_RENAME_WARNING.call_once(|| {
                    println!("❗️Action Required: 'Flows' are now called 'Functions.' Please rename the directory.");
                });
                return flows_dir;
            }

            std::fs::create_dir_all(&functions_dir).expect("Failed to create functions directory");
        }

        debug!("Functions dir: {:?}", functions_dir);
        functions_dir
    }

    pub fn blocks_dir(&self) -> PathBuf {
        let blocks_dir = self.app_dir().join(BLOCKS_DIR);

        if !blocks_dir.exists() {
            std::fs::create_dir_all(&blocks_dir).expect("Failed to create blocks directory");
        }

        debug!("Blocks dir: {:?}", blocks_dir);
        blocks_dir
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

    pub fn cur_version(&self) -> &Version {
        match &self.language_project_config {
            LanguageProjectConfig::Typescript(package_json) => &package_json.version,
            LanguageProjectConfig::Python(proj) => &proj.version,
        }
    }

    pub fn old_versions_sorted(&self) -> Vec<String> {
        self.supported_old_versions
            .keys()
            .sorted()
            .map(|v| v.to_string())
            .collect()
    }

    pub fn versions(&self) -> Vec<String> {
        let mut versions = self.old_versions_sorted();
        versions.push(self.cur_version().to_string());
        versions
    }

    pub fn get_functions(&self) -> HashMap<String, Vec<String>> {
        let mut functions_map = HashMap::new();

        if let Ok(entries) = std::fs::read_dir(self.streaming_func_dir()) {
            // flatten here means ignoring the Err case
            for entry in entries.flatten() {
                if entry.file_type().is_ok_and(|t| t.is_file())
                    && entry.path().ext_is_supported_lang()
                {
                    parse_streaming_function(
                        entry
                            .path()
                            .with_extension("")
                            .file_name()
                            .unwrap()
                            .to_string_lossy()
                            .as_ref(),
                    )
                    .iter()
                    .for_each(|(input, output)| {
                        if let Some(output_str) = output {
                            functions_map
                                .entry(input.to_string())
                                .or_insert_with(Vec::new)
                                .push(output_str.to_string());
                        }
                    });
                } else if let Some((input_model, output_models)) =
                    self.process_function_input(&entry)
                {
                    functions_map.insert(input_model, output_models);
                }
            }
        }

        functions_map
    }

    fn process_function_input(&self, entry: &std::fs::DirEntry) -> Option<(String, Vec<String>)> {
        let input_model = entry.file_name().to_string_lossy().into_owned();
        let mut output_models = Vec::new();

        if let Ok(output_entries) = std::fs::read_dir(entry.path()) {
            for output_entry in output_entries.flatten() {
                if let Some(output_model) = self.process_function_output(&output_entry) {
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

    fn process_function_output(&self, entry: &std::fs::DirEntry) -> Option<String> {
        if let Ok(file_type) = entry.file_type() {
            if file_type.is_dir() {
                let function_path = entry.path().join(TS_FLOW_FILE);
                if function_path.exists() {
                    return Some(entry.file_name().to_string_lossy().into_owned());
                }
            }
        }
        None
    }

    /**
     * Check if the project is running in a docker constainer we built from the CLI
     * The docker container will have a special environment variable that is set by the build
     * image that we can check for. DOCKER_IMAGE=true
     */
    pub fn is_docker_image(&self) -> bool {
        std::env::var("DOCKER_IMAGE").unwrap_or("false".to_string()) == "true"
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

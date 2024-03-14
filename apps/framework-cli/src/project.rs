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

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::io::Write;
use std::sync::Mutex;
pub mod typescript_project;

use std::fmt::Debug;
use std::path::Path;
use std::path::PathBuf;

use config::{Config, ConfigError, Environment, File};
use log::debug;
use serde::Deserialize;
use serde::Serialize;

use crate::cli::local_webserver::LocalWebserverConfig;
use crate::framework::languages::SupportedLanguages;
use crate::framework::readme::BASE_README_TEMPLATE;
use crate::framework::schema::templates::BASE_MODEL_TEMPLATE;
use crate::framework::transform::TRANSFORM_FILE;
use crate::infrastructure::console::ConsoleConfig;
use crate::infrastructure::olap::clickhouse::config::ClickhouseConfig;
use crate::infrastructure::stream::redpanda::RedpandaConfig;
use crate::project::typescript_project::TypescriptProject;

use crate::utilities::constants::{APP_DIR, APP_DIR_LAYOUT, CLI_PROJECT_INTERNAL_DIR, SCHEMAS_DIR};
use crate::utilities::constants::{DENO_DIR, DENO_TRANSFORM};
use crate::utilities::constants::{FLOWS_DIR, PROJECT_CONFIG_FILE};

lazy_static! {
    pub static ref PROJECT: Mutex<Project> = Mutex::new(Project {
        language: SupportedLanguages::Typescript,
        redpanda_config: RedpandaConfig::default(),
        clickhouse_config: ClickhouseConfig::default(),
        http_server_config: LocalWebserverConfig::default(),
        console_config: ConsoleConfig::default(),
        language_project_config: LanguageProjectConfig::Typescript(TypescriptProject::default()),
        project_location: PathBuf::new(),
        supported_old_versions: HashMap::new(),
    });
}

// We have explored using a Generic associated Types as well as
// Dynamic Dispatch to handle the different types of projects
// the approach with enums is the one that is the simplest to put into practice and
// maintain. With Copilot - it also has the advaantage that the boiler plate is really fast to write
#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    pub language: SupportedLanguages,
    pub redpanda_config: RedpandaConfig,
    pub clickhouse_config: ClickhouseConfig,
    pub http_server_config: LocalWebserverConfig,
    pub console_config: ConsoleConfig,

    #[serde(skip)]
    pub language_project_config: LanguageProjectConfig,
    #[serde(skip)]
    pub project_location: PathBuf,

    #[serde(default = "HashMap::new")]
    pub supported_old_versions: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LanguageProjectConfig {
    Typescript(TypescriptProject),
}

impl Default for LanguageProjectConfig {
    fn default() -> Self {
        LanguageProjectConfig::Typescript(TypescriptProject::default())
    }
}

impl Project {
    pub fn name(&self) -> String {
        match &self.language_project_config {
            LanguageProjectConfig::Typescript(p) => p.name.clone(),
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
                project_location: location.clone(),
                redpanda_config: RedpandaConfig::default(),
                clickhouse_config: ClickhouseConfig::default(),
                http_server_config: LocalWebserverConfig::default(),
                console_config: ConsoleConfig::default(),
                language_project_config: LanguageProjectConfig::Typescript(TypescriptProject::new(
                    name,
                )),
                supported_old_versions: HashMap::new(),
            },
        }
    }

    pub fn load(directory: PathBuf) -> Result<Project, ConfigError> {
        let mut project_file = directory.clone();
        project_file.push(PROJECT_CONFIG_FILE);

        let mut project_config: Project = Config::builder()
            .add_source(File::from(project_file).required(true))
            .add_source(
                Environment::with_prefix("MOOSE")
                    .prefix_separator("_")
                    .separator("__"),
            )
            .build()?
            .try_deserialize()?;

        project_config.project_location = directory.clone();

        match project_config.language {
            SupportedLanguages::Typescript => {
                let ts_config = TypescriptProject::load(directory)?;
                project_config.language_project_config =
                    LanguageProjectConfig::Typescript(ts_config);
                Ok(project_config)
            }
        }
    }

    pub fn load_from_current_dir() -> Result<Project, ConfigError> {
        let current_dir = std::env::current_dir().expect("Failed to get the current directory");
        Project::load(current_dir)
    }

    pub fn write_to_disk(&self) -> Result<(), anyhow::Error> {
        // Write to disk what is common to all project types, the project.toml
        let project_file = self.project_location.join(PROJECT_CONFIG_FILE);
        let toml_project = toml::to_string(&self)?;

        std::fs::write(project_file, toml_project)?;

        // Write language specific files to disk
        match &self.language_project_config {
            LanguageProjectConfig::Typescript(p) => p.write_to_disk(self.project_location.clone()),
        }
    }

    pub fn setup_app_dir(&self) -> Result<(), std::io::Error> {
        let app_dir = self.app_dir();
        std::fs::create_dir_all(&app_dir)?;

        for dir in APP_DIR_LAYOUT.iter() {
            let to_create = app_dir.join(dir);
            std::fs::create_dir_all(to_create)?;
        }

        Ok(())
    }

    pub fn create_deno_files(&self) -> Result<(), std::io::Error> {
        let deno_dir = self.internal_dir()?.join(DENO_DIR);
        let transform_file_path = deno_dir.join(DENO_TRANSFORM);

        let mut transform_file = std::fs::File::create(transform_file_path)?;

        transform_file.write_all(TRANSFORM_FILE.as_bytes())?;

        Ok(())
    }

    pub fn create_base_app_files(&self) -> Result<(), std::io::Error> {
        let app_dir = self.app_dir();
        let readme_file_path = app_dir.join("README.md");
        let base_model_file_path = self.schemas_dir().join("models.prisma");

        let mut readme_file = std::fs::File::create(readme_file_path)?;
        let mut base_model_file = std::fs::File::create(base_model_file_path)?;

        readme_file.write_all(BASE_README_TEMPLATE.as_bytes())?;
        base_model_file.write_all(BASE_MODEL_TEMPLATE.as_bytes())?;

        Ok(())
    }

    pub fn app_dir(&self) -> PathBuf {
        let mut app_dir = self.project_location.clone();
        app_dir.push(APP_DIR);

        debug!("App dir: {:?}", app_dir);
        app_dir
    }

    pub fn schemas_dir(&self) -> PathBuf {
        let mut schemas_dir = self.app_dir();
        schemas_dir.push(SCHEMAS_DIR);

        debug!("Schemas dir: {:?}", schemas_dir);
        schemas_dir
    }

    pub fn flows_dir(&self) -> PathBuf {
        let mut flows_dir = self.app_dir();
        flows_dir.push(FLOWS_DIR);

        debug!("Flows dir: {:?}", flows_dir);
        flows_dir
    }

    // This is a Result of io::Error because the caller
    // can be returning a Result of io::Error or a Routine Failure
    pub fn internal_dir(&self) -> std::io::Result<PathBuf> {
        let mut internal_dir = self.project_location.clone();
        internal_dir.push(CLI_PROJECT_INTERNAL_DIR);

        if !internal_dir.is_dir() {
            if internal_dir.exists() {
                debug!("Internal dir exists as a file: {:?}", internal_dir);
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "The {} file exists but is not a directory",
                        CLI_PROJECT_INTERNAL_DIR
                    ),
                ));
            } else {
                debug!("Creating internal dir: {:?}", internal_dir);
                std::fs::create_dir_all(&internal_dir)?;
            }
        } else {
            debug!("Internal directory Exists: {:?}", internal_dir);
        }

        Ok(internal_dir)
    }

    pub fn version(&self) -> &str {
        match &self.language_project_config {
            LanguageProjectConfig::Typescript(package_json) => &package_json.version,
        }
    }
}

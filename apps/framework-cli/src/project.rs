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

use std::io::Write;
pub mod project_config_file;
pub mod typescript_project;

use std::fmt::Debug;
use std::path::Path;
use std::path::PathBuf;

use config::{Config, ConfigError, File};
use log::debug;
use serde::Serialize;

use crate::cli::local_webserver::LocalWebserverConfig;
use crate::framework::languages::SupportedLanguages;
use crate::framework::readme::BASE_README_TEMPLATE;
use crate::framework::schema::templates::BASE_MODEL_TEMPLATE;
use crate::infrastructure::console::ConsoleConfig;
use crate::infrastructure::olap::clickhouse::config::ClickhouseConfig;
use crate::infrastructure::stream::redpanda::RedpandaConfig;
use crate::project::project_config_file::ProjectConfigFile;
use crate::project::typescript_project::TypescriptProject;

use crate::utilities::constants::PROJECT_CONFIG_FILE;
use crate::utilities::constants::{APP_DIR, APP_DIR_LAYOUT, CLI_PROJECT_INTERNAL_DIR, SCHEMAS_DIR};

// We have explored using a Generic associated Types as well as
// Dynaimc Dispatch to handle the different types of projects
// the approach with enums is the one that is the simplest to put into practice and
// maintain. With Copilot - it also has the advaantage that the boiler plate is really fast to write
#[derive(Debug)]
pub enum Project {
    Typescript(TypescriptProject),
}

impl Serialize for Project {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self {
            Project::Typescript(p) => p.serialize(serializer),
        }
    }
}

impl Project {
    pub fn new(dir_location: &Path, name: String, language: SupportedLanguages) -> Project {
        let mut location = dir_location.to_path_buf();

        location = location
            .canonicalize()
            .expect("The directory provided does not exist");

        debug!("Package.json file location: {:?}", location);

        match language {
            SupportedLanguages::Typescript => {
                Project::Typescript(TypescriptProject::new(&location, name))
            }
        }
    }

    pub fn load(directory: PathBuf) -> Result<Project, ConfigError> {
        let project_config = load_project_file(directory.clone())?;

        match project_config.language {
            SupportedLanguages::Typescript => Ok(Project::Typescript(TypescriptProject::load(
                project_config,
                directory,
            )?)),
        }
    }

    pub fn load_from_current_dir() -> Result<Project, ConfigError> {
        let current_dir = std::env::current_dir().expect("Failed to get the current directory");
        Project::load(current_dir)
    }

    pub fn name(&self) -> &String {
        match self {
            Project::Typescript(p) => &p.name,
        }
    }

    pub fn language(&self) -> SupportedLanguages {
        match self {
            Project::Typescript(_) => SupportedLanguages::Typescript,
        }
    }

    pub fn project_location(&self) -> &PathBuf {
        match self {
            Project::Typescript(p) => &p.project_location,
        }
    }

    pub fn redpanda_config(&self) -> &RedpandaConfig {
        match self {
            Project::Typescript(p) => &p.redpanda_config,
        }
    }

    pub fn clickhouse_config(&self) -> &ClickhouseConfig {
        match self {
            Project::Typescript(p) => &p.clickhouse_config,
        }
    }

    pub fn console_config(&self) -> &ConsoleConfig {
        match self {
            Project::Typescript(p) => &p.console_config,
        }
    }

    pub fn http_server_config(&self) -> &LocalWebserverConfig {
        match self {
            Project::Typescript(p) => &p.http_server_config,
        }
    }

    pub fn write_to_disk(&self) -> Result<(), anyhow::Error> {
        // Write to disk what is common to all project types, the project.toml
        let project_file = self.project_location().join(PROJECT_CONFIG_FILE);

        let config = ProjectConfigFile {
            language: self.language(),
            redpanda: self.redpanda_config().clone(),
            clickhouse: self.clickhouse_config().clone(),
            http_server: self.http_server_config().clone(),
            console: self.console_config().clone(),
        };

        let toml_project = toml::to_string(&config)?;

        std::fs::write(&project_file, toml_project)?;

        // Write language specific files to disk
        match self {
            Project::Typescript(p) => p.write_to_disk(),
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
        let mut app_dir = self.project_location().clone();
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

    // This is a Result of io::Error because the caller
    // can be returning a Result of io::Error or a Routine Failure
    pub fn internal_dir(&self) -> std::io::Result<PathBuf> {
        let mut internal_dir = self.project_location().clone();
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
}

fn load_project_file(directory: PathBuf) -> Result<ProjectConfigFile, ConfigError> {
    let mut project_file = directory.clone();
    project_file.push(PROJECT_CONFIG_FILE);

    let s = Config::builder()
        .add_source(File::from(project_file).required(true))
        .build()?;

    s.try_deserialize()
}

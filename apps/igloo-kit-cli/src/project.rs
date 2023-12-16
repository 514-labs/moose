//! # Project
//!
//! This module contains the `Project` struct, which represents a users project.
//! These projects are data-intensive applications or services.
//! A project is initialized using the `igloo init` command and is stored
//!  in the `$PROJECT_PATH/.igloo` directory.
//!
//! The `Project` struct contains the following fields:
//! - `name` - The name of the project
//! - `language` - The language of the project
//! - `project_file_location` - The location of the project file on disk
//! ```

use std::path::PathBuf;

use crate::cli::local_webserver::LocalWebserverConfig;
use crate::constants::{
    APP_DIR, APP_DIR_LAYOUT, CLI_PROJECT_INTERNAL_DIR, PROJECT_CONFIG_FILE, SCHEMAS_DIR,
};
use crate::framework::languages::SupportedLanguages;
use crate::infrastructure::olap::clickhouse::config::ClickhouseConfig;
use crate::infrastructure::stream::redpanda::RedpandaConfig;
use config::{Config, ConfigError, File};
use log::debug;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ProjectConfigFile {
    pub name: String,
    pub language: SupportedLanguages,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Project {
    pub name: String,
    pub language: SupportedLanguages,
    pub project_file_location: PathBuf,
    pub redpanda_config: RedpandaConfig,
    pub clickhouse_config: ClickhouseConfig,
    pub local_webserver_config: LocalWebserverConfig,
}

impl Project {
    pub fn new(name: String, language: SupportedLanguages, location: PathBuf) -> Self {
        Self {
            name,
            language,
            project_file_location: location,
            redpanda_config: RedpandaConfig::default(), 
            clickhouse_config: ClickhouseConfig::default(), 
            local_webserver_config: LocalWebserverConfig::default(),
        }
    }

    pub fn from_dir(dir_location: &Path, name: String, language: SupportedLanguages) -> Self {
        //! Creates a new `Project` from a directory path.
        //! 
        //! This function cleans up any relative paths and canonicalizes the path.
        let mut location = dir_location.to_path_buf();
        location = location
            .canonicalize()
            .expect("The directory provided does not exist");
        location.push(PROJECT_CONFIG_FILE);

        debug!("Project file location: {:?}", location);

        Self {
            name,
            language,
            project_file_location: location,
            redpanda_config: RedpandaConfig::default(), // TODO: Add the ability for the developer to configure this
            clickhouse_config: ClickhouseConfig::default(), // TODO: Add the ability for the developer to configure this
            local_webserver_config: LocalWebserverConfig::default(), // TODO: Add the ability for the developer to configure this
        }
    }

    pub fn load_from_current_dir() -> Result<Self, ConfigError> {
        let current_dir = std::env::current_dir().expect("Failed to get the current directory");
        Self::load(current_dir)
    }

    pub fn load(directory: PathBuf) -> Result<Self, ConfigError> {
        let mut project_file = directory.clone();
        project_file.push(PROJECT_CONFIG_FILE);

        let project_file_location = project_file
            .clone()
            .into_os_string()
            .into_string()
            .expect("Failed to convert project file location to string");

        let s = Config::builder()
            .add_source(File::from(project_file).required(true))
            .set_override("project_file_location", project_file_location)?
            .build()?;

        s.try_deserialize()
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

    pub fn app_dir(&self) -> PathBuf {
        let mut app_dir = self.project_file_location.clone();
        app_dir.pop();
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
    // can be retruning a Result of io::Error or a  Routine Failure
    pub fn internal_dir(&self) -> std::io::Result<PathBuf> {
        let mut internal_dir = self.project_file_location.clone();
        internal_dir.pop();
        internal_dir.push(CLI_PROJECT_INTERNAL_DIR);

        if !internal_dir.is_dir() {
            if internal_dir.exists() {
                debug!("Internal dir exists as a file: {:?}", internal_dir);
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "The .igloo file exists but is not a directory",
                );
            } else {
                debug!("Creating internal dir: {:?}", internal_dir);
                std::fs::create_dir_all(&internal_dir)?;
            }
        } else {
            debug!("Internal directory Exists: {:?}", internal_dir);
        }

        Ok(internal_dir)
    }

    pub fn write_to_file(&self) -> Result<(), std::io::Error> {
        let config_file = ProjectConfigFile {
            name: self.name.clone(),
            language: self.language,
        };

        let toml_project = toml::to_string(&config_file);

        match toml_project {
            Ok(project) => {
                std::fs::write(&self.project_file_location, project)?;
                Ok(())
            }
            Err(err) => {
                println!("Failed to serialize project: {}", err);
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to serialize project",
                ))
            }
        }
    }
}

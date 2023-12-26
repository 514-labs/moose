use config::{Config, ConfigError, Environment, File};
use home::home_dir;
use serde::Deserialize;
use std::path::PathBuf;

use super::display::{Message, MessageType};
use super::logger::LoggerSettings;
use crate::constants::{CLI_CONFIG_FILE, CLI_USER_DIRECTORY};

/// # Config
/// Module to handle reading the config file from the user's home directory and configuring the CLI
///
/// ## Suggested Improvements
/// - add a config file option to the CLI
/// - add a config file generator to the CLI
/// - add a config file validation and error handling
///

const ENVIRONMENT_VARIABLE_PREFIX: &str = "IGLOO";

#[derive(Deserialize, Debug)]
pub struct Features {
    pub coming_soon_wall: bool,
}

impl Default for Features {
    fn default() -> Self {
        Features {
            coming_soon_wall: true,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Settings {
    #[serde(default)]
    pub logger: LoggerSettings,
    #[serde(default)]
    pub features: Features,
}

fn config_path() -> PathBuf {
    let mut path: PathBuf = home_dir().unwrap();
    path.push(CLI_CONFIG_FILE);
    path.to_owned()
}

pub fn user_directory() -> PathBuf {
    let mut path: PathBuf = home_dir().unwrap();
    path.push(CLI_USER_DIRECTORY);
    path.to_owned()
}

pub fn setup_user_directory() -> Result<(), std::io::Error> {
    let path = user_directory();
    std::fs::create_dir_all(path.clone())?;
    Ok(())
}

// TODO: Turn this part of the code into a routine
pub fn read_settings() -> Result<Settings, ConfigError> {
    show_message!(
        MessageType::Info,
        Message {
            action: "Init".to_string(),
            details: "Loading config...".to_string(),
        }
    );

    let config_file_location: PathBuf = config_path();

    let s = Config::builder()
        // Start off by merging in the "default" values
        // Add the local configuration
        .add_source(File::from(config_file_location).required(false))
        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `IGLOO_DEBUG=1 ./target/app` would set the `debug` key
        .add_source(
            Environment::with_prefix(ENVIRONMENT_VARIABLE_PREFIX)
                .try_parsing(true)
                .separator("-"),
        )
        .build()?;

    s.try_deserialize()
}

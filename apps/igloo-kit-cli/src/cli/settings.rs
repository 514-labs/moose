use crate::infrastructure::olap::clickhouse::config::ClickhouseConfig;
use crate::infrastructure::stream::redpanda::RedpandaConfig;
use config::{Config, ConfigError, Environment, File};
use home::home_dir;
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use super::display::{show_message, Message, MessageType};
use super::local_webserver::LocalWebserverConfig;
use super::logger::LoggerSettings;
use super::CommandTerminal;

/// # Config
/// Module to handle reading the config file from the user's home directory and configuring the CLI
///
/// ## Suggested Improvements
/// - add clickhouse and redpanda config to the config file
/// - add a config file option to the CLI
/// - add a config file generator to the CLI
/// - add a config file validation and error handling
///

const CONFIG_FILE: &str = ".igloo-config.toml";
const USER_DIRECTORY: &str = ".igloo";
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
    #[serde(default)]
    pub clickhouse: ClickhouseConfig,
    #[serde(default)]
    pub redpanda: RedpandaConfig,
    #[serde(default)]
    pub local_webserver: LocalWebserverConfig,
}

fn config_path() -> PathBuf {
    let mut path: PathBuf = home_dir().unwrap();
    path.push(CONFIG_FILE);
    path.to_owned()
}

pub fn user_directory() -> PathBuf {
    let mut path: PathBuf = home_dir().unwrap();
    path.push(USER_DIRECTORY);
    path.to_owned()
}

pub fn setup_user_directory() -> Result<(), std::io::Error> {
    let path = user_directory();
    std::fs::create_dir_all(path.clone())?;
    Ok(())
}

// TODO: Turn this part of the code into a routine
pub fn read_settings(term: Arc<RwLock<CommandTerminal>>) -> Result<Settings, ConfigError> {
    show_message(
        term,
        MessageType::Info,
        Message {
            action: "Init".to_string(),
            details: "Loading config...".to_string(),
        },
    );

    let config_file_location: PathBuf = config_path();

    let s = Config::builder()
        // Start off by merging in the "default" values
        // Add the local configuration
        .add_source(File::from(config_file_location).required(false))
        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `IGLOO_DEBUG=1 ./target/app` would set the `debug` key
        .add_source(Environment::with_prefix(ENVIRONMENT_VARIABLE_PREFIX))
        .build()?;

    s.try_deserialize()
}

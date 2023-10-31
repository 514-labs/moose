use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use std::sync::{RwLock, Arc};
use home::home_dir;
use crate::infrastructure::PANDA_NETWORK;
use crate::infrastructure::olap::clickhouse::config::ClickhouseConfig;
use crate::infrastructure::stream::redpanda::RedpandaConfig;

use super::CommandTerminal;
use super::display::{show_message, MessageType, Message};
use super::local_webserver::LocalWebserverConfig;

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

#[derive(Deserialize, Debug)]
pub struct Config {
    pub features: Features,
    #[serde(default)]
    pub clickhouse: ClickhouseConfig,
    #[serde(default)]
    pub redpanda: RedpandaConfig,
    #[serde(default)]
    pub local_webserver: LocalWebserverConfig,
}

#[derive(Deserialize, Debug)]
pub struct Features {
   pub coming_soon_wall: bool
}

fn config_path() -> PathBuf {
    let mut path: PathBuf = home_dir().unwrap();
    path.push(CONFIG_FILE);
    path.to_owned()
}

fn default_config() -> Config {
    Config { 
        features: Features { coming_soon_wall: true } , 
        clickhouse: ClickhouseConfig::default(), 
        redpanda: RedpandaConfig::default(), 
        local_webserver: LocalWebserverConfig::default() 
    }
}

pub fn read_config(term: Arc<RwLock<CommandTerminal>>) -> Config {
    let config_file_location: PathBuf = config_path();
    match config_file_location.try_exists() {
        Ok(true) => {
            show_message(term, MessageType::Info, Message {
                action: "Loading Config".to_string(),
                details: "Reading configuration from ~/.igloo-config.toml".to_string(),
            });
            let contents: String = fs::read_to_string(config_file_location)
                .expect("Something went wrong reading the config file ");
            toml::from_str(&contents)
                .expect("Something went wrong parsing the config file ")
        }
        _ => {
            default_config()
        }
    }
}

use config::{Config, ConfigError, Environment, File};
use home::home_dir;
use serde::Deserialize;
use std::path::PathBuf;
use toml::Value;
use uuid::Uuid;

use super::display::{Message, MessageType};
use super::logger::LoggerSettings;
use crate::utilities::constants::{CLI_CONFIG_FILE, CLI_USER_DIRECTORY};

/// # Config
/// Module to handle reading the config file from the user's home directory and configuring the CLI
///
/// ## Suggested Improvements
/// - add a config file option to the CLI
/// - add a config file generator to the CLI
/// - add a config file validation and error handling
///

const ENVIRONMENT_VARIABLE_PREFIX: &str = "MOOSE";

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
pub struct Telemetry {
    pub machine_id: String,
    pub enabled: bool,

    #[serde(default)]
    pub is_moose_developer: bool,
}

impl Default for Telemetry {
    fn default() -> Self {
        Telemetry {
            enabled: true,
            is_moose_developer: false,
            machine_id: Uuid::new_v4().to_string(),
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
    pub telemetry: Telemetry,
}

fn config_path() -> PathBuf {
    let mut path: PathBuf = user_directory();
    path.push(CLI_CONFIG_FILE);
    path
}

pub fn user_directory() -> PathBuf {
    let mut path: PathBuf = home_dir().unwrap();
    path.push(CLI_USER_DIRECTORY);
    path
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
        // Eg.. `MOOSE_DEBUG=1 ./target/app` would set the `debug` key
        .add_source(
            Environment::with_prefix(ENVIRONMENT_VARIABLE_PREFIX)
                .try_parsing(true)
                .separator("_"),
        )
        .build()?;

    s.try_deserialize()
}

pub fn init_config_file() -> Result<(), std::io::Error> {
    let path = config_path();
    if !path.exists() {
        let contents_toml = r#"
# Feature flags to hide ongoing feature work on the CLI
[features]

# Coming soon wall on all the CLI commands as we build the MVP.
# if you want to try features as they are built, set this to false
coming_soon_wall=false

# Helps gather insights, identify issues, & improve the user experience
[telemetry]

# Set this to false to opt-out
enabled=true
is_moose_developer=false
machine_id="{{uuid}}"
"#;
        std::fs::write(
            path,
            contents_toml.replace("{{uuid}}", &Uuid::new_v4().to_string()),
        )?;
    } else {
        let data = std::fs::read_to_string(&path)?;
        match data.parse::<Value>() {
            Ok(mut toml) => {
                let telemetry = toml
                    .as_table_mut()
                    .unwrap()
                    .entry("telemetry")
                    .or_insert_with(|| Value::Table(toml::map::Map::new()));

                if let Some(telemetry) = telemetry.as_table_mut() {
                    telemetry.entry("enabled").or_insert(Value::Boolean(true));
                    telemetry
                        .entry("is_moose_developer")
                        .or_insert(Value::Boolean(false));
                    telemetry
                        .entry("machine_id")
                        .or_insert(Value::String(Uuid::new_v4().to_string()));
                }

                std::fs::write(path, toml.to_string())?;
            }
            Err(e) => {
                show_message!(
                    MessageType::Error,
                    Message {
                        action: "Init".to_string(),
                        details: format!("Error parsing config file: {:?}", e),
                    }
                );
            }
        }
    }
    Ok(())
}

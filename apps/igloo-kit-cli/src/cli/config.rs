use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use home::home_dir;
use super::CommandTerminal;
use super::display::{show_message, MessageType, Message};
const CONFIG_FILE: &str = ".igloo-config.toml";

#[derive(Deserialize, Debug)]
pub struct Config {
   pub features: Features
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
   Config { features: Features { coming_soon_wall: true } }
}

pub fn read_config(term: &mut CommandTerminal) -> Config {
    let config_file_location: PathBuf = config_path();
    match config_file_location.try_exists() {
        Ok(true) => {
            show_message(term, MessageType::Info, Message {
                action: "Loading Config",
                details: "Reading configuration from ~/.igloo-config.toml",
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

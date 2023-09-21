use std::collections::{HashSet, HashMap};
use bimap::BiMap;
use std::sync::Arc;
use std::{io::Error, path::PathBuf};

use tokio::sync::Mutex;

use crate::infrastructure::db::clickhouse::ClickhouseConfig;
use crate::{infrastructure, framework};

use super::watcher::RouteMeta;
use super::{watcher, webserver};
use super::{CommandTerminal, user_messages::show_message, MessageType, Message};



pub fn start_containers(term: &mut CommandTerminal, clickhouse_config: ClickhouseConfig) -> Result<(), Error> {
    show_message( term, MessageType::Info, Message {
        action: "Running",
        details: "infrastructure spin up",
    });
    
    infrastructure::spin_up(term, clickhouse_config)?;
    Ok(())
}

pub fn initialize_project(term: &mut CommandTerminal) -> Result<(), Error> {
    let igloo_dir = framework::directories::create_top_level_temp_dir(term)?;
    match framework::directories::create_app_directories(term) {
        Ok(_) => {
            show_message( term, MessageType::Success, Message {
                action: "Finished",
                details: "initializing project directory",
            });
        },
        Err(err) => {
            show_message( term, MessageType::Error, Message {
                action: "Failed",
                details: "to create project directories",
            });
            return Err(err)
        }
    };
    infrastructure::init(term, &igloo_dir)?;
    Ok(())
}

pub fn clean_project(term: &mut CommandTerminal, igloo_dir: &PathBuf) -> Result<(), Error> {
    show_message( term, MessageType::Info, Message {
        action: "Cleaning",
        details: "project directory",
    });
    infrastructure::clean(term, igloo_dir)?;
    show_message(
        term,
        MessageType::Success,
        Message {
            action: "Finished",
            details: "cleaning project directory",
        },
    );
    Ok(())
}

pub fn stop_containers(term: &mut CommandTerminal) -> Result<(), Error> {
    show_message( term, MessageType::Info, Message {
        action: "Stopping",
        details: "local infrastructure",
    });
    infrastructure::spin_down(term)?;
    Ok(())
}

// Starts the file watcher and the webserver
pub async fn start_development_mode(term: &mut CommandTerminal, clickhouse_config: ClickhouseConfig) -> Result<(), Error> {
    show_message( term, MessageType::Success, Message {
        action: "Starting",
        details: "development mode...",
    });

    // TODO: Explore using a RWLock instead of a Mutex to ensure concurrent reads without locks
    let route_table = Arc::new(Mutex::new(HashMap::<PathBuf, RouteMeta>::new()));

    // TODO: When starting the file watcher, we should check the current directory for files that have been 
    // added or removed since the last time the file watcher was started and ensure that the infra reflects 
    // the application state
    watcher::start_file_watcher(term, Arc::clone(&route_table), clickhouse_config)?;
    webserver::start_webserver(term, Arc::clone(&route_table)).await;
    Ok(())
}
use std::collections::HashMap;
use std::sync::Arc;
use std::{io::Error, path::PathBuf};

use tokio::sync::Mutex;

use crate::infrastructure::olap::clickhouse::ClickhouseConfig;
use crate::infrastructure::stream::redpanda::RedpandaConfig;

use self::start::spin_up;
use self::stop::spin_down;

use super::watcher::RouteMeta;
use super::{watcher, local_webserver};
use super::{CommandTerminal, display::show_message, MessageType, Message};

pub mod clean;
pub mod initialize;
pub mod start;
pub mod stop;
pub mod validate;

#[derive(PartialEq)]
enum DebugStatus {
    Debug,
    Silent,
}

pub struct RoutineSuccess<'a> {
    message: Message<'a>,
    message_type: MessageType,
}

// Implement success and info contructors and a new constructor that lets the user choose which type of message to display
impl RoutineSuccess<'_> {
    pub fn new<'a>(message: Message<'a>, message_type: MessageType) -> Self {
        Self { message, message_type }
    }

    pub fn success<'a>(message: Message<'a>) -> Self {
        Self { message, message_type: MessageType::Success }
    }

    pub fn info<'a>(message: Message<'a>) -> Self {
        Self { message, message_type: MessageType::Info }
    }
}


pub struct RoutineFailure<'a> {
    message: Message<'a>,
    message_type: MessageType,
    error: Error,
}
impl RoutineFailure<'_> {
    pub fn new<'a>(message: Message<'a>, error: Error) -> Self {
        Self { message, message_type: MessageType::Error, error }
    }
}


/// Routines are a collection of operations that are run in sequence.
pub trait Routine {
    // Runs the routine and returns a result without displaying any messages
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure>;

    // Runs the routine and displays messages to the user
    fn run_explicit(&self, term: &mut CommandTerminal) -> Result<RoutineSuccess, RoutineFailure> {
        match self.run_silent() {
            Ok(success) => {
                show_message(term, success.message_type, success.message);
                Ok(success)
            },
            Err(failure) => {
                show_message(term, failure.message_type, failure.message);
                Err(failure)
            },  
        }
    }
}

struct RoutineController {
    routines: Vec<Box<dyn Routine>>,
}

impl RoutineController {
    fn new() -> Self {
        Self { routines: vec![] }
    }

    fn add_routine(&mut self, routine: Box<dyn Routine>) {
        self.routines.push(routine);
    }

    fn run_silent_routines(&self, term: &mut CommandTerminal) -> Vec<Result<RoutineSuccess, RoutineFailure>> {
        self.routines.iter().map(|routine| routine.run_silent()).collect()
    }

    fn run_explicit_routines(&self, term: &mut CommandTerminal) -> Vec<Result<RoutineSuccess, RoutineFailure>> {
        self.routines.iter().map(|routine| routine.run_explicit(term)).collect()
    }
}


// Routines run a sequence of operations and give feedback to the user. 
pub fn start_containers(term: &mut CommandTerminal, clickhouse_config: ClickhouseConfig) -> Result<(), Error> {
    show_message( term, MessageType::Info, Message {
        action: "Running",
        details: "infrastructure spin up",
    });
    
    spin_up(term, clickhouse_config)?;
    Ok(())
}



pub fn clean_project(term: &mut CommandTerminal, igloo_dir: &PathBuf) -> Result<(), Error> {
    show_message( term, MessageType::Info, Message {
        action: "Cleaning",
        details: "project directory",
    });
    clean::clean_project(term, igloo_dir)?;
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
    match spin_down(term) {
        Ok(_) => {
            show_message(term, MessageType::Info, Message {
                    action: "Spinning down",
                    details: "igloo cluster",
                },
            );
            Ok(())
        },
        Err(err) => {
            show_message( term, MessageType::Error, Message {
                action: "Failed",
                details: "to stop local infrastructure",
            });
            return Err(err)
        }
    }
}

// Starts the file watcher and the webserver
pub async fn start_development_mode(term: &mut CommandTerminal, clickhouse_config: ClickhouseConfig, redpanda_config: RedpandaConfig) -> Result<(), Error> {
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
    local_webserver::start_webserver(term, Arc::clone(&route_table), redpanda_config).await;
    Ok(())
}
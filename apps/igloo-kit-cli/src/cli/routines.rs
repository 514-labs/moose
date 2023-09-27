use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::{io::Error, path::PathBuf};

use tokio::sync::Mutex;

use crate::infrastructure::olap::clickhouse::ClickhouseConfig;
use crate::infrastructure::stream::redpanda::RedpandaConfig;
use super::watcher::RouteMeta;
use super::{watcher, local_webserver};
use super::{CommandTerminal, display::show_message, MessageType, Message};

pub mod clean;
pub mod initialize;
pub mod start;
pub mod stop;
pub mod validate;



#[derive(Clone)]
pub struct RoutineSuccess {
    message: Message,
    message_type: MessageType,
}

// Implement success and info contructors and a new constructor that lets the user choose which type of message to display
impl RoutineSuccess {
    pub fn new(message: Message, message_type: MessageType) -> Self {
        Self { message, message_type }
    }

    pub fn success(message: Message) -> Self {
        Self { message, message_type: MessageType::Success }
    }

    pub fn info(message: Message) -> Self {
        Self { message, message_type: MessageType::Info }
    }
}


pub struct RoutineFailure {
    message: Message,
    message_type: MessageType,
    error: Error,
}
impl RoutineFailure {
    pub fn new(message: Message, error: Error) -> Self {
        Self { message, message_type: MessageType::Error, error }
    }
}

#[derive(Clone)]
pub enum RunMode {
    Silent,
    Explicit {
        term:  Arc<RwLock<CommandTerminal>>,
    },
}


/// Routines are a collection of operations that are run in sequence.
pub trait Routine {
    fn run(&self, mode: RunMode) -> Result<RoutineSuccess, RoutineFailure> {
        match mode {
            RunMode::Silent => self.run_silent(),
            RunMode::Explicit {term} => self.run_explicit(term),
        }
    }

    // Runs the routine and returns a result without displaying any messages
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure>;

    // Runs the routine and displays messages to the user
    fn run_explicit(&self, term: Arc<RwLock<CommandTerminal>>) -> Result<RoutineSuccess, RoutineFailure> {
        match self.run_silent() {
            Ok(success) => {
                show_message(term, success.message_type.clone(), success.message.clone());
                Ok(success)
            },
            Err(failure) => {
                show_message(term, failure.message_type.clone(), failure.message.clone());
                Err(failure)
            },  
        }
    }
}

pub struct RoutineController {
    routines: Vec<Box<dyn Routine>>,
}

impl RoutineController {
    pub fn new() -> Self {
        Self { routines: vec![] }
    }

    pub fn add_routine(&mut self, routine: Box<dyn Routine>) {
        self.routines.push(routine);
    }

    pub fn run_routines(&self, run_mode: RunMode) -> Vec<Result<RoutineSuccess, RoutineFailure>> {
        self.routines.iter().map(|routine| {
            let run_mode = run_mode.clone();
            routine.run(run_mode)
        }).collect()
    }

    pub fn run_silent_routines(&self) -> Vec<Result<RoutineSuccess, RoutineFailure>> {
        self.routines.iter().map(|routine| routine.run_silent()).collect()
    }

    pub fn run_explicit_routines(&self, term: Arc<RwLock<CommandTerminal>>) -> Vec<Result<RoutineSuccess, RoutineFailure>> {
        self.routines.iter().map(|routine| {
            let term = Arc::clone(&term);
            routine.run_explicit(term)}
        ).collect()
    }
}



// Starts the file watcher and the webserver
pub async fn start_development_mode(clickhouse_config: ClickhouseConfig, redpanda_config: RedpandaConfig) -> Result<(), Error> {
    let term = Arc::new(RwLock::new(CommandTerminal::new()));

    show_message( term.clone(), MessageType::Success, Message {
        action: "Starting".to_string(),
        details: "development mode...".to_string(),
    });

    // TODO: Explore using a RWLock instead of a Mutex to ensure concurrent reads without locks
    let route_table = Arc::new(Mutex::new(HashMap::<PathBuf, RouteMeta>::new()));

    // TODO: When starting the file watcher, we should check the current directory for files that have been 
    // added or removed since the last time the file watcher was started and ensure that the infra reflects 
    // the application state
    watcher::start_file_watcher(term.clone(), Arc::clone(&route_table), clickhouse_config)?;
    local_webserver::start_webserver(term.clone(), Arc::clone(&route_table), redpanda_config).await;
    Ok(())
}
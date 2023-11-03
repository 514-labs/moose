//! # Routines
//! This module is used to define routines that can be run by the CLI. Routines are a collection of operations that are run in 
//! sequence. They can be run silently or explicitly. When run explicitly, they display messages to the user. When run silently,
//! they do not display any messages to the user.
//! 
//! ## Example
//! ```
//! use igloo::cli::routines::{Routine, RoutineSuccess, RoutineFailure, RunMode};
//! use igloo::cli::display::{Message, MessageType};
//! 
//! struct HelloWorldRoutine {}
//! impl HelloWorldRoutine {
//!    pub fn new() -> Self {
//!       Self {}
//!   }
//! }
//! impl Routine for HelloWorldRoutine {
//!   fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
//!      Ok(RoutineSuccess::success(Message::new("Hello".to_string(), "world".to_string())))
//!  }
//! }
//! 
//! let routine_controller = RoutineController::new();
//! routine_controller.add_routine(Box::new(HelloWorldRoutine::new()));
//! let results = routine_controller.run_silent_routines();
//! 
//! assert_eq!(results.len(), 1);
//! assert!(results[0].is_ok());
//! assert_eq!(results[0].as_ref().unwrap().message_type, MessageType::Success);
//! assert_eq!(results[0].as_ref().unwrap().message.action, "Hello");
//! assert_eq!(results[0].as_ref().unwrap().message.details, "world");
//! ```
//! 
//! ## Routine
//! The `Routine` trait defines the interface for a routine. It has three methods:
//! - `run` - This method runs the routine and returns a result. It takes a `RunMode` as an argument. The `RunMode` enum defines
//!  the different ways that a routine can be run. It can be run silently or explicitly. When run explicitly, it displays messages
//! to the user. When run silently, it does not display any messages to the user.
//! - `run_silent` - This method runs the routine and returns a result without displaying any messages to the user.
//! - `run_explicit` - This method runs the routine and displays messages to the user.
//! 
//! ## RoutineSuccess
//! The `RoutineSuccess` struct is used to return a successful result from a routine. It contains a `Message` and a `MessageType`.
//! The `Message` is the message that will be displayed to the user. The `MessageType` is the type of message that will be displayed
//! to the user. The `MessageType` enum defines the different types of messages that can be displayed to the user.
//! 
//! ## RoutineFailure
//! The `RoutineFailure` struct is used to return a failure result from a routine. It contains a `Message`, a `MessageType`, and an
//! `Error`. The `Message` is the message that will be displayed to the user. The `MessageType` is the type of message that will be
//! displayed to the user. The `MessageType` enum defines the different types of messages that can be displayed to the user. The `Error`
//! is the error that caused the routine to fail.
//! 
//! ## RunMode
//! The `RunMode` enum defines the different ways that a routine can be run. It can be run silently or explicitly. When run explicitly,
//! it displays messages to the user. When run silently, it does not display any messages to the user.
//! 
//! ## RoutineController
//! The `RoutineController` struct is used to run a collection of routines. It contains a vector of `Box<dyn Routine>`. It has the
//! following methods:
//! - `new` - This method creates a new `RoutineController`.
//! - `add_routine` - This method adds a routine to the `RoutineController`.
//! - `run_routines` - This method runs all of the routines in the `RoutineController` and returns a vector of results. It takes a
//! `RunMode` as an argument. The `RunMode` enum defines the different ways that a routine can be run. It can be run silently or
//! explicitly. When run explicitly, it displays messages to the user. When run silently, it does not display any messages to the user.
//! - `run_silent_routines` - This method runs all of the routines in the `RoutineController` and returns a vector of results without
//! displaying any messages to the user.
//! - `run_explicit_routines` - This method runs all of the routines in the `RoutineController` and returns a vector of results while
//! displaying messages to the user.
//! 
//! ## Start Development Mode
//! The `start_development_mode` function is used to start the file watcher and the webserver. It takes a `ClickhouseConfig` and a
//! `RedpandaConfig` as arguments. The `ClickhouseConfig` is used to configure the Clickhouse database. The `RedpandaConfig` is used
//! to configure the Redpanda stream processor. This is a special routine due to it's async nature.
//! 
//! ## Suggested Improvements
//! - Explore using a RWLock instead of a Mutex to ensure concurrent reads without locks
//! - Simplify the API for the user when using RunMode::Explicit since it creates lifetime and ownership issues
//! - Enable creating nested routines and cascading down the RunMode to show messages to the user
//! - Organize routines better in the file hiearchy
//! 



use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::{io::Error, path::PathBuf};

use tokio::sync::Mutex;

use crate::infrastructure::olap::clickhouse::config::ClickhouseConfig;
use crate::infrastructure::stream::redpanda::RedpandaConfig;
use crate::project::Project;
use super::local_webserver::{Webserver, LocalWebserverConfig};
use super::watcher::{RouteMeta, FileWatcher};
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
pub async fn start_development_mode(project: Project, clickhouse_config: ClickhouseConfig, redpanda_config: RedpandaConfig, local_webserver_config: LocalWebserverConfig) -> Result<(), Error> {
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
    
    let web_server = Webserver::new(local_webserver_config.host, local_webserver_config.port);
    let file_watcher = FileWatcher::new();
    
    file_watcher.start(project, web_server.clone(), term.clone(), Arc::clone(&route_table), clickhouse_config)?;
    web_server.start(term.clone(), Arc::clone(&route_table), redpanda_config).await;

    Ok(())
}
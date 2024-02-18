//! # Routines
//! This module is used to define routines that can be run by the CLI. Routines are a collection of operations that are run in
//! sequence. They can be run silently or explicitly. When run explicitly, they display messages to the user. When run silently,
//! they do not display any messages to the user.
//!
//! ## Example
//! ```
//! use crate::cli::routines::{Routine, RoutineSuccess, RoutineFailure, RunMode};
//! use crate::cli::display::{Message, MessageType};
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
use std::path::Path;
use std::{io::Error, path::PathBuf};

use log::debug;
use tokio::sync::RwLock;

use super::local_webserver::Webserver;
use super::watcher::FileWatcher;
use super::{Message, MessageType};

use crate::framework::controller::RouteMeta;
use crate::framework::schema::process_schema_file;
use crate::infrastructure::console::post_current_state_to_console;
use crate::infrastructure::olap;
use crate::infrastructure::olap::clickhouse::ConfiguredDBClient;
use crate::infrastructure::stream::redpanda;
use crate::project::Project;
use log::info;

use async_recursion::async_recursion;

pub mod clean;
pub mod initialize;
pub mod start;
pub mod stop;
mod util;
pub mod validate;

#[derive(Clone)]
pub struct RoutineSuccess {
    message: Message,
    message_type: MessageType,
}

// Implement success and info contructors and a new constructor that lets the user choose which type of message to display
impl RoutineSuccess {
    #[allow(dead_code)]
    pub fn info(message: Message) -> Self {
        Self {
            message,
            message_type: MessageType::Info,
        }
    }

    pub fn success(message: Message) -> Self {
        Self {
            message,
            message_type: MessageType::Success,
        }
    }
}

#[derive(Debug)]
pub struct RoutineFailure {
    message: Message,
    message_type: MessageType,
    error: Option<Error>,
}
impl RoutineFailure {
    pub fn new(message: Message, error: Error) -> Self {
        Self {
            message,
            message_type: MessageType::Error,
            error: Some(error),
        }
    }

    /// create a RoutineFailure error without an io error
    pub fn error(message: Message) -> Self {
        Self {
            message,
            message_type: MessageType::Error,
            error: None,
        }
    }
}

#[derive(Clone, Copy)]
pub enum RunMode {
    Explicit,
}

/// Routines are a collection of operations that are run in sequence.
pub trait Routine {
    fn run(&self, mode: RunMode) -> Result<RoutineSuccess, RoutineFailure> {
        match mode {
            RunMode::Explicit => self.run_explicit(),
        }
    }

    // Runs the routine and returns a result without displaying any messages
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure>;

    // Runs the routine and displays messages to the user
    fn run_explicit(&self) -> Result<RoutineSuccess, RoutineFailure> {
        match self.run_silent() {
            Ok(success) => {
                show_message!(success.message_type, success.message.clone());
                Ok(success)
            }
            Err(failure) => {
                show_message!(
                    failure.message_type,
                    Message::new(
                        failure.message.action.clone(),
                        match &failure.error {
                            None => {
                                failure.message.details.clone()
                            }
                            Some(error) => {
                                format!("{}: {}", failure.message.details.clone(), error)
                            }
                        },
                    )
                );
                Err(failure)
            }
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
        self.routines
            .iter()
            .map(|routine| routine.run(run_mode))
            .collect()
    }
}

// Starts the file watcher and the webserver
pub async fn start_development_mode(project: &Project) -> anyhow::Result<()> {
    show_message!(
        MessageType::Success,
        Message {
            action: "Starting".to_string(),
            details: "development mode".to_string(),
        }
    );

    let mut route_table = HashMap::<PathBuf, RouteMeta>::new();

    info!("Initializing project state");
    initialize_project_state(project.schemas_dir(), project, &mut route_table).await?;

    let route_table: &'static RwLock<HashMap<PathBuf, RouteMeta>> =
        Box::leak(Box::new(RwLock::new(route_table)));

    let web_server = Webserver::new(
        project.local_webserver_config.host.clone(),
        project.local_webserver_config.port,
    );
    let file_watcher = FileWatcher::new();

    file_watcher.start(project, route_table)?;

    info!("Starting web server...");

    web_server.start(route_table, project).await;

    Ok(())
}

async fn initialize_project_state(
    schema_dir: PathBuf,
    project: &Project,
    route_table: &mut HashMap<PathBuf, RouteMeta>,
) -> anyhow::Result<()> {
    let configured_client = olap::clickhouse::create_client(project.clickhouse_config.clone());
    let producer = redpanda::create_producer(project.redpanda_config.clone());

    info!("Starting schema directory crawl...");
    let crawl_result =
        process_schemas_in_dir(&schema_dir, project, &configured_client, route_table).await;

    let _ = post_current_state_to_console(
        project,
        &configured_client,
        &producer,
        route_table.clone(),
        project.console_config.clone(),
    )
    .await;

    match crawl_result {
        Ok(_) => {
            info!("Schema directory crawl completed successfully");
            Ok(())
        }
        Err(e) => {
            debug!("Schema directory crawl failed");
            debug!("Error: {:?}", e);
            Err(e)
        }
    }
}

#[async_recursion]
async fn process_schemas_in_dir(
    schema_dir: &Path,
    project: &Project,
    configured_client: &ConfiguredDBClient,
    route_table: &mut HashMap<PathBuf, RouteMeta>,
) -> anyhow::Result<()> {
    if schema_dir.is_dir() {
        for entry in std::fs::read_dir(schema_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                debug!("Processing directory: {:?}", path);
                process_schemas_in_dir(&path, project, configured_client, route_table).await?;
            } else {
                debug!("Processing file: {:?}", path);
                process_schema_file(&path, project, configured_client, route_table).await?
            }
        }
    }
    Ok(())
}

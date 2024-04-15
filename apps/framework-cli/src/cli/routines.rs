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
use std::path::PathBuf;
use std::sync::Arc;

use log::debug;
use log::info;
use tokio::sync::RwLock;

use crate::framework::controller::{
    create_or_replace_version_sync, get_all_framework_objects, process_objects, FrameworkObject,
    FrameworkObjectVersions, RouteMeta, SchemaVersion,
};
use crate::framework::sdks::generate_ts_sdk;
use crate::infrastructure::console::post_current_state_to_console;
use crate::infrastructure::kafka_clickhouse_sync::SyncingProcessesRegistry;
use crate::infrastructure::olap;
use crate::infrastructure::olap::clickhouse::version_sync::get_all_version_syncs;
use crate::infrastructure::stream::redpanda;
use crate::project::{Project, PROJECT};
use crate::utilities::package_managers;

use super::display::{with_spinner, with_spinner_async};
use super::local_webserver::Webserver;
use super::watcher::FileWatcher;
use super::{Message, MessageType};

pub mod clean;
pub mod dev;
pub mod docker_packager;
pub mod flow;
pub mod initialize;
pub mod migrate;
pub mod stop;
pub mod templates;
mod util;
pub mod validate;
pub mod version;

#[derive(Debug, Clone)]
pub struct RoutineSuccess {
    pub message: Message,
    pub message_type: MessageType,
}

// Implement success and info contructors and a new constructor that lets the user choose which type of message to display
impl RoutineSuccess {
    // E.g. when we try to create a resource that already exists,
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
    pub message: Message,
    pub message_type: MessageType,
    pub error: Option<anyhow::Error>,
}
impl RoutineFailure {
    pub fn new<F: Into<anyhow::Error>>(message: Message, error: F) -> Self {
        Self {
            message,
            message_type: MessageType::Error,
            error: Some(error.into()),
        }
    }

    /// create a RoutineFailure error without an error
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
pub async fn start_development_mode(project: Arc<Project>) -> anyhow::Result<()> {
    show_message!(
        MessageType::Success,
        Message {
            action: "Starting".to_string(),
            details: "development mode".to_string(),
        }
    );

    let mut route_table = HashMap::<PathBuf, RouteMeta>::new();

    info!("<DCM> Initializing project state");
    let framework_object_versions =
        initialize_project_state(project.clone(), &mut route_table).await?;

    let route_table: &'static RwLock<HashMap<PathBuf, RouteMeta>> =
        Box::leak(Box::new(RwLock::new(route_table)));

    let mut syncing_processes_registry = SyncingProcessesRegistry::new(
        project.redpanda_config.clone(),
        project.clickhouse_config.clone(),
    );

    syncing_processes_registry.start_all(&framework_object_versions);

    let file_watcher = FileWatcher::new();
    file_watcher.start(
        project.clone(),
        framework_object_versions,
        route_table,
        syncing_processes_registry,
    )?;

    info!("Starting web server...");
    let server_config = project.http_server_config.clone();
    let web_server = Webserver::new(server_config.host.clone(), server_config.port);
    web_server.start(route_table, project).await;

    Ok(())
}

// Starts the webserver in production mode
pub async fn start_production_mode(project: Arc<Project>) -> anyhow::Result<()> {
    show_message!(
        MessageType::Success,
        Message {
            action: "Starting".to_string(),
            details: "production mode".to_string(),
        }
    );

    let mut route_table = HashMap::<PathBuf, RouteMeta>::new();

    info!("<DCM> Initializing project state");
    let framework_object_versions =
        initialize_project_state(project.clone(), &mut route_table).await?;

    let route_table: &'static RwLock<HashMap<PathBuf, RouteMeta>> =
        Box::leak(Box::new(RwLock::new(route_table)));

    let mut syncing_processes_registry = SyncingProcessesRegistry::new(
        project.redpanda_config.clone(),
        project.clickhouse_config.clone(),
    );
    syncing_processes_registry.start_all(&framework_object_versions);

    info!("Starting web server...");
    let server_config = project.http_server_config.clone();
    let web_server = Webserver::new(server_config.host.clone(), server_config.port);
    web_server.start(route_table, project).await;

    Ok(())
}

fn crawl_schema(
    project: &Project,
    old_versions: &[String],
) -> anyhow::Result<FrameworkObjectVersions> {
    info!("<DCM> Checking for old version directories...");

    let mut framework_object_versions =
        FrameworkObjectVersions::new(project.version().to_string(), project.schemas_dir().clone());

    for version in old_versions.iter() {
        let path = project.old_version_location(version)?;

        debug!("<DCM> Processing old version directory: {:?}", path);

        let mut framework_objects = HashMap::new();
        get_all_framework_objects(&mut framework_objects, &path, version)?;

        let schema_version = SchemaVersion {
            base_path: path,
            models: framework_objects,
            typescript_objects: HashMap::new(),
        };

        framework_object_versions
            .previous_version_models
            .insert(version.clone(), schema_version);
    }

    let schema_dir = project.schemas_dir();
    info!("<DCM> Starting schema directory crawl...");
    with_spinner("Processing schema file", || {
        let mut framework_objects: HashMap<String, FrameworkObject> = HashMap::new();
        get_all_framework_objects(&mut framework_objects, &schema_dir, project.version())?;

        framework_object_versions.current_models = SchemaVersion {
            base_path: schema_dir.clone(),
            models: framework_objects.clone(),
            typescript_objects: HashMap::new(),
        };
        anyhow::Ok(())
    })?;

    Ok(framework_object_versions)
}

async fn initialize_project_state(
    project: Arc<Project>,
    route_table: &mut HashMap<PathBuf, RouteMeta>,
) -> anyhow::Result<FrameworkObjectVersions> {
    let old_versions = project.old_versions_sorted();

    let configured_client = olap::clickhouse::create_client(project.clickhouse_config.clone());
    let producer = redpanda::create_producer(project.redpanda_config.clone());

    info!("<DCM> Checking for old version directories...");

    let mut framework_object_versions = crawl_schema(&project, &old_versions)?;

    with_spinner_async("Processing versions", async {
        // TODO: enforce linearity, if 1.1 is linked to 2.0, 1.2 cannot be added
        let mut previous_version: Option<(String, HashMap<String, FrameworkObject>)> = None;
        for version in old_versions {
            let schema_version: &mut SchemaVersion = framework_object_versions
                .previous_version_models
                .get_mut(&version)
                .unwrap();

            process_objects(
                &schema_version.models,
                &previous_version,
                project.clone(),
                &schema_version.base_path,
                &configured_client,
                &mut schema_version.typescript_objects,
                route_table,
                &version,
            )
            .await?;
            previous_version = Some((version, schema_version.models.clone()));
        }

        let result = process_objects(
            &framework_object_versions.current_models.models,
            &previous_version,
            project.clone(),
            &framework_object_versions.current_models.base_path,
            &configured_client,
            &mut framework_object_versions.current_models.typescript_objects,
            route_table,
            &framework_object_versions.current_version,
        )
        .await;

        // TODO: add old versions to SDK
        if !PROJECT.lock().unwrap().is_production {
            let sdk_location = generate_ts_sdk(
                project.clone(),
                &framework_object_versions.current_models.typescript_objects,
            )?;
            let package_manager = package_managers::PackageManager::Npm;
            package_managers::install_packages(&sdk_location, &package_manager)?;
            package_managers::run_build(&sdk_location, &package_manager)?;
            package_managers::link_sdk(&sdk_location, None, &package_manager)?;
        }
        let _ = post_current_state_to_console(
            project.clone(),
            &configured_client,
            &producer,
            &framework_object_versions,
        )
        .await;

        match result {
            Ok(_) => {
                info!("<DCM> Schema directory crawl completed successfully");
                Ok(())
            }
            Err(e) => {
                debug!("<DCM> Schema directory crawl failed");
                debug!("<DCM> Error: {:?}", e);
                Err(e)
            }
        }
    })
    .await?;

    info!("<DCM> Crawling version syncs");
    with_spinner_async::<_, anyhow::Result<()>>("Setting up version syncs", async {
        let version_syncs = get_all_version_syncs(&project, &framework_object_versions)?;
        for vs in version_syncs {
            debug!("<DCM> Creating version sync: {:?}", vs);
            create_or_replace_version_sync(vs, &configured_client).await?;
        }
        Ok(())
    })
    .await?;

    Ok(framework_object_versions)
}

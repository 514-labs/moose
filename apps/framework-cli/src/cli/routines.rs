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

use log::{debug, error, info};
use tokio::sync::RwLock;

use crate::cli::routines::aggregation::start_aggregation_process;
use crate::cli::routines::consumption::start_consumption_process;

use crate::cli::watcher::process_flows_changes;
use crate::framework::flows::registry::FlowProcessRegistry;
use crate::infrastructure::olap::clickhouse::{
    fetch_table_names, fetch_table_schema, table_schema_to_hash,
};

use crate::cli::routines::flow::verify_flows_against_datamodels;
use crate::framework::controller::{
    create_or_replace_version_sync, get_all_framework_objects, process_objects, FrameworkObject,
    FrameworkObjectVersions, RouteMeta, SchemaVersion,
};
use crate::framework::typescript;
use crate::infrastructure::console::post_current_state_to_console;
use crate::infrastructure::kafka_clickhouse_sync::SyncingProcessesRegistry;
use crate::infrastructure::olap;
use crate::infrastructure::olap::clickhouse::version_sync::{get_all_version_syncs, VersionSync};
use crate::infrastructure::olap::clickhouse_alt_client::{get_pool, store_current_state};
use crate::infrastructure::stream::redpanda;
use crate::project::{AggregationSet, Project};
use crate::utilities::package_managers;

use super::display::with_spinner_async;
use super::local_webserver::Webserver;
use super::watcher::FileWatcher;
use super::{Message, MessageType};

pub mod aggregation;
pub mod clean;
pub mod consumption;
pub mod dev;
pub mod docker_packager;
pub mod flow;
pub mod initialize;
pub mod logs;
pub mod migrate;
pub mod stop;
pub mod templates;
mod util;
pub mod validate;
pub mod version;

#[derive(Debug, Clone)]
#[must_use = "The message should be displayed."]
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

    pub fn highlight(message: Message) -> Self {
        Self {
            message,
            message_type: MessageType::Highlight,
        }
    }

    pub fn show(&self) {
        show_message!(self.message_type, self.message);
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
        MessageType::Info,
        Message {
            action: "Starting".to_string(),
            details: "development mode".to_string(),
        }
    );

    let mut route_table = HashMap::<PathBuf, RouteMeta>::new();

    info!("<DCM> Initializing project state");
    let (framework_object_versions, version_syncs) =
        initialize_project_state(project.clone(), &mut route_table).await?;

    {
        let mut client = get_pool(&project.clickhouse_config).get_handle().await?;
        let aggregations = project.get_aggregations();
        store_current_state(
            &mut client,
            &framework_object_versions,
            &aggregations,
            &project.clickhouse_config,
        )
        .await?
    }

    let route_table: &'static RwLock<HashMap<PathBuf, RouteMeta>> =
        Box::leak(Box::new(RwLock::new(route_table)));

    let mut syncing_processes_registry = SyncingProcessesRegistry::new(
        project.redpanda_config.clone(),
        project.clickhouse_config.clone(),
    );

    syncing_processes_registry.start_all(&framework_object_versions, &version_syncs);

    let mut flows_process_registry = FlowProcessRegistry::new(project.redpanda_config.clone());
    // Once the below function is optimized to act on events, this
    // will need to get refactored out.
    process_flows_changes(&project, &mut flows_process_registry).await?;

    let file_watcher = FileWatcher::new();
    file_watcher.start(
        project.clone(),
        framework_object_versions,
        route_table,
        syncing_processes_registry,
        flows_process_registry,
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
    let (framework_object_versions, version_syncs) =
        initialize_project_state(project.clone(), &mut route_table).await?;

    debug!("Route table: {:?}", route_table);

    let route_table: &'static RwLock<HashMap<PathBuf, RouteMeta>> =
        Box::leak(Box::new(RwLock::new(route_table)));

    let mut syncing_processes_registry = SyncingProcessesRegistry::new(
        project.redpanda_config.clone(),
        project.clickhouse_config.clone(),
    );
    syncing_processes_registry.start_all(&framework_object_versions, &version_syncs);

    let mut flows_process_registry = FlowProcessRegistry::new(project.redpanda_config.clone());
    // Once the below function is optimized to act on events, this
    // will need to get refactored out.
    process_flows_changes(&project, &mut flows_process_registry).await?;
    start_aggregation_process(&project)?;
    start_consumption_process(&project)?;

    info!("Starting web server...");
    let server_config = project.http_server_config.clone();
    let web_server = Webserver::new(server_config.host.clone(), server_config.port);
    web_server.start(route_table, project).await;

    Ok(())
}

async fn crawl_schema(
    project: &Project,
    old_versions: &[String],
) -> anyhow::Result<FrameworkObjectVersions> {
    info!("<DCM> Checking for old version directories...");

    let mut framework_object_versions =
        FrameworkObjectVersions::new(project.version().to_string(), project.schemas_dir().clone());

    let aggregations = AggregationSet {
        current_version: project.version().to_owned(),
        names: project.get_aggregations(),
    };

    for version in old_versions.iter() {
        let path = project.old_version_location(version)?;

        debug!("<DCM> Processing old version directory: {:?}", path);

        let mut framework_objects = HashMap::new();
        get_all_framework_objects(&mut framework_objects, &path, version, &aggregations).await?;

        let schema_version = SchemaVersion {
            base_path: path,
            models: framework_objects,
        };

        framework_object_versions
            .previous_version_models
            .insert(version.clone(), schema_version);
    }

    let schema_dir = project.schemas_dir();

    let aggregations = AggregationSet {
        current_version: project.version().to_owned(),
        names: project.get_aggregations(),
    };

    info!("<DCM> Starting schema directory crawl...");
    let mut framework_objects: HashMap<String, FrameworkObject> = HashMap::new();
    get_all_framework_objects(
        &mut framework_objects,
        &schema_dir,
        project.version(),
        &aggregations,
    )
    .await?;

    framework_object_versions.current_models = SchemaVersion {
        base_path: schema_dir.clone(),
        models: framework_objects.clone(),
    };

    Ok(framework_object_versions)
}

async fn check_for_model_changes(
    project: Arc<Project>,
    framework_object_versions: FrameworkObjectVersions,
) {
    let configured_client = olap::clickhouse::create_client(project.clickhouse_config.clone());

    let mut current_data_models = HashMap::new();
    for fo in framework_object_versions.current_models.models.values() {
        let mut data_elements = vec![];
        for column in &fo.data_model.columns {
            data_elements.push((column.name.clone(), column.data_type.to_string()));
        }
        data_elements.sort();
        // Comments below left in for testing and debugging purposes
        // println!("  current_schema: {:?}", data_elements.clone());
        let hash_val = table_schema_to_hash(data_elements).unwrap();
        // println!(
        //     "current_data_models: {:?}-{:?}",
        //     fo.data_model.name.clone(),
        //     hash_val.clone()
        // );
        current_data_models.insert(fo.data_model.name.clone(), hash_val.clone());
    }

    // println!("");

    let mut prev_data_models = HashMap::new();
    for sv in framework_object_versions.previous_version_models.values() {
        for fo in sv.models.values() {
            let mut data_elements = vec![];
            for column in &fo.data_model.columns {
                data_elements.push((column.name.clone(), column.data_type.to_string()));
            }
            data_elements.sort();
            // Comments below left in for testing and debugging purposes
            // println!("  prev_schema: {:?}", data_elements.clone());
            let hash_val = table_schema_to_hash(data_elements).unwrap();
            // println!(
            //     "prev_data_models: {:?}-{:?}",
            //     fo.data_model.name.clone(),
            //     hash_val.clone()
            // );
            prev_data_models.insert(fo.data_model.name.clone(), hash_val.clone());
        }
    }

    olap::clickhouse::check_ready(&configured_client)
        .await
        .unwrap();

    let mut db_data_models = HashMap::new();
    let tables_result = fetch_table_names(&configured_client).await;
    match tables_result {
        Ok(tables) => {
            for table in &tables {
                let table_schema_result = fetch_table_schema(&configured_client, table).await;
                match table_schema_result {
                    Ok(table_schema) => {
                        let hash_result = table_schema_to_hash(table_schema.clone());
                        match hash_result {
                            Ok(hash) => {
                                // println!("  table_schema: {:?}", table_schema.clone());
                                // println!("db_data_models: {:?}-{:?}", table.clone(), hash.clone());
                                db_data_models.insert(table.clone(), hash.clone());
                            }
                            Err(e) => {
                                error!("Failed to hash table schema for table {}: {}", table, e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to fetch table schema for table {}: {}", table, e);
                    }
                }
            }
        }
        Err(e) => {
            error!("Failed to fetch table names: {}", e);
        }
    }
}

async fn initialize_project_state(
    project: Arc<Project>,
    route_table: &mut HashMap<PathBuf, RouteMeta>,
) -> anyhow::Result<(FrameworkObjectVersions, Vec<VersionSync>)> {
    let old_versions = project.old_versions_sorted();

    let configured_client = olap::clickhouse::create_client(project.clickhouse_config.clone());
    let producer = redpanda::create_producer(project.redpanda_config.clone());

    info!("<DCM> Checking for old version directories...");

    let mut framework_object_versions = crawl_schema(&project, &old_versions).await?;

    check_for_model_changes(project.clone(), framework_object_versions.clone()).await;

    with_spinner_async(
        "Processing versions",
        async {
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
                route_table,
                &framework_object_versions.current_version,
            )
            .await;

            // TODO: add old versions to SDK
            if !project.is_production {
                let sdk_location =
                    typescript::generator::generate_sdk(&project, &framework_object_versions)?;
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
        },
        !project.is_production,
    )
    .await?;

    info!("<DCM> Crawling version syncs");
    let version_syncs = with_spinner_async::<_, anyhow::Result<Vec<VersionSync>>>(
        "Setting up version syncs",
        async {
            let version_syncs = get_all_version_syncs(&project, &framework_object_versions)?;
            for vs in &version_syncs {
                debug!("<DCM> Creating version sync: {:?}", vs);
                create_or_replace_version_sync(&project, vs, &configured_client).await?;
            }
            Ok(version_syncs)
        },
        !project.is_production,
    )
    .await?;

    let _ = verify_flows_against_datamodels(&project, &framework_object_versions);

    Ok((framework_object_versions, version_syncs))
}

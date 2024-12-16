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
//!   the different ways that a routine can be run. It can be run silently or explicitly. When run explicitly, it displays messages
//!   to the user. When run silently, it does not display any messages to the user.
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
//!   `RunMode` as an argument. The `RunMode` enum defines the different ways that a routine can be run. It can be run silently or
//!   explicitly. When run explicitly, it displays messages to the user. When run silently, it does not display any messages to the user.
//! - `run_silent_routines` - This method runs all of the routines in the `RoutineController` and returns a vector of results without
//!   displaying any messages to the user.
//! - `run_explicit_routines` - This method runs all of the routines in the `RoutineController` and returns a vector of results while
//!   displaying messages to the user.
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

use crate::framework::core::plan_validator;
use crate::infrastructure::redis::redis_client::RedisClient;
use log::{debug, error, info};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::time::{interval, Duration};

use crate::framework::core::execute::execute_initial_infra_change;
use crate::framework::core::plan::plan_changes;

use crate::framework::controller::RouteMeta;
use crate::framework::core::infrastructure_map::InfrastructureMap;
use crate::infrastructure::olap::clickhouse_alt_client::{get_pool, store_infrastructure_map};
use crate::infrastructure::processes::cron_registry::CronRegistry;
use crate::infrastructure::processes::kafka_clickhouse_sync::clickhouse_writing_pause_button;
use crate::project::Project;

use super::super::metrics::Metrics;
use super::display;
use super::local_webserver::Webserver;
use super::watcher::FileWatcher;
use super::{Message, MessageType};

pub mod auth;
pub mod block;
pub mod clean;
pub mod consumption;
pub mod datamodel;
pub mod dev;
pub mod docker_packager;
pub mod initialize;
pub mod logs;
pub mod ls;
pub mod metrics_console;
pub mod migrate;
pub mod peek;
pub mod ps;
pub mod streaming;
pub mod templates;
mod util;
pub mod validate;
pub mod version;

const LEADERSHIP_LOCK_RENEWAL_INTERVAL: u64 = 5; // 5 seconds
const LEADERSHIP_LOCK_TTL: u64 = LEADERSHIP_LOCK_RENEWAL_INTERVAL * 3; // best practice to set lock expiration to 2-3x the renewal interval

// Static flag to track if leadership tasks are running
static IS_RUNNING_LEADERSHIP_TASKS: AtomicBool = AtomicBool::new(false);

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

pub async fn setup_redis_client(project: Arc<Project>) -> anyhow::Result<Arc<Mutex<RedisClient>>> {
    let redis_client = RedisClient::new(project.name(), project.redis_config.clone()).await?;
    let redis_client = Arc::new(Mutex::new(redis_client));

    let (service_name, instance_id) = {
        let client = redis_client.lock().await;
        (
            client.get_service_name().to_string(),
            client.get_instance_id().to_string(),
        )
    };

    show_message!(
        MessageType::Info,
        Message {
            action: "Node Id:".to_string(),
            details: format!("{}::{}", service_name, instance_id),
        }
    );

    // Register the leadership lock
    redis_client
        .lock()
        .await
        .register_lock("leadership", LEADERSHIP_LOCK_TTL as i64)
        .await?;

    let redis_client_clone = redis_client.clone();
    let callback = Arc::new(move |message: String| {
        let redis_client = redis_client_clone.clone();
        tokio::spawn(async move {
            if let Err(e) = process_pubsub_message(message, redis_client).await {
                error!("Error processing pubsub message: {}", e);
            }
        });
    });

    // Start the leadership lock management task
    start_leadership_lock_task(redis_client.clone(), project.clone());

    redis_client
        .lock()
        .await
        .register_message_handler(callback)
        .await;
    redis_client.lock().await.start_periodic_tasks();

    Ok(redis_client)
}

async fn process_pubsub_message(
    message: String,
    redis_client: Arc<Mutex<RedisClient>>,
) -> anyhow::Result<()> {
    let has_lock = {
        let client = redis_client.lock().await;
        client.has_lock("leadership").await?
    };

    if has_lock {
        if message.contains("<migration_start>") {
            info!("<Routines> This instance is the leader so ignoring the Migration start message: {}", message);
        } else if message.contains("<migration_end>") {
            info!("<Routines> This instance is the leader so ignoring the Migration end message received: {}", message);
        } else {
            info!(
                "<Routines> This instance is the leader and received pubsub message: {}",
                message
            );
        }
    } else {
        // this assumes that the leader is not doing inserts during migration
        if message.contains("<migration_start>") {
            clickhouse_writing_pause_button().send(true)?;
            info!("Pausing write to CH");
        } else if message.contains("<migration_end>") {
            clickhouse_writing_pause_button().send(false)?;
            info!("Resuming write to CH");
        } else {
            info!(
                "<Routines> This instance is not the leader and received pubsub message: {}",
                message
            );
        }
    }
    Ok(())
}

fn start_leadership_lock_task(redis_client: Arc<Mutex<RedisClient>>, project: Arc<Project>) {
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(LEADERSHIP_LOCK_RENEWAL_INTERVAL)); // Adjust the interval as needed
        loop {
            interval.tick().await;
            if let Err(e) = manage_leadership_lock(&redis_client, &project).await {
                error!("<RedisClient> Error managing leadership lock: {:#}", e);
            }
        }
    });
}

async fn manage_leadership_lock(
    redis_client: &Arc<Mutex<RedisClient>>,
    project: &Arc<Project>,
) -> Result<(), anyhow::Error> {
    let (has_lock, is_new_acquisition) = {
        let client = redis_client.lock().await;
        client.check_and_renew_lock("leadership").await?
    };

    if has_lock && is_new_acquisition {
        info!("<RedisClient> Obtained leadership lock, performing leadership tasks");

        let project_clone = project.clone();

        tokio::spawn(async move {
            if let Err(e) = leadership_tasks(project_clone).await {
                error!("<RedisClient> Error executing leadership tasks: {}", e);
            }
        });

        let mut client = redis_client.lock().await;
        if let Err(e) = client.broadcast_message("leader.new").await {
            error!(
                "<RedisClient> Failed to broadcast new leader message: {}",
                e
            );
        }
    } else {
        if IS_RUNNING_LEADERSHIP_TASKS.load(Ordering::SeqCst) {
            IS_RUNNING_LEADERSHIP_TASKS.store(false, Ordering::SeqCst);
        }
    }
    Ok(())
}

async fn leadership_tasks(project: Arc<Project>) -> Result<(), anyhow::Error> {
    let cron_registry = CronRegistry::new().await?;
    cron_registry.register_jobs(&project).await?;
    cron_registry.start().await?;
    Ok(())
}

// Starts the file watcher and the webserver
pub async fn start_development_mode(
    project: Arc<Project>,
    metrics: Arc<Metrics>,
    redis_client: Arc<Mutex<RedisClient>>,
) -> anyhow::Result<()> {
    show_message!(
        MessageType::Info,
        Message {
            action: "Starting".to_string(),
            details: "development mode".to_string(),
        }
    );

    let server_config = project.http_server_config.clone();
    let web_server = Webserver::new(
        server_config.host.clone(),
        server_config.port,
        server_config.management_port,
    );

    let consumption_apis: &'static RwLock<HashSet<String>> =
        Box::leak(Box::new(RwLock::new(HashSet::new())));

    let route_table = HashMap::<PathBuf, RouteMeta>::new();
    let route_table: &'static RwLock<HashMap<PathBuf, RouteMeta>> =
        Box::leak(Box::new(RwLock::new(route_table)));

    let route_update_channel = web_server
        .spawn_api_update_listener(route_table, consumption_apis)
        .await;

    let mut client = get_pool(&project.clickhouse_config).get_handle().await?;

    let plan = plan_changes(&mut client, &project).await?;
    info!("Plan Changes: {:?}", plan.changes);

    plan_validator::validate(&plan)?;

    let api_changes_channel = web_server
        .spawn_api_update_listener(route_table, consumption_apis)
        .await;

    let (syncing_registry, process_registry) = execute_initial_infra_change(
        &project,
        &plan,
        api_changes_channel,
        metrics.clone(),
        &mut client,
        &redis_client,
    )
    .await?;
    // TODO - need to add a lock on the table to prevent concurrent updates as migrations are going through.

    // Storing the result of the changes in the table
    store_infrastructure_map(
        &mut client,
        &project.clickhouse_config,
        &plan.target_infra_map,
    )
    .await?;

    plan.target_infra_map
        .store_in_redis(&*redis_client.lock().await)
        .await?;

    let infra_map: &'static RwLock<InfrastructureMap> =
        Box::leak(Box::new(RwLock::new(plan.target_infra_map)));

    let file_watcher = FileWatcher::new();
    file_watcher.start(
        project.clone(),
        route_update_channel,
        infra_map,
        syncing_registry,
        process_registry,
        metrics.clone(),
        redis_client.clone(),
    )?;

    info!("Starting web server...");
    web_server
        .start(route_table, consumption_apis, infra_map, project, metrics)
        .await;

    {
        let mut redis_client = redis_client.lock().await;
        let _ = redis_client.stop_periodic_tasks();
    }

    Ok(())
}

// Starts the webserver in production mode
pub async fn start_production_mode(
    project: Arc<Project>,
    metrics: Arc<Metrics>,
    redis_client: Arc<Mutex<RedisClient>>,
) -> anyhow::Result<()> {
    show_message!(
        MessageType::Success,
        Message {
            action: "Starting".to_string(),
            details: "production mode".to_string(),
        }
    );

    if std::env::var("MOOSE_TEST__CRASH").is_ok() {
        panic!("Crashing for testing purposes");
    }

    let server_config = project.http_server_config.clone();
    info!("Server config: {:?}", server_config);
    let web_server = Webserver::new(
        server_config.host.clone(),
        server_config.port,
        server_config.management_port,
    );
    info!("Web server initialized");

    let consumption_apis: &'static RwLock<HashSet<String>> =
        Box::leak(Box::new(RwLock::new(HashSet::new())));
    info!("Consumption APIs initialized");

    let route_table = HashMap::<PathBuf, RouteMeta>::new();

    debug!("Route table: {:?}", route_table);
    let route_table: &'static RwLock<HashMap<PathBuf, RouteMeta>> =
        Box::leak(Box::new(RwLock::new(route_table)));

    let mut client = get_pool(&project.clickhouse_config).get_handle().await?;
    info!("Clickhouse client initialized");

    let plan = plan_changes(&mut client, &project).await?;
    info!("Plan Changes: {:?}", plan.changes);

    plan_validator::validate(&plan)?;

    let api_changes_channel = web_server
        .spawn_api_update_listener(route_table, consumption_apis)
        .await;

    execute_initial_infra_change(
        &project,
        &plan,
        api_changes_channel,
        metrics.clone(),
        &mut client,
        &redis_client,
    )
    .await?;
    // TODO - need to add a lock on the table to prevent concurrent updates as migrations are going through.

    // Storing the result of the changes in the table
    store_infrastructure_map(
        &mut client,
        &project.clickhouse_config,
        &plan.target_infra_map,
    )
    .await?;

    plan.target_infra_map
        .store_in_redis(&*redis_client.lock().await)
        .await?;

    let infra_map: &'static InfrastructureMap = Box::leak(Box::new(plan.target_infra_map));

    web_server
        .start(route_table, consumption_apis, infra_map, project, metrics)
        .await;

    {
        let mut redis_client = redis_client.lock().await;
        let _ = redis_client.stop_periodic_tasks();
    }

    Ok(())
}

pub async fn plan(project: &Project) -> anyhow::Result<()> {
    let mut client = get_pool(&project.clickhouse_config).get_handle().await?;

    let plan_results = plan_changes(&mut client, project).await?;

    display::show_changes(&plan_results);

    if plan_results.changes.is_empty() {
        show_message!(
            MessageType::Info,
            Message {
                action: "No".to_string(),
                details: "changes detected".to_string(),
            }
        );
    }

    Ok(())
}

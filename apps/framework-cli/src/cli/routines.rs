//! # Routines [Deprecation warning]
//!
//! *****
//! Routines that get run by a CLI should simply be a function that returns a routine success or routine failure. Do not use
//! the Routine and Routine controller structs and traits
//! *****
//!
//!
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
use log::{debug, error, info, warn};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::time::{interval, sleep, Duration};

use crate::cli::routines::openapi::openapi;
use crate::framework::controller::RouteMeta;
use crate::framework::core::execute::execute_initial_infra_change;
use crate::framework::core::infrastructure_map::InfrastructureMap;
use crate::infrastructure::olap::clickhouse_alt_client::{get_pool, store_infrastructure_map};
use crate::infrastructure::processes::cron_registry::CronRegistry;
use crate::infrastructure::processes::kafka_clickhouse_sync::clickhouse_writing_pause_button;
use crate::project::Project;

use super::super::metrics::Metrics;
use super::display;
use super::local_webserver::{PlanRequest, PlanResponse, Webserver};
use super::settings::Settings;
use super::watcher::FileWatcher;
use super::{Message, MessageType};

use crate::framework::core::plan::plan_changes;
use crate::framework::core::plan::InfraPlan;
use crate::framework::core::primitive_map::PrimitiveMap;

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
pub mod openapi;
pub mod peek;
pub mod ps;
pub mod scripts;
pub mod streaming;
pub mod templates;
mod util;
pub mod validate;

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
        display::show_message_wrapper(self.message_type, self.message.clone());
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

pub async fn setup_redis_client(project: Arc<Project>) -> anyhow::Result<Arc<Mutex<RedisClient>>> {
    let redis_client = RedisClient::new(project.name(), project.redis_config.clone()).await?;
    let redis_client = Arc::new(Mutex::new(redis_client));

    if redis_client.lock().await.is_connected() {
        spawn_connection_monitor(redis_client.clone());
    } else {
        warn!("Redis connection is not available. Some features will be limited.");
    }

    let (service_name, instance_id) = {
        let client = redis_client.lock().await;
        (
            client.get_service_name().to_string(),
            client.get_instance_id().to_string(),
        )
    };

    display::show_message_wrapper(
        MessageType::Info,
        Message {
            action: "Node Id:".to_string(),
            details: format!("{}::{}", service_name, instance_id),
        },
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
                error!("<RedisClient> Error processing pubsub message: {}", e);
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

        let cron_registry = CronRegistry::new().await.unwrap();

        loop {
            interval.tick().await;
            if let Err(e) = manage_leadership_lock(&redis_client, &project, &cron_registry).await {
                error!("<RedisClient> Error managing leadership lock: {:#}", e);
            }
        }
    });
}

async fn manage_leadership_lock(
    redis_client: &Arc<Mutex<RedisClient>>,
    project: &Arc<Project>,
    cron_registry: &CronRegistry,
) -> Result<(), anyhow::Error> {
    let (has_lock, is_new_acquisition) = {
        let client = redis_client.lock().await;
        client.check_and_renew_lock("leadership").await?
    };

    if has_lock && is_new_acquisition {
        info!("<RedisClient> Obtained leadership lock, performing leadership tasks");

        IS_RUNNING_LEADERSHIP_TASKS.store(true, Ordering::SeqCst);

        let project_clone = project.clone();
        let cron_registry: CronRegistry = cron_registry.clone();
        tokio::spawn(async move {
            let result = leadership_tasks(project_clone, cron_registry).await;
            if let Err(e) = result {
                error!("<RedisClient> Error executing leadership tasks: {}", e);
            }
            IS_RUNNING_LEADERSHIP_TASKS.store(false, Ordering::SeqCst);
        });

        let mut client = redis_client.lock().await;
        if let Err(e) = client.broadcast_message("leader.new").await {
            error!(
                "<RedisClient> Failed to broadcast new leader message: {}",
                e
            );
        }
    } else if IS_RUNNING_LEADERSHIP_TASKS.load(Ordering::SeqCst) {
        if let Err(e) = cron_registry.stop().await {
            error!("<RedisClient> Failed to stop CronRegistry: {}", e);
        }
        // Then mark leadership tasks as not running
        IS_RUNNING_LEADERSHIP_TASKS.store(false, Ordering::SeqCst);
    }
    Ok(())
}

async fn leadership_tasks(
    project: Arc<Project>,
    cron_registry: CronRegistry,
) -> Result<(), anyhow::Error> {
    cron_registry.register_jobs(&project).await?;
    cron_registry.start().await?;
    Ok(())
}

/// Starts the application in development mode.
/// This mode is optimized for development workflows and includes additional debugging features.
///
/// # Arguments
/// * `project` - Arc wrapped Project instance containing configuration
/// * `metrics` - Arc wrapped Metrics instance for monitoring
/// * `redis_client` - Arc and Mutex wrapped RedisClient for caching
/// * `settings` - Reference to application Settings
///
/// # Returns
/// * `anyhow::Result<()>` - Success or error result
pub async fn start_development_mode(
    project: Arc<Project>,
    metrics: Arc<Metrics>,
    redis_client: Arc<Mutex<RedisClient>>,
    settings: &Settings,
) -> anyhow::Result<()> {
    display::show_message_wrapper(
        MessageType::Info,
        Message {
            action: "Starting".to_string(),
            details: "development mode".to_string(),
        },
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
        settings,
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

    let openapi_file = openapi(&project, &plan.target_infra_map).await?;

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
        .start(
            settings,
            route_table,
            consumption_apis,
            infra_map,
            project,
            metrics,
            Some(openapi_file),
        )
        .await;

    {
        let mut redis_client = redis_client.lock().await;
        let _ = redis_client.stop_periodic_tasks();
    }

    Ok(())
}

/// Starts the application in production mode.
/// This mode is optimized for production use with appropriate security and performance settings.
///
/// # Arguments
/// * `settings` - Reference to application Settings
/// * `project` - Arc wrapped Project instance containing configuration
/// * `metrics` - Arc wrapped Metrics instance for monitoring
/// * `redis_client` - Arc and Mutex wrapped RedisClient for caching
///
/// # Returns
/// * `anyhow::Result<()>` - Success or error result
pub async fn start_production_mode(
    settings: &Settings,
    project: Arc<Project>,
    metrics: Arc<Metrics>,
    redis_client: Arc<Mutex<RedisClient>>,
) -> anyhow::Result<()> {
    display::show_message_wrapper(
        MessageType::Success,
        Message {
            action: "Starting".to_string(),
            details: "production mode".to_string(),
        },
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
        settings,
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
        .start(
            settings,
            route_table,
            consumption_apis,
            infra_map,
            project,
            metrics,
            None,
        )
        .await;

    {
        let mut redis_client = redis_client.lock().await;
        let _ = redis_client.stop_periodic_tasks();
    }

    Ok(())
}

/// Authentication for remote plan requests:
///
/// When making requests to a remote Moose instance, authentication is required for admin operations.
/// The authentication token is sent as a Bearer token in the Authorization header.
///
/// The token is determined in the following order of precedence:
/// 1. Command line parameter: `--token <value>`
/// 2. Environment variable: `MOOSE_ADMIN_TOKEN`
/// 3. Project configuration: `authentication.admin_api_key` in moose.yaml
///
/// Note that the admin_api_key in the project configuration is typically stored in hashed form,
/// so options 1 or 2 are recommended for remote plan operations.
///
/// Simulates a plan command against a remote Moose instance
///
/// # Arguments
/// * `project` - Reference to the project
/// * `url` - Optional URL of the remote Moose instance (default: http://localhost:4000)
/// * `token` - Optional API token for authentication (overrides MOOSE_ADMIN_TOKEN env var)
///
/// # Returns
/// * Result indicating success or failure
pub async fn remote_plan(
    project: &Project,
    url: &Option<String>,
    token: &Option<String>,
) -> anyhow::Result<()> {
    // Build the inframap from the local project
    let primitive_map = PrimitiveMap::load(project).await?;
    let local_infra_map = InfrastructureMap::new(project, primitive_map);

    // Determine the target URL
    let target_url = match url {
        Some(u) => format!("{}/admin/plan", u.trim_end_matches('/')),
        None => "http://localhost:4000/admin/plan".to_string(),
    };

    display::show_message_wrapper(
        MessageType::Info,
        Message {
            action: "Remote Plan".to_string(),
            details: format!(
                "Comparing local project code with remote instance at {}",
                target_url
            ),
        },
    );

    let request_body = PlanRequest {
        infra_map: local_infra_map,
    };

    // Get authentication token - prioritize command line parameter, then env var, then project config
    let auth_token = token
        .clone()
        .or_else(|| std::env::var("MOOSE_ADMIN_TOKEN").ok())
        .ok_or_else(|| anyhow::anyhow!("Authentication token required. Please provide token via --token parameter or MOOSE_ADMIN_TOKEN environment variable"))?;

    // Create HTTP client
    let client = reqwest::Client::new();
    let mut request_builder = client
        .post(&target_url)
        .header("Content-Type", "application/json")
        .json(&request_body);

    // Add authorization header if token is available
    request_builder = request_builder.header("Authorization", format!("Bearer {}", auth_token));

    // Send request
    let response = request_builder.send().await?;

    // Check response status
    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow::anyhow!(
            "Failed to get plan from remote instance: {}",
            error_text
        ));
    }

    // Parse response
    let plan_response: PlanResponse = response.json().await?;

    // Display results
    display::show_message_wrapper(
        MessageType::Success,
        Message {
            action: "Remote Plan".to_string(),
            details: "Retrieved plan from remote instance".to_string(),
        },
    );

    // Check if there are any changes to display
    if plan_response.changes.is_empty() {
        display::show_message_wrapper(
            MessageType::Info,
            Message {
                action: "No Changes".to_string(),
                details: "No changes detected".to_string(),
            },
        );
        return Ok(());
    }

    // Create a temporary InfraPlan to use with the show_changes function
    let temp_plan = InfraPlan {
        changes: plan_response.changes,
        target_infra_map: InfrastructureMap::new(project, PrimitiveMap::default()),
    };

    // Use the existing display method to show the changes
    display::show_changes(&temp_plan);

    Ok(())
}

fn spawn_connection_monitor(redis_client: Arc<Mutex<RedisClient>>) {
    tokio::spawn(async move {
        loop {
            let handle = {
                let client = redis_client.lock().await;
                client.start_connection_monitor()
            };

            // Handle the result directly without using catch_unwind
            let result = handle.await;

            match result {
                Ok(()) => {
                    info!("Connection monitor exited gracefully");
                    // Even if it exited gracefully, we should restart it
                    warn!("Restarting connection monitor in 1 second...");
                    sleep(Duration::from_secs(1)).await;
                }
                Err(err) => {
                    error!(
                        "Connection monitor panicked: {:#?}. Restarting in 1 second...",
                        err
                    );
                    sleep(Duration::from_secs(1)).await;
                }
            }

            // Update the connection state to false since we're restarting
            let client = redis_client.lock().await;
            if client.is_connected() {
                warn!("Setting Redis connection state to disconnected during monitor restart");
                client.set_connection_state(false);
            }
        }
    });
}

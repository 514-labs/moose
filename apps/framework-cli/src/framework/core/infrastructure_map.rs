//! # Infrastructure Map Module
//!
//! This module is a cornerstone of the Moose framework, providing a comprehensive representation
//! of all infrastructure components and their relationships. It serves as the source of truth for
//! the entire system architecture and enables infrastructure-as-code capabilities.
//!
//! ## Overview
//!
//! The `InfrastructureMap` tracks all components of the system:
//! - Data storage components (tables, views)
//! - Streaming components (topics)
//! - API components (endpoints)
//! - Process components (synchronization processes, function processes, etc.)
//!
//! This map enables the framework to:
//! 1. Generate a complete representation of the required infrastructure
//! 2. Compare infrastructure states to determine necessary changes
//! 3. Apply changes to actual infrastructure components
//! 4. Persist infrastructure state for later reference
//!
//! ## Key Components
//!
//! - `InfrastructureMap`: The main struct containing all infrastructure components
//! - `InfraChanges`: Represents changes between two infrastructure states
//! - Change enums: Various enums representing specific types of changes to components
//!
//! ## Usage Flow
//!
//! 1. Generate an `InfrastructureMap` from primitive components
//! 2. Compare maps to determine changes using `diff()`
//! 3. Apply changes to actual infrastructure
//! 4. Store the updated map for future reference
//!
//! This module is essential for maintaining consistency between the defined infrastructure
//! and the actual deployed components.
use super::infrastructure::api_endpoint::{APIType, ApiEndpoint, Method};
use super::infrastructure::consumption_webserver::ConsumptionApiWebServer;
use super::infrastructure::function_process::FunctionProcess;
use super::infrastructure::olap_process::OlapProcess;
use super::infrastructure::orchestration_worker::OrchestrationWorker;
use super::infrastructure::table::{Column, Table};
use super::infrastructure::topic::{Topic, DEFAULT_MAX_MESSAGE_BYTES};
use super::infrastructure::topic_sync_process::{TopicToTableSyncProcess, TopicToTopicSyncProcess};
use super::infrastructure::view::View;
use super::primitive_map::PrimitiveMap;
use crate::cli::display::{show_message_wrapper, Message, MessageType};
use crate::framework::data_model::config::EndpointIngestionFormat;
use crate::framework::languages::SupportedLanguages;
use crate::framework::python::datamodel_config::load_main_py;
use crate::framework::versions::Version;
use crate::infrastructure::redis::redis_client::RedisClient;
use crate::project::Project;
use crate::proto::infrastructure_map::InfrastructureMap as ProtoInfrastructureMap;
use anyhow::{Context, Result};
use log::debug;
use protobuf::{EnumOrUnknown, Message as ProtoMessage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tokio::io::AsyncReadExt;
use tokio::process::Child;

/// Error types for InfrastructureMap protocol buffer operations
///
/// This enum defines errors that can occur when converting between protocol
/// buffer representations and Rust representations of the infrastructure map.
#[derive(Debug, thiserror::Error)]
#[error("Failed to convert infrastructure map from proto")]
#[non_exhaustive]
pub enum InfraMapProtoError {
    /// Error occurred during protobuf parsing
    #[error("Failed to parse proto message")]
    ProtoParseError(#[from] protobuf::Error),

    /// A required field was missing in the protobuf message
    #[error("Missing required field: {field_name}")]
    MissingField { field_name: String },
}

/// Types of primitives that can be represented in the infrastructure
///
/// These represent the core building blocks of the system that can be
/// transformed into infrastructure components.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PrimitiveTypes {
    /// A data model that defines the structure of data
    DataModel,
    /// A function that processes data
    Function,
    /// A database block for OLAP operations
    DBBlock,
    /// An API for consumption of data
    ConsumptionAPI,
}

/// Signature that uniquely identifies a primitive component
///
/// This combines the name and type of a primitive to provide a consistent
/// way to reference primitives throughout the system.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrimitiveSignature {
    /// Name of the primitive component
    pub name: String,
    /// Type of the primitive component
    pub primitive_type: PrimitiveTypes,
}

impl PrimitiveSignature {
    /// Converts this signature to its protocol buffer representation
    pub fn to_proto(&self) -> crate::proto::infrastructure_map::PrimitiveSignature {
        crate::proto::infrastructure_map::PrimitiveSignature {
            name: self.name.clone(),
            primitive_type: EnumOrUnknown::new(self.primitive_type.to_proto()),
            special_fields: Default::default(),
        }
    }

    pub fn from_proto(proto: crate::proto::infrastructure_map::PrimitiveSignature) -> Self {
        PrimitiveSignature {
            name: proto.name,
            primitive_type: PrimitiveTypes::from_proto(proto.primitive_type.unwrap()),
        }
    }
}

impl PrimitiveTypes {
    /// Converts this primitive type to its protocol buffer representation
    ///
    /// Maps each variant of the enum to the corresponding protocol buffer enum value.
    ///
    /// # Returns
    /// The protocol buffer enum value corresponding to this primitive type
    fn to_proto(&self) -> crate::proto::infrastructure_map::PrimitiveTypes {
        match self {
            PrimitiveTypes::DataModel => {
                crate::proto::infrastructure_map::PrimitiveTypes::DATA_MODEL
            }
            PrimitiveTypes::Function => crate::proto::infrastructure_map::PrimitiveTypes::FUNCTION,
            PrimitiveTypes::DBBlock => crate::proto::infrastructure_map::PrimitiveTypes::DB_BLOCK,
            PrimitiveTypes::ConsumptionAPI => {
                crate::proto::infrastructure_map::PrimitiveTypes::CONSUMPTION_API
            }
        }
    }

    /// Creates a primitive type from its protocol buffer representation
    ///
    /// Maps each protocol buffer enum value to the corresponding Rust enum variant.
    ///
    /// # Arguments
    /// * `proto` - The protocol buffer enum value to convert
    ///
    /// # Returns
    /// The corresponding PrimitiveTypes variant
    pub fn from_proto(proto: crate::proto::infrastructure_map::PrimitiveTypes) -> Self {
        match proto {
            crate::proto::infrastructure_map::PrimitiveTypes::DATA_MODEL => {
                PrimitiveTypes::DataModel
            }
            crate::proto::infrastructure_map::PrimitiveTypes::FUNCTION => PrimitiveTypes::Function,
            crate::proto::infrastructure_map::PrimitiveTypes::DB_BLOCK => PrimitiveTypes::DBBlock,
            crate::proto::infrastructure_map::PrimitiveTypes::CONSUMPTION_API => {
                PrimitiveTypes::ConsumptionAPI
            }
        }
    }
}

/// Represents a change to a database column
///
/// This enum captures the three possible states of change for a column:
/// addition, removal, or update with before and after states.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColumnChange {
    /// A new column has been added
    Added(Column),
    /// An existing column has been removed
    Removed(Column),
    /// An existing column has been modified
    Updated { before: Column, after: Column },
}

/// Represents changes to the order_by configuration of a table
///
/// Tracks the before and after states of the ordering columns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderByChange {
    /// Previous ordering columns
    pub before: Vec<String>,
    /// New ordering columns
    pub after: Vec<String>,
}

/// Represents a change to a database table
///
/// This captures the complete picture of table changes, including additions,
/// removals, and detailed updates with column-level changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub enum TableChange {
    /// A new table has been added
    Added(Table),
    /// An existing table has been removed
    Removed(Table),
    /// An existing table has been modified
    Updated {
        /// Name of the table that was updated
        name: String,
        /// List of column-level changes
        column_changes: Vec<ColumnChange>,
        /// Changes to the ordering columns
        order_by_change: OrderByChange,
        /// Complete representation of the table before changes
        before: Table,
        /// Complete representation of the table after changes
        after: Table,
    },
}

/// Generic representation of a change to any infrastructure component
///
/// This type-parametrized enum can represent changes to any serializable
/// component type in a consistent way.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Change<T: Serialize> {
    /// A new component has been added
    Added(Box<T>),
    /// An existing component has been removed
    Removed(Box<T>),
    /// An existing component has been modified
    Updated { before: Box<T>, after: Box<T> },
}

/// High-level categories of infrastructure changes
///
/// This enum categorizes changes by the part of the infrastructure they affect.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InfraChange {
    /// Changes to OLAP (Online Analytical Processing) components
    Olap(OlapChange),
    /// Changes to streaming components
    Streaming(StreamingChange),
    /// Changes to API components
    Api(ApiChange),
    /// Changes to process components
    Process(ProcessChange),
}

/// Changes to OLAP (database) components
///
/// This includes changes to tables and views.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub enum OlapChange {
    /// Change to a database table
    Table(TableChange),
    /// Change to a database view
    View(Change<View>),
}

/// Changes to streaming components
///
/// Currently only includes changes to topics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamingChange {
    /// Change to a streaming topic
    Topic(Change<Topic>),
}

/// Changes to API components
///
/// Currently only includes changes to API endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiChange {
    /// Change to an API endpoint
    ApiEndpoint(Change<ApiEndpoint>),
}

/// Changes to process components
///
/// This includes various types of processes that operate on the infrastructure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessChange {
    /// Change to a process that syncs data from a topic to a table
    TopicToTableSyncProcess(Change<TopicToTableSyncProcess>),
    /// Change to a process that syncs data between topics
    TopicToTopicSyncProcess(Change<TopicToTopicSyncProcess>),
    /// Change to a function process
    FunctionProcess(Change<FunctionProcess>),
    /// Change to an OLAP process
    OlapProcess(Change<OlapProcess>),
    /// Change to a consumption API web server
    ConsumptionApiWebServer(Change<ConsumptionApiWebServer>),
    /// Change to an orchestration worker
    OrchestrationWorker(Change<OrchestrationWorker>),
}

/// Collection of all changes detected between two infrastructure states
///
/// This struct aggregates changes across all parts of the infrastructure
/// and is the primary output of the difference calculation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InfraChanges {
    /// Changes to OLAP (database) components
    pub olap_changes: Vec<OlapChange>,
    /// Changes to process components
    pub processes_changes: Vec<ProcessChange>,
    /// Changes to API components
    pub api_changes: Vec<ApiChange>,
    /// Changes to streaming components
    pub streaming_engine_changes: Vec<StreamingChange>,
}

impl InfraChanges {
    /// Checks if there are any changes in this collection
    ///
    /// Returns true if all change vectors are empty, false otherwise.
    pub fn is_empty(&self) -> bool {
        self.olap_changes.is_empty()
            && self.processes_changes.is_empty()
            && self.api_changes.is_empty()
            && self.streaming_engine_changes.is_empty()
    }
}

/// Represents the complete infrastructure map of the system, containing all components and their relationships
///
/// The `InfrastructureMap` is the central data structure of the Moose framework's infrastructure management.
/// It contains a comprehensive representation of all infrastructure components and their relationships,
/// serving as the source of truth for the entire system architecture.
///
/// Components are organized by type and indexed by appropriate identifiers for efficient lookup.
/// The map can be serialized to/from various formats (JSON, Protocol Buffers) for persistence
/// and can be compared with other maps to detect changes.
///
/// The relationship between the components is maintained by reference rather than by value.
/// Helper methods facilitate navigating the map and finding related components.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfrastructureMap {
    /// Collection of topics indexed by topic ID
    pub topics: HashMap<String, Topic>,

    /// Collection of API endpoints indexed by endpoint ID
    pub api_endpoints: HashMap<String, ApiEndpoint>,

    /// Collection of database tables indexed by table name
    pub tables: HashMap<String, Table>,

    /// Collection of database views indexed by view name
    pub views: HashMap<String, View>,

    /// Processes that sync data from topics to tables
    pub topic_to_table_sync_processes: HashMap<String, TopicToTableSyncProcess>,

    /// Processes that sync data between topics
    #[serde(default = "HashMap::new")]
    pub topic_to_topic_sync_processes: HashMap<String, TopicToTopicSyncProcess>,

    /// Collection of function processes that transform data
    pub function_processes: HashMap<String, FunctionProcess>,

    /// Process handling OLAP database operations
    // TODO change to a hashmap of processes when we have several
    pub block_db_processes: OlapProcess,

    /// Web server handling consumption API endpoints
    // Not sure if we will want to change that or not in the future to be able to tell
    // the new consumption endpoints that were added or removed.
    pub consumption_api_web_server: ConsumptionApiWebServer,

    /// Collection of orchestration workers indexed by worker ID
    #[serde(default = "HashMap::new")]
    pub orchestration_workers: HashMap<String, OrchestrationWorker>,
}

impl InfrastructureMap {
    /// Creates a new infrastructure map from a project and primitive map
    ///
    /// This is the primary constructor for creating an infrastructure map. It transforms
    /// the high-level primitives (data models, functions, blocks, etc.) into concrete
    /// infrastructure components and their relationships.
    ///
    /// The method handles complex logic like:
    /// - Processing data models with version changes first
    /// - Creating appropriate infrastructure for each primitive type
    /// - Setting up relationships between components
    /// - Handling special cases for unchanged components with version changes
    ///
    /// # Arguments
    /// * `project` - The project context with configuration and features
    /// * `primitive_map` - The map of primitives to transform into infrastructure
    ///
    /// # Returns
    /// A complete infrastructure map with all components and their relationships
    pub fn new(project: &Project, primitive_map: PrimitiveMap) -> InfrastructureMap {
        let mut tables = HashMap::new();
        let mut views = HashMap::new();
        let mut topics = HashMap::new();
        let mut api_endpoints = HashMap::new();
        let mut topic_to_table_sync_processes = HashMap::new();
        let topic_to_topic_sync_processes = HashMap::new();
        let mut function_processes = HashMap::new();

        // Process data models that have changes in their latest version
        // This ensures we create new infrastructure for updated data models first
        let mut data_models_that_have_not_changed_with_new_version = vec![];

        // Iterate through data models and process those that have changes
        for data_model in primitive_map.data_models_iter() {
            // Check if the data model has changed compared to its previous version
            if primitive_map
                .datamodels
                .has_data_model_changed_with_previous_version(
                    &data_model.name,
                    data_model.version.as_str(),
                )
            {
                let topic = Topic::from_data_model(data_model);
                let api_endpoint = ApiEndpoint::from_data_model(data_model, &topic);

                // If storage is enabled for this data model, create necessary infrastructure
                if data_model.config.storage.enabled {
                    let table = data_model.to_table();
                    let topic_to_table_sync_process = TopicToTableSyncProcess::new(&topic, &table);

                    tables.insert(table.id(), table);
                    topic_to_table_sync_processes.insert(
                        topic_to_table_sync_process.id(),
                        topic_to_table_sync_process,
                    );
                }

                // If streaming engine is enabled, create topics and API endpoints
                if project.features.streaming_engine {
                    topics.insert(topic.id(), topic);
                    api_endpoints.insert(api_endpoint.id(), api_endpoint);
                }
            } else {
                // Store unchanged data models for later processing
                // This allows us to reference infrastructure created by older versions
                data_models_that_have_not_changed_with_new_version.push(data_model);
            }
        }

        // Process data models that haven't changed with their registered versions
        // For those requiring storage, we create views pointing to the oldest table
        // that has the data with the same schema. We also reuse existing topics.
        for data_model in data_models_that_have_not_changed_with_new_version {
            match primitive_map
                .datamodels
                .find_earliest_similar_version(&data_model.name, data_model.version.as_str())
            {
                Some(previous_version_model) => {
                    // This will be already created with the previous data model.
                    // That's why we don't add it to the map
                    let previous_version_topic = Topic::from_data_model(previous_version_model);
                    let api_endpoint =
                        ApiEndpoint::from_data_model(data_model, &previous_version_topic);

                    if data_model.config.storage.enabled
                        && previous_version_model.config.storage.enabled
                    {
                        let view = View::alias_view(data_model, previous_version_model);
                        views.insert(view.id(), view);
                    }

                    if project.features.streaming_engine {
                        api_endpoints.insert(api_endpoint.id(), api_endpoint);
                    }
                }
                None => {
                    log::error!(
                        "Could not find previous version with no change for data model: {} {}",
                        data_model.name,
                        data_model.version
                    );
                    log::debug!("Data Models Dump: {:?}", primitive_map.datamodels);
                }
            }
        }

        if !project.features.streaming_engine && !primitive_map.functions.is_empty() {
            log::error!("Streaming disabled. Functions are disabled.");
            show_message_wrapper(
                MessageType::Error,
                Message {
                    action: "Disabled".to_string(),
                    details: format!(
                        "Streaming is disabled but {} function(s) found.",
                        primitive_map.functions.len()
                    ),
                },
            );
        } else {
            for function in primitive_map.functions.iter() {
                // Currently we are not creating 1 per function source and target.
                // We reuse the topics that were created from the data models.
                // Unless for streaming function migrations where we will have to create new topics.

                let function_process = FunctionProcess::from_function(function, &topics);
                function_processes.insert(function_process.id(), function_process);
            }
        }

        // TODO update here when we have several blocks processes
        let block_db_processes = OlapProcess::from_blocks(&primitive_map.blocks);

        // consumption api endpoints
        let consumption_api_web_server = ConsumptionApiWebServer {};
        for api_endpoint in primitive_map.consumption.endpoint_files {
            let api_endpoint_infra = ApiEndpoint::from(api_endpoint);
            api_endpoints.insert(api_endpoint_infra.id(), api_endpoint_infra);
        }

        // Orchestration workers
        let mut orchestration_workers = HashMap::new();
        let orchestration_worker = OrchestrationWorker::new(project.language);
        orchestration_workers.insert(orchestration_worker.id(), orchestration_worker);

        InfrastructureMap {
            // primitive_map,
            topics,
            api_endpoints,
            topic_to_table_sync_processes,
            topic_to_topic_sync_processes,
            tables,
            views,
            function_processes,
            block_db_processes,
            consumption_api_web_server,
            orchestration_workers,
        }
    }

    /// Generates all the changes needed for initial infrastructure deployment
    ///
    /// This method creates a complete set of changes representing the creation of
    /// all components in this infrastructure map. It's used when deploying to an
    /// environment with no existing infrastructure.
    ///
    /// # Arguments
    /// * `project` - The project context with configuration and features
    ///
    /// # Returns
    /// An `InfraChanges` object containing all components marked as additions
    pub fn init(&self, project: &Project) -> InfraChanges {
        let olap_changes = self.init_tables();
        let processes_changes = self.init_processes(project);
        let api_changes = self.init_api_endpoints();
        let streaming_engine_changes = self.init_topics();

        InfraChanges {
            olap_changes,
            processes_changes,
            api_changes,
            streaming_engine_changes,
        }
    }

    /// Creates changes for initial topic deployment
    ///
    /// Generates changes representing the creation of all topics in this map.
    ///
    /// # Returns
    /// A vector of `StreamingChange` objects for topic creation
    pub fn init_topics(&self) -> Vec<StreamingChange> {
        self.topics
            .values()
            .map(|topic| StreamingChange::Topic(Change::<Topic>::Added(Box::new(topic.clone()))))
            .collect()
    }

    /// Creates changes for initial API endpoint deployment
    ///
    /// Generates changes representing the creation of all API endpoints in this map.
    ///
    /// # Returns
    /// A vector of `ApiChange` objects for endpoint creation
    pub fn init_api_endpoints(&self) -> Vec<ApiChange> {
        self.api_endpoints
            .values()
            .map(|api_endpoint| {
                ApiChange::ApiEndpoint(Change::<ApiEndpoint>::Added(Box::new(api_endpoint.clone())))
            })
            .collect()
    }

    /// Creates changes for initial table deployment
    ///
    /// Generates changes representing the creation of all tables in this map.
    ///
    /// # Returns
    /// A vector of `OlapChange` objects for table creation
    pub fn init_tables(&self) -> Vec<OlapChange> {
        self.tables
            .values()
            .map(|table| OlapChange::Table(TableChange::Added(table.clone())))
            .collect()
    }

    /// Creates changes for initial process deployment
    ///
    /// Generates changes representing the creation of all processes in this map.
    ///
    /// # Arguments
    /// * `project` - The project context with configuration and features
    ///
    /// # Returns
    /// A vector of `ProcessChange` objects for process creation
    pub fn init_processes(&self, project: &Project) -> Vec<ProcessChange> {
        let mut process_changes: Vec<ProcessChange> = self
            .topic_to_table_sync_processes
            .values()
            .map(|topic_to_table_sync_process| {
                ProcessChange::TopicToTableSyncProcess(Change::<TopicToTableSyncProcess>::Added(
                    Box::new(topic_to_table_sync_process.clone()),
                ))
            })
            .collect();

        let mut topic_to_topic_process_changes: Vec<ProcessChange> = self
            .topic_to_topic_sync_processes
            .values()
            .map(|topic_to_table_sync_process| {
                ProcessChange::TopicToTopicSyncProcess(Change::<TopicToTopicSyncProcess>::Added(
                    Box::new(topic_to_table_sync_process.clone()),
                ))
            })
            .collect();
        process_changes.append(&mut topic_to_topic_process_changes);

        let mut function_process_changes: Vec<ProcessChange> = self
            .function_processes
            .values()
            .map(|function_process| {
                ProcessChange::FunctionProcess(Change::<FunctionProcess>::Added(Box::new(
                    function_process.clone(),
                )))
            })
            .collect();

        process_changes.append(&mut function_process_changes);

        // TODO Change this when we have multiple processes for blocks
        process_changes.push(ProcessChange::OlapProcess(Change::<OlapProcess>::Added(
            Box::new(OlapProcess {}),
        )));

        process_changes.push(ProcessChange::ConsumptionApiWebServer(Change::<
            ConsumptionApiWebServer,
        >::Added(
            Box::new(ConsumptionApiWebServer {}),
        )));

        process_changes.push(ProcessChange::OrchestrationWorker(Change::<
            OrchestrationWorker,
        >::Added(
            Box::new(OrchestrationWorker {
                supported_language: project.language,
            }),
        )));

        process_changes
    }

    /// Compares this infrastructure map with a target map to determine changes
    ///
    /// This is a central method of the infrastructure management system. It performs a
    /// comprehensive comparison between this map (representing the current state) and
    /// a target map (representing the desired state), calculating all changes needed
    /// to transform the current state into the desired state.
    ///
    /// The method:
    /// - Analyzes each component type separately (topics, API endpoints, tables, views, etc.)
    /// - Identifies additions, removals, and modifications
    /// - For complex components like tables, calculates detailed changes (column additions, etc.)
    /// - Logs detailed information about the detected changes
    ///
    /// # Arguments
    /// * `target_map` - The target infrastructure map to compare against
    ///
    /// # Returns
    /// An `InfraChanges` object containing all detected changes
    pub fn diff(&self, target_map: &InfrastructureMap) -> InfraChanges {
        let mut changes = InfraChanges::default();

        // =================================================================
        //                              Topics
        // =================================================================
        log::info!("Analyzing changes in Topics...");
        let mut topic_updates = 0;
        let mut topic_removals = 0;
        let mut topic_additions = 0;

        for (id, topic) in &self.topics {
            if let Some(target_topic) = target_map.topics.get(id) {
                if topic != target_topic {
                    log::debug!("Topic updated: {} ({})", topic.name, id);
                    topic_updates += 1;
                    changes
                        .streaming_engine_changes
                        .push(StreamingChange::Topic(Change::<Topic>::Updated {
                            before: Box::new(topic.clone()),
                            after: Box::new(target_topic.clone()),
                        }));
                }
            } else {
                log::debug!("Topic removed: {} ({})", topic.name, id);
                topic_removals += 1;
                changes
                    .streaming_engine_changes
                    .push(StreamingChange::Topic(Change::<Topic>::Removed(Box::new(
                        topic.clone(),
                    ))));
            }
        }

        for (id, topic) in &target_map.topics {
            if !self.topics.contains_key(id) {
                log::debug!("Topic added: {} ({})", topic.name, id);
                topic_additions += 1;
                changes
                    .streaming_engine_changes
                    .push(StreamingChange::Topic(Change::<Topic>::Added(Box::new(
                        topic.clone(),
                    ))));
            }
        }

        log::info!(
            "Topic changes: {} added, {} removed, {} updated",
            topic_additions,
            topic_removals,
            topic_updates
        );

        // =================================================================
        //                              API Endpoints
        // =================================================================
        log::info!("Analyzing changes in API Endpoints...");
        let mut api_updates = 0;
        let mut api_removals = 0;
        let mut api_additions = 0;

        for (id, api_endpoint) in &self.api_endpoints {
            if let Some(target_api_endpoint) = target_map.api_endpoints.get(id) {
                if api_endpoint != target_api_endpoint {
                    log::debug!("API Endpoint updated: {}", id);
                    api_updates += 1;
                    changes.api_changes.push(ApiChange::ApiEndpoint(
                        Change::<ApiEndpoint>::Updated {
                            before: Box::new(api_endpoint.clone()),
                            after: Box::new(target_api_endpoint.clone()),
                        },
                    ));
                }
            } else {
                log::debug!("API Endpoint removed: {}", id);
                api_removals += 1;
                changes
                    .api_changes
                    .push(ApiChange::ApiEndpoint(Change::<ApiEndpoint>::Removed(
                        Box::new(api_endpoint.clone()),
                    )));
            }
        }

        for (id, api_endpoint) in &target_map.api_endpoints {
            if !self.api_endpoints.contains_key(id) {
                log::debug!("API Endpoint added: {}", id);
                api_additions += 1;
                changes
                    .api_changes
                    .push(ApiChange::ApiEndpoint(Change::<ApiEndpoint>::Added(
                        Box::new(api_endpoint.clone()),
                    )));
            }
        }

        log::info!(
            "API Endpoint changes: {} added, {} removed, {} updated",
            api_additions,
            api_removals,
            api_updates
        );

        // =================================================================
        //                              Tables
        // =================================================================
        log::info!("Analyzing changes in Tables...");
        let olap_changes_len_before = changes.olap_changes.len();
        Self::diff_tables(&self.tables, &target_map.tables, &mut changes.olap_changes);
        let table_changes = changes.olap_changes.len() - olap_changes_len_before;
        log::info!("Table changes detected: {}", table_changes);

        // =================================================================
        //                              Views
        // =================================================================
        log::info!("Analyzing changes in Views...");
        let mut view_updates = 0;
        let mut view_removals = 0;
        let mut view_additions = 0;

        for (id, view) in &self.views {
            if let Some(target_view) = target_map.views.get(id) {
                if view != target_view {
                    log::debug!("View updated: {}", view.name);
                    view_updates += 1;
                    changes
                        .olap_changes
                        .push(OlapChange::View(Change::<View>::Updated {
                            before: Box::new(view.clone()),
                            after: Box::new(target_view.clone()),
                        }));
                }
            } else {
                log::debug!("View removed: {}", view.name);
                view_removals += 1;
                changes
                    .olap_changes
                    .push(OlapChange::View(Change::<View>::Removed(Box::new(
                        view.clone(),
                    ))));
            }
        }

        for (id, view) in &target_map.views {
            if !self.views.contains_key(id) {
                log::debug!("View added: {}", view.name);
                view_additions += 1;
                changes
                    .olap_changes
                    .push(OlapChange::View(Change::<View>::Added(Box::new(
                        view.clone(),
                    ))));
            }
        }

        log::info!(
            "View changes: {} added, {} removed, {} updated",
            view_additions,
            view_removals,
            view_updates
        );

        // =================================================================
        //                              Topic to Table Sync Processes
        // =================================================================
        log::info!("Analyzing changes in Topic to Table Sync Processes...");
        let mut t2t_sync_updates = 0;
        let mut t2t_sync_removals = 0;
        let mut t2t_sync_additions = 0;

        for (id, topic_to_table_sync_process) in &self.topic_to_table_sync_processes {
            if let Some(target_topic_to_table_sync_process) =
                target_map.topic_to_table_sync_processes.get(id)
            {
                if topic_to_table_sync_process != target_topic_to_table_sync_process {
                    log::debug!("Topic to Table Sync Process updated: {}", id);
                    t2t_sync_updates += 1;
                    changes
                        .processes_changes
                        .push(ProcessChange::TopicToTableSyncProcess(Change::<
                            TopicToTableSyncProcess,
                        >::Updated {
                            before: Box::new(topic_to_table_sync_process.clone()),
                            after: Box::new(target_topic_to_table_sync_process.clone()),
                        }));
                }
            } else {
                log::debug!("Topic to Table Sync Process removed: {}", id);
                t2t_sync_removals += 1;
                changes
                    .processes_changes
                    .push(ProcessChange::TopicToTableSyncProcess(Change::<
                        TopicToTableSyncProcess,
                    >::Removed(
                        Box::new(topic_to_table_sync_process.clone()),
                    )));
            }
        }

        for (id, topic_to_table_sync_process) in &target_map.topic_to_table_sync_processes {
            if !self.topic_to_table_sync_processes.contains_key(id) {
                log::debug!("Topic to Table Sync Process added: {}", id);
                t2t_sync_additions += 1;
                changes
                    .processes_changes
                    .push(ProcessChange::TopicToTableSyncProcess(Change::<
                        TopicToTableSyncProcess,
                    >::Added(
                        Box::new(topic_to_table_sync_process.clone()),
                    )));
            }
        }

        log::info!(
            "Topic to Table Sync Process changes: {} added, {} removed, {} updated",
            t2t_sync_additions,
            t2t_sync_removals,
            t2t_sync_updates
        );

        // =================================================================
        //                              Topic to Topic Sync Processes
        // =================================================================
        log::info!("Analyzing changes in Topic to Topic Sync Processes...");
        let mut t2t_topic_sync_updates = 0;
        let mut t2t_topic_sync_removals = 0;
        let mut t2t_topic_sync_additions = 0;

        for (id, topic_to_topic_sync_process) in &self.topic_to_topic_sync_processes {
            if let Some(target_topic_to_topic_sync_process) =
                target_map.topic_to_topic_sync_processes.get(id)
            {
                if topic_to_topic_sync_process != target_topic_to_topic_sync_process {
                    log::debug!("Topic to Topic Sync Process updated: {}", id);
                    t2t_topic_sync_updates += 1;
                    changes
                        .processes_changes
                        .push(ProcessChange::TopicToTopicSyncProcess(Change::<
                            TopicToTopicSyncProcess,
                        >::Updated {
                            before: Box::new(topic_to_topic_sync_process.clone()),
                            after: Box::new(target_topic_to_topic_sync_process.clone()),
                        }));
                }
            } else {
                log::debug!("Topic to Topic Sync Process removed: {}", id);
                t2t_topic_sync_removals += 1;
                changes
                    .processes_changes
                    .push(ProcessChange::TopicToTopicSyncProcess(Change::<
                        TopicToTopicSyncProcess,
                    >::Removed(
                        Box::new(topic_to_topic_sync_process.clone()),
                    )));
            }
        }

        for (id, topic_to_topic_sync_process) in &target_map.topic_to_topic_sync_processes {
            if !self.topic_to_topic_sync_processes.contains_key(id) {
                log::debug!("Topic to Topic Sync Process added: {}", id);
                t2t_topic_sync_additions += 1;
                changes
                    .processes_changes
                    .push(ProcessChange::TopicToTopicSyncProcess(Change::<
                        TopicToTopicSyncProcess,
                    >::Added(
                        Box::new(topic_to_topic_sync_process.clone()),
                    )));
            }
        }

        log::info!(
            "Topic to Topic Sync Process changes: {} added, {} removed, {} updated",
            t2t_topic_sync_additions,
            t2t_topic_sync_removals,
            t2t_topic_sync_updates
        );

        // =================================================================
        //                             Function Processes
        // =================================================================
        log::info!("Analyzing changes in Function Processes...");
        let mut function_updates = 0;
        let mut function_removals = 0;
        let mut function_additions = 0;

        for (id, function_process) in &self.function_processes {
            if let Some(target_function_process) = target_map.function_processes.get(id) {
                // In this case we don't do a comparison check because the function process is not just
                // dependent on changing one file, but also on its dependencies. Until we are able to
                // properly compare the function processes holistically (File + Dependencies), we will just
                // assume that the function process has changed.
                log::debug!("Function Process updated: {}", id);
                function_updates += 1;
                changes
                    .processes_changes
                    .push(ProcessChange::FunctionProcess(
                        Change::<FunctionProcess>::Updated {
                            before: Box::new(function_process.clone()),
                            after: Box::new(target_function_process.clone()),
                        },
                    ));
            } else {
                log::debug!("Function Process removed: {}", id);
                function_removals += 1;
                changes
                    .processes_changes
                    .push(ProcessChange::FunctionProcess(
                        Change::<FunctionProcess>::Removed(Box::new(function_process.clone())),
                    ));
            }
        }

        for (id, function_process) in &target_map.function_processes {
            if !self.function_processes.contains_key(id) {
                log::debug!("Function Process added: {}", id);
                function_additions += 1;
                changes
                    .processes_changes
                    .push(ProcessChange::FunctionProcess(
                        Change::<FunctionProcess>::Added(Box::new(function_process.clone())),
                    ));
            }
        }

        log::info!(
            "Function Process changes: {} added, {} removed, {} updated",
            function_additions,
            function_removals,
            function_updates
        );

        // =================================================================
        //                             Blocks Processes
        // =================================================================
        log::info!("Analyzing changes in OLAP Processes...");

        // Until we refactor to have multiple processes, we will consider that we need to restart
        // the process all the times and the blocks changes all the time.
        // Once we do the other refactor, we will be able to compare the changes and only restart
        // the process if there are changes

        // currently we assume there is always a change and restart the processes
        log::debug!("OLAP Process updated (assumed for now)");
        changes.processes_changes.push(ProcessChange::OlapProcess(
            Change::<OlapProcess>::Updated {
                before: Box::new(OlapProcess {}),
                after: Box::new(OlapProcess {}),
            },
        ));

        // =================================================================
        //                          Consumption Process
        // =================================================================
        log::info!("Analyzing changes in Consumption API Web Server...");

        // We are currently not tracking individual consumption endpoints, so we will just restart
        // the consumption web server when something changed. we might want to change that in the future
        // to be able to only make changes when something in the dependency tree of a consumption api has
        // changed.
        log::debug!("Consumption API Web Server updated (assumed for now)");
        changes
            .processes_changes
            .push(ProcessChange::ConsumptionApiWebServer(Change::<
                ConsumptionApiWebServer,
            >::Updated {
                before: Box::new(ConsumptionApiWebServer {}),
                after: Box::new(ConsumptionApiWebServer {}),
            }));

        // =================================================================
        //                      Orchestration Workers
        // =================================================================
        log::info!("Analyzing changes in Orchestration Workers...");
        let mut worker_updates = 0;
        let mut worker_removals = 0;
        let mut worker_additions = 0;

        for (id, orchestration_worker) in &self.orchestration_workers {
            if let Some(target_orchestration_worker) = target_map.orchestration_workers.get(id) {
                // Until we track individual files changes, we want workers to be restarted for every change
                log::debug!("Orchestration Worker updated: {}", id);
                worker_updates += 1;
                changes
                    .processes_changes
                    .push(ProcessChange::OrchestrationWorker(Change::<
                        OrchestrationWorker,
                    >::Updated {
                        before: Box::new(orchestration_worker.clone()),
                        after: Box::new(target_orchestration_worker.clone()),
                    }));
            } else {
                log::debug!("Orchestration Worker removed: {}", id);
                worker_removals += 1;
                changes
                    .processes_changes
                    .push(ProcessChange::OrchestrationWorker(Change::<
                        OrchestrationWorker,
                    >::Removed(
                        Box::new(orchestration_worker.clone()),
                    )));
            }
        }

        for (id, orchestration_worker) in &target_map.orchestration_workers {
            if !self.orchestration_workers.contains_key(id) {
                log::debug!("Orchestration Worker added: {}", id);
                worker_additions += 1;
                changes
                    .processes_changes
                    .push(ProcessChange::OrchestrationWorker(Change::<
                        OrchestrationWorker,
                    >::Added(
                        Box::new(orchestration_worker.clone()),
                    )));
            }
        }

        log::info!(
            "Orchestration Worker changes: {} added, {} removed, {} updated",
            worker_additions,
            worker_removals,
            worker_updates
        );

        // Summarize total changes
        log::info!(
            "Total changes: {} OLAP, {} Process, {} API, {} Streaming",
            changes.olap_changes.len(),
            changes.processes_changes.len(),
            changes.api_changes.len(),
            changes.streaming_engine_changes.len()
        );

        changes
    }

    /// Compare tables between two infrastructure maps and compute the differences
    ///
    /// This method identifies added, removed, and updated tables by comparing
    /// the source and target table maps. For updated tables, it performs a detailed
    /// analysis of column-level changes.
    ///
    /// Changes are collected in the provided changes vector with detailed logging
    /// of what has changed.
    ///
    /// # Arguments
    /// * `self_tables` - HashMap of source tables to compare from
    /// * `target_tables` - HashMap of target tables to compare against
    /// * `olap_changes` - Mutable vector to collect the identified changes
    pub fn diff_tables(
        self_tables: &HashMap<String, Table>,
        target_tables: &HashMap<String, Table>,
        olap_changes: &mut Vec<OlapChange>,
    ) {
        log::info!(
            "Analyzing table differences between {} source tables and {} target tables",
            self_tables.len(),
            target_tables.len()
        );

        let mut table_updates = 0;
        let mut table_removals = 0;
        let mut table_additions = 0;

        for (id, table) in self_tables {
            if let Some(target_table) = target_tables.get(id) {
                if table != target_table {
                    // Debug logging to identify what's different
                    log::debug!("Table '{}' has differences:", id);
                    if table.name != target_table.name {
                        log::debug!(
                            "  - Name changed: '{}' -> '{}'",
                            table.name,
                            target_table.name
                        );
                    }
                    if table.columns != target_table.columns {
                        log::debug!("  - Column differences detected");
                        // Detail column differences
                        for col in &table.columns {
                            if !target_table
                                .columns
                                .iter()
                                .any(|c| c.name == col.name && c == col)
                            {
                                if target_table.columns.iter().any(|c| c.name == col.name) {
                                    log::debug!("    * Column '{}' modified", col.name);
                                } else {
                                    log::debug!("    * Column '{}' removed", col.name);
                                }
                            }
                        }
                        for col in &target_table.columns {
                            if !table.columns.iter().any(|c| c.name == col.name) {
                                log::debug!("    * Column '{}' added", col.name);
                            }
                        }
                    }
                    if table.order_by != target_table.order_by {
                        log::debug!(
                            "  - Order by changed: {:?} -> {:?}",
                            table.order_by,
                            target_table.order_by
                        );
                    }
                    if table.deduplicate != target_table.deduplicate {
                        log::debug!(
                            "  - Deduplicate changed: {} -> {}",
                            table.deduplicate,
                            target_table.deduplicate
                        );
                    }
                    if table.version != target_table.version {
                        log::debug!(
                            "  - Version changed: {} -> {}",
                            table.version,
                            target_table.version
                        );
                    }
                    if table.source_primitive != target_table.source_primitive {
                        log::debug!(
                            "  - Source primitive changed: {:?} -> {:?}",
                            table.source_primitive,
                            target_table.source_primitive
                        );
                    }

                    let column_changes = compute_table_diff(table, target_table);

                    // Log column change details
                    if !column_changes.is_empty() {
                        log::debug!("Column changes for table '{}':", table.name);
                        for change in &column_changes {
                            match change {
                                ColumnChange::Added(col) => {
                                    log::debug!(
                                        "  - Added column: {} ({})",
                                        col.name,
                                        col.data_type
                                    );
                                }
                                ColumnChange::Removed(col) => {
                                    log::debug!(
                                        "  - Removed column: {} ({})",
                                        col.name,
                                        col.data_type
                                    );
                                }
                                ColumnChange::Updated { before, after } => {
                                    log::debug!("  - Updated column: {}", before.name);
                                    if before.data_type != after.data_type {
                                        log::debug!(
                                            "    * Type changed: {:?} -> {:?}",
                                            before.data_type,
                                            after.data_type
                                        );
                                    }
                                    if before.required != after.required {
                                        log::debug!(
                                            "    * Required changed: {:?} -> {:?}",
                                            before.required,
                                            after.required
                                        );
                                    }
                                    if before.unique != after.unique {
                                        log::debug!(
                                            "    * Unique changed: {:?} -> {:?}",
                                            before.unique,
                                            after.unique
                                        );
                                    }
                                    if before.primary_key != after.primary_key {
                                        log::debug!(
                                            "    * Primary key changed: {:?} -> {:?}",
                                            before.primary_key,
                                            after.primary_key
                                        );
                                    }
                                    if before.default != after.default {
                                        log::debug!(
                                            "    * Default value changed: {:?} -> {:?}",
                                            before.default,
                                            after.default
                                        );
                                    }
                                }
                            }
                        }
                    }

                    let order_by_change = if table.order_by != target_table.order_by {
                        OrderByChange {
                            before: table.order_by.clone(),
                            after: target_table.order_by.clone(),
                        }
                    } else {
                        OrderByChange {
                            before: vec![],
                            after: vec![],
                        }
                    };

                    // Only push changes if there are actual differences to report
                    if !column_changes.is_empty()
                        || table.order_by != target_table.order_by
                        || table.deduplicate != target_table.deduplicate
                    {
                        table_updates += 1;
                        olap_changes.push(OlapChange::Table(TableChange::Updated {
                            name: table.name.clone(),
                            column_changes,
                            order_by_change,
                            before: table.clone(),
                            after: target_table.clone(),
                        }));
                    }
                }
            } else {
                log::debug!("Table '{}' removed", table.name);
                table_removals += 1;
                olap_changes.push(OlapChange::Table(TableChange::Removed(table.clone())));
            }
        }

        for (id, table) in target_tables {
            if !self_tables.contains_key(id) {
                log::debug!(
                    "Table '{}' added with {} columns",
                    table.name,
                    table.columns.len()
                );
                for col in &table.columns {
                    log::trace!("  - Column: {} ({})", col.name, col.data_type);
                }
                table_additions += 1;
                olap_changes.push(OlapChange::Table(TableChange::Added(table.clone())));
            }
        }

        log::info!(
            "Table changes: {} added, {} removed, {} updated",
            table_additions,
            table_removals,
            table_updates
        );
    }

    /// Serializes the infrastructure map to JSON and saves it to a file
    ///
    /// # Arguments
    /// * `path` - The path where the JSON file should be saved
    ///
    /// # Returns
    /// A Result indicating success or an IO error
    pub fn save_to_json(&self, path: &Path) -> Result<(), std::io::Error> {
        let json = serde_json::to_string(self)?;
        fs::write(path, json)
    }

    /// Loads an infrastructure map from a JSON file
    ///
    /// # Arguments
    /// * `path` - The path to the JSON file
    ///
    /// # Returns
    /// A Result containing either the loaded map or an IO error
    pub fn load_from_json(path: &Path) -> Result<Self, std::io::Error> {
        let json = fs::read_to_string(path)?;
        let infra_map = serde_json::from_str(&json)?;
        Ok(infra_map)
    }

    /// Stores the infrastructure map in Redis for persistence and sharing
    ///
    /// Serializes the map to protocol buffers and stores it in Redis using
    /// a service-specific prefix.
    ///
    /// # Arguments
    /// * `redis_client` - The Redis client to use for storage
    ///
    /// # Returns
    /// A Result indicating success or an error
    pub async fn store_in_redis(&self, redis_client: &RedisClient) -> Result<()> {
        let encoded: Vec<u8> = self.to_proto().write_to_bytes()?;
        redis_client
            .set_with_service_prefix("infrastructure_map", &encoded)
            .await
            .context("Failed to store InfrastructureMap in Redis")?;

        Ok(())
    }

    /// Loads an infrastructure map from Redis
    ///
    /// Attempts to retrieve the map from Redis and deserialize it from
    /// protocol buffers.
    ///
    /// # Arguments
    /// * `redis_client` - The Redis client to use for retrieval
    ///
    /// # Returns
    /// A Result containing either the loaded map (if found) or None (if not found),
    /// or an error if retrieval or deserialization failed
    pub async fn load_from_redis(redis_client: &RedisClient) -> Result<Option<Self>> {
        let encoded = redis_client
            .get_with_service_prefix("infrastructure_map")
            .await
            .context("Failed to get InfrastructureMap from Redis")?;

        if let Some(encoded) = encoded {
            let decoded = InfrastructureMap::from_proto(encoded).map_err(|e| {
                anyhow::anyhow!("Failed to decode InfrastructureMap from proto: {}", e)
            })?;
            Ok(Some(decoded))
        } else {
            Ok(None)
        }
    }

    /// Converts the infrastructure map to its protocol buffer representation
    ///
    /// This creates a complete protocol buffer representation of the map
    /// for serialization and transport.
    ///
    /// # Returns
    /// A protocol buffer representation of the infrastructure map
    pub fn to_proto(&self) -> ProtoInfrastructureMap {
        ProtoInfrastructureMap {
            topics: self
                .topics
                .iter()
                .map(|(k, v)| (k.clone(), v.to_proto()))
                .collect(),
            api_endpoints: self
                .api_endpoints
                .iter()
                .map(|(k, v)| (k.clone(), v.to_proto()))
                .collect(),
            tables: self
                .tables
                .iter()
                .map(|(k, v)| (k.clone(), v.to_proto()))
                .collect(),
            views: self
                .views
                .iter()
                .map(|(k, v)| (k.clone(), v.to_proto()))
                .collect(),
            topic_to_table_sync_processes: self
                .topic_to_table_sync_processes
                .iter()
                .map(|(k, v)| (k.clone(), v.to_proto()))
                .collect(),
            topic_to_topic_sync_processes: self
                .topic_to_topic_sync_processes
                .iter()
                .map(|(k, v)| (k.clone(), v.to_proto()))
                .collect(),
            function_processes: self
                .function_processes
                .iter()
                .map(|(k, v)| (k.clone(), v.to_proto()))
                .collect(),
            // Still here for reverse compatibility
            initial_data_loads: HashMap::new(),
            orchestration_workers: self
                .orchestration_workers
                .iter()
                .map(|(k, v)| (k.clone(), v.to_proto()))
                .collect(),
            special_fields: Default::default(),
        }
    }

    /// Serializes the infrastructure map to protocol buffer bytes
    ///
    /// # Returns
    /// A byte vector containing the serialized map
    pub fn to_proto_bytes(&self) -> Vec<u8> {
        self.to_proto().write_to_bytes().unwrap()
    }

    /// Creates an infrastructure map from its protocol buffer representation
    ///
    /// # Arguments
    /// * `bytes` - The byte vector containing the serialized map
    ///
    /// # Returns
    /// A Result containing either the deserialized map or a proto error
    pub fn from_proto(bytes: Vec<u8>) -> Result<Self, InfraMapProtoError> {
        let proto = ProtoInfrastructureMap::parse_from_bytes(&bytes)?;

        Ok(InfrastructureMap {
            topics: proto
                .topics
                .into_iter()
                .map(|(k, v)| (k, Topic::from_proto(v)))
                .collect(),
            api_endpoints: proto
                .api_endpoints
                .into_iter()
                .map(|(k, v)| (k, ApiEndpoint::from_proto(v)))
                .collect(),
            tables: proto
                .tables
                .into_iter()
                .map(|(k, v)| (k, Table::from_proto(v)))
                .collect(),
            views: proto
                .views
                .into_iter()
                .map(|(k, v)| (k, View::from_proto(v)))
                .collect(),
            topic_to_table_sync_processes: proto
                .topic_to_table_sync_processes
                .into_iter()
                .map(|(k, v)| (k, TopicToTableSyncProcess::from_proto(v)))
                .collect(),
            topic_to_topic_sync_processes: proto
                .topic_to_topic_sync_processes
                .into_iter()
                .map(|(k, v)| (k, TopicToTopicSyncProcess::from_proto(v)))
                .collect(),
            function_processes: proto
                .function_processes
                .into_iter()
                .map(|(k, v)| (k, FunctionProcess::from_proto(v)))
                .collect(),
            orchestration_workers: proto
                .orchestration_workers
                .into_iter()
                .map(|(k, v)| (k, OrchestrationWorker::from_proto(v)))
                .collect(),
            consumption_api_web_server: ConsumptionApiWebServer {},
            block_db_processes: OlapProcess {},
        })
    }

    /// Adds a table to the infrastructure map
    ///
    /// # Arguments
    /// * `table` - The table to add
    pub fn add_table(&mut self, table: Table) {
        self.tables.insert(table.id(), table);
    }

    /// Finds a table by name
    ///
    /// # Arguments
    /// * `name` - The name of the table to find
    ///
    /// # Returns
    /// An Option containing a reference to the table if found
    pub fn find_table_by_name(&self, name: &str) -> Option<&Table> {
        self.tables.values().find(|table| table.name == name)
    }

    /// Adds a topic to the infrastructure map
    ///
    /// # Arguments
    /// * `topic` - The topic to add
    pub fn add_topic(&mut self, topic: Topic) {
        self.topics.insert(topic.id(), topic);
    }

    /// Loads an infrastructure map from user code
    ///
    /// # Arguments
    /// * `project` - The project to load the infrastructure map from
    ///
    /// # Returns
    /// A Result containing the infrastructure map or an error
    pub async fn load_from_user_code(project: &Project) -> anyhow::Result<Self> {
        let partial = if project.language == SupportedLanguages::Typescript {
            let process = crate::framework::typescript::export_collectors::collect_from_index(
                &project.project_location,
            )?;

            PartialInfrastructureMap::from_subprocess(process, "index.ts").await?
        } else {
            load_main_py(&project.project_location).await?
        };
        Ok(partial.into_infra_map(project.language))
    }

    /// Gets a topic by its ID
    ///
    /// # Arguments
    /// * `id` - The ID of the topic to get
    ///
    /// # Returns
    /// An Option containing a reference to the topic if found
    pub fn get_topic_by_id(&self, id: &str) -> Option<&Topic> {
        self.topics.get(id)
    }

    /// Gets a topic by its name
    ///
    /// # Arguments
    /// * `name` - The name of the topic to get
    ///
    /// # Returns
    /// An Option containing a reference to the topic if found
    pub fn get_topic_by_name(&self, name: &str) -> Option<&Topic> {
        self.topics.values().find(|topic| topic.name == name)
    }

    pub fn from_json_value(
        language: SupportedLanguages,
        value: serde_json::Value,
    ) -> Result<Self, serde_json::Error> {
        let partial: PartialInfrastructureMap = serde_json::from_value(value)?;
        let tables = partial.convert_tables();
        let topics = partial.convert_topics();
        let api_endpoints = partial.convert_api_endpoints(language, &topics);
        let topic_to_table_sync_processes = partial.create_topic_to_table_sync_processes(&topics);
        let function_processes = partial.create_function_processes(language, &topics);

        Ok(InfrastructureMap {
            topics,
            api_endpoints,
            tables,
            views: partial.views,
            topic_to_table_sync_processes,
            topic_to_topic_sync_processes: partial.topic_to_topic_sync_processes,
            function_processes,
            block_db_processes: partial.block_db_processes.unwrap_or(OlapProcess {}),
            consumption_api_web_server: partial
                .consumption_api_web_server
                .unwrap_or(ConsumptionApiWebServer {}),
            orchestration_workers: partial.orchestration_workers,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PartialTable {
    pub name: String,
    pub columns: Vec<Column>,
    pub order_by: Vec<String>,
    pub deduplicate: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PartialTopic {
    pub name: String,
    pub columns: Vec<Column>,
    pub retention_period: u64,
    pub partition_count: usize,
    pub transformation_targets: Vec<TransformationTarget>,
    pub target_table: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WriteToKind {
    Stream,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteTo {
    pub kind: WriteToKind,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformationTarget {
    pub kind: WriteToKind,
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PartialIngestApi {
    pub name: String,
    pub columns: Vec<Column>,
    pub format: EndpointIngestionFormat,
    pub write_to: WriteTo,
}

#[derive(Debug, Deserialize)]
struct PartialEgressApi {
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartialInfrastructureMap {
    #[serde(default)]
    topics: HashMap<String, PartialTopic>,
    #[serde(default)]
    ingest_apis: HashMap<String, PartialIngestApi>,
    #[serde(default)]
    egress_apis: HashMap<String, PartialEgressApi>,
    #[serde(default)]
    tables: HashMap<String, PartialTable>,
    #[serde(default)]
    views: HashMap<String, View>,
    #[serde(default)]
    topic_to_table_sync_processes: HashMap<String, TopicToTableSyncProcess>,
    #[serde(default)]
    topic_to_topic_sync_processes: HashMap<String, TopicToTopicSyncProcess>,
    #[serde(default)]
    function_processes: HashMap<String, FunctionProcess>,
    block_db_processes: Option<OlapProcess>,
    consumption_api_web_server: Option<ConsumptionApiWebServer>,
    #[serde(default)]
    orchestration_workers: HashMap<String, OrchestrationWorker>,
}

#[derive(Debug, thiserror::Error)]
#[error("Failed to load Data Model V2")]
#[non_exhaustive]
pub enum DmV2LoadingError {
    Tokio(#[from] tokio::io::Error),
    #[error("Error collecting Moose resources from {user_code_file_name}:\n{message}")]
    StdErr {
        user_code_file_name: String,
        message: String,
    },
    JsonParsing(#[from] serde_json::Error),
    #[error("{message}")]
    Other {
        message: String,
    },
}

impl PartialInfrastructureMap {
    pub async fn from_subprocess(
        process: Child,
        user_code_file_name: &str,
    ) -> Result<PartialInfrastructureMap, DmV2LoadingError> {
        let mut stdout = process
            .stdout
            .unwrap_or_else(|| panic!("Process did not have a handle to stdout"));

        let mut stderr = process
            .stderr
            .unwrap_or_else(|| panic!("Process did not have a handle to stderr"));

        let mut raw_string_stderr: String = String::new();
        stderr.read_to_string(&mut raw_string_stderr).await?;

        if !raw_string_stderr.is_empty() {
            Err(DmV2LoadingError::StdErr {
                user_code_file_name: user_code_file_name.to_string(),
                message: raw_string_stderr,
            })
        } else {
            let mut raw_string_stdout: String = String::new();
            stdout.read_to_string(&mut raw_string_stdout).await?;

            let output_format = || DmV2LoadingError::Other {
                message: "invalid output format".to_string(),
            };

            let json = raw_string_stdout
                .split("___MOOSE_STUFF___start")
                .nth(1)
                .ok_or_else(output_format)?
                .split("end___MOOSE_STUFF___")
                .next()
                .ok_or_else(output_format)?;
            log::debug!("load_from_user_code inframap json: {}", json);

            Ok(serde_json::from_str(json)
                .inspect_err(|_| debug!("Invalid JSON from exports: {}", raw_string_stdout))?)
        }
    }

    fn into_infra_map(self, language: SupportedLanguages) -> InfrastructureMap {
        let tables = self.convert_tables();
        let topics = self.convert_topics();
        let api_endpoints = self.convert_api_endpoints(language, &topics);
        let topic_to_table_sync_processes = self.create_topic_to_table_sync_processes(&topics);
        let function_processes = self.create_function_processes(language, &topics);

        InfrastructureMap {
            topics,
            api_endpoints,
            tables,
            views: self.views,
            topic_to_table_sync_processes,
            topic_to_topic_sync_processes: self.topic_to_topic_sync_processes,
            function_processes,
            block_db_processes: self.block_db_processes.unwrap_or(OlapProcess {}),
            consumption_api_web_server: self
                .consumption_api_web_server
                .unwrap_or(ConsumptionApiWebServer {}),
            orchestration_workers: self.orchestration_workers,
        }
    }

    fn convert_tables(&self) -> HashMap<String, Table> {
        self.tables
            .iter()
            .map(|(id, partial_table)| {
                let table = Table {
                    name: partial_table.name.clone(),
                    columns: partial_table.columns.clone(),
                    order_by: partial_table.order_by.clone(),
                    deduplicate: partial_table.deduplicate,
                    // TODO pass through version from the TS / PY api
                    version: Version::from_string("0.0".to_string()),
                    source_primitive: PrimitiveSignature {
                        name: partial_table.name.clone(),
                        primitive_type: PrimitiveTypes::DataModel,
                    },
                };
                (id.clone(), table)
            })
            .collect()
    }

    fn convert_topics(&self) -> HashMap<String, Topic> {
        self.topics
            .values()
            .map(|partial_topic| {
                let topic = Topic {
                    name: partial_topic.name.clone(),
                    columns: partial_topic.columns.clone(),
                    max_message_bytes: DEFAULT_MAX_MESSAGE_BYTES,
                    retention_period: std::time::Duration::from_secs(
                        partial_topic.retention_period,
                    ),
                    partition_count: partial_topic.partition_count,
                    // TODO pass through version from the TS / PY api
                    version: Version::from_string("0.0".to_string()),
                    source_primitive: PrimitiveSignature {
                        name: partial_topic.name.clone(),
                        primitive_type: PrimitiveTypes::DataModel,
                    },
                };
                // TODO pass through version from the TS / PY api
                (format!("{}_0_0", partial_topic.name), topic)
            })
            .collect()
    }

    fn convert_api_endpoints(
        &self,
        language: SupportedLanguages,
        topics: &HashMap<String, Topic>,
    ) -> HashMap<String, ApiEndpoint> {
        let mut api_endpoints = HashMap::new();

        for partial_api in self.ingest_apis.values() {
            let target_topic_name = match &partial_api.write_to.kind {
                WriteToKind::Stream => partial_api.write_to.name.clone(),
            };

            let not_found = &format!("Target topic '{}' not found", target_topic_name);
            let target_topic = topics
                .values()
                .find(|topic| topic.name == target_topic_name)
                .expect(not_found);

            let data_model = crate::framework::data_model::model::DataModel {
                name: partial_api.name.clone(),
                // TODO pass through version from the TS / PY api
                version: Version::from_string("0.0".to_string()),
                config: crate::framework::data_model::config::DataModelConfig {
                    ingestion: crate::framework::data_model::config::IngestionConfig {
                        format: partial_api.format,
                    },
                    // TODO pass through parallelism from the TS / PY api
                    storage: crate::framework::data_model::config::StorageConfig {
                        enabled: true,
                        order_by_fields: vec![],
                        deduplicate: false,
                        name: None,
                    },
                    // TODO pass through parallelism from the TS / PY api
                    parallelism: 1,
                },
                columns: partial_api.columns.clone(),
                // If this is the app directory, we should use the project reference so that
                // if we rename the app folder we don't have to fish for references
                abs_file_path: std::env::current_dir()
                    .unwrap_or_default()
                    .join("app")
                    // The convention for py should be main no?
                    .join(format!("index.{}", language.extension())),
            };

            let api_endpoint = ApiEndpoint {
                name: partial_api.name.clone(),
                api_type: APIType::INGRESS {
                    target_topic_id: target_topic.id(),
                    data_model: Some(data_model),
                    format: partial_api.format,
                },
                path: PathBuf::from(format!("ingest/{}", partial_api.name)),
                method: Method::POST,
                // TODO pass through version from the TS / PY api
                version: Version::from_string("0.0".to_string()),
                source_primitive: PrimitiveSignature {
                    name: partial_api.name.clone(),
                    primitive_type: PrimitiveTypes::DataModel,
                },
            };

            let key = format!("INGRESS_{}", partial_api.name);

            api_endpoints.insert(key, api_endpoint);
        }

        for partial_api in self.egress_apis.values() {
            let api_endpoint = ApiEndpoint {
                name: partial_api.name.clone(),
                api_type: APIType::EGRESS {
                    query_params: vec![],
                    output_schema: serde_json::Value::Null,
                },
                path: PathBuf::from(partial_api.name.clone()),
                method: Method::GET,
                // TODO pass through version from the TS / PY api
                version: Version::from_string("0.0".to_string()),
                source_primitive: PrimitiveSignature {
                    name: partial_api.name.clone(),
                    primitive_type: PrimitiveTypes::ConsumptionAPI,
                },
            };

            let key = format!("EGRESS_{}", partial_api.name);
            api_endpoints.insert(key, api_endpoint);
        }

        api_endpoints
    }

    fn create_topic_to_table_sync_processes(
        &self,
        topics: &HashMap<String, Topic>,
    ) -> HashMap<String, TopicToTableSyncProcess> {
        let mut sync_processes = self.topic_to_table_sync_processes.clone();

        for (topic_name, partial_topic) in &self.topics {
            if let Some(target_table) = &partial_topic.target_table {
                let not_found = &format!("Source topic '{}' not found", topic_name);
                let source_topic = topics
                    .values()
                    .find(|topic| &topic.name == topic_name)
                    .expect(not_found);
                let source_topic_id = source_topic.id();

                let sync_id = format!("{}_{}", source_topic_id, target_table);

                let sync_process = TopicToTableSyncProcess {
                    source_topic_id,
                    target_table_id: target_table.to_string(),
                    columns: partial_topic.columns.clone(),
                    // TODO pass through version from the TS / PY api
                    version: Version::from_string("0.0".to_string()),
                    source_primitive: PrimitiveSignature {
                        name: target_table.to_string(),
                        primitive_type: PrimitiveTypes::DataModel,
                    },
                };

                sync_processes.insert(sync_id.clone(), sync_process);
                log::info!("<dmv2> Created topic_to_table_sync_processes {}", sync_id);
            } else {
                log::info!(
                    "<dmv2> Topic {} has no target_table specified, skipping sync process creation",
                    partial_topic.name
                );
            }
        }

        sync_processes
    }

    fn create_function_processes(
        &self,
        language: SupportedLanguages,
        topics: &HashMap<String, Topic>,
    ) -> HashMap<String, FunctionProcess> {
        let mut function_processes = self.function_processes.clone();

        for (topic_name, source_partial_topic) in &self.topics {
            if source_partial_topic.transformation_targets.is_empty() {
                continue;
            }

            let not_found = &format!("Source topic '{}' not found", topic_name);
            let source_topic = topics
                .values()
                .find(|topic| &topic.name == topic_name)
                .expect(not_found);

            for transformation_target in &source_partial_topic.transformation_targets {
                let process_id = format!("{}_{}", topic_name, transformation_target.name);

                let not_found = &format!("Target topic '{}' not found", transformation_target.name);
                let target_topic = topics
                    .values()
                    .find(|topic| topic.name == transformation_target.name)
                    .expect(not_found);

                let function_process = FunctionProcess {
                    name: process_id.clone(),
                    source_topic_id: source_topic.id(),
                    target_topic_id: target_topic.id(),
                    executable: std::env::current_dir()
                        .unwrap_or_default()
                        .join("app")
                        .join(format!("index.{}", language.extension())),
                    parallel_process_count: target_topic.partition_count,
                    // TODO pass through version from the TS / PY api
                    version: "0.0".to_string(),
                    source_primitive: PrimitiveSignature {
                        name: topic_name.clone(),
                        primitive_type: PrimitiveTypes::DataModel,
                    },
                };

                function_processes.insert(process_id.clone(), function_process);
            }
        }

        function_processes
    }
}

/// Computes the detailed differences between two table versions
///
/// This function performs a column-by-column comparison between two tables
/// and identifies added, removed, and modified columns. For modified columns,
/// it logs the specific attributes that have changed.
///
/// # Arguments
/// * `before` - The original table
/// * `after` - The modified table
///
/// # Returns
/// A vector of `ColumnChange` objects describing the differences
pub fn compute_table_diff(before: &Table, after: &Table) -> Vec<ColumnChange> {
    log::debug!("Computing detailed diff for table '{}'", before.name);
    let mut diff = Vec::new();

    // Check for added or modified columns
    for after_col in &after.columns {
        match before.columns.iter().find(|c| c.name == after_col.name) {
            // If the column is in the before table, but different, then it is modified
            Some(before_col) if before_col != after_col => {
                log::trace!("Column '{}' has been modified", before_col.name);
                // Log specific differences
                if before_col.data_type != after_col.data_type {
                    log::trace!(
                        "  - Type changed: {:?} -> {:?}",
                        before_col.data_type,
                        after_col.data_type
                    );
                }
                if before_col.required != after_col.required {
                    log::trace!(
                        "  - Required changed: {} -> {}",
                        before_col.required,
                        after_col.required
                    );
                }
                if before_col.unique != after_col.unique {
                    log::trace!(
                        "  - Unique changed: {} -> {}",
                        before_col.unique,
                        after_col.unique
                    );
                }
                if before_col.primary_key != after_col.primary_key {
                    log::trace!(
                        "  - Primary key changed: {} -> {}",
                        before_col.primary_key,
                        after_col.primary_key
                    );
                }
                if before_col.default != after_col.default {
                    log::trace!(
                        "  - Default value changed: {:?} -> {:?}",
                        before_col.default,
                        after_col.default
                    );
                }

                diff.push(ColumnChange::Updated {
                    before: before_col.clone(),
                    after: after_col.clone(),
                });
            }
            // If the column is not in the before table, then it is added
            None => {
                log::trace!(
                    "Column '{}' has been added with type {:?}",
                    after_col.name,
                    after_col.data_type
                );
                diff.push(ColumnChange::Added(after_col.clone()));
            }
            _ => {
                log::trace!("Column '{}' unchanged", after_col.name);
            }
        }
    }

    // Check for dropped columns
    for before_col in &before.columns {
        if !after.columns.iter().any(|c| c.name == before_col.name) {
            log::trace!("Column '{}' has been removed", before_col.name);
            diff.push(ColumnChange::Removed(before_col.clone()));
        }
    }

    log::debug!(
        "Found {} column changes for table '{}'",
        diff.len(),
        before.name
    );
    diff
}

impl Default for InfrastructureMap {
    /// Creates a default empty infrastructure map
    ///
    /// This creates an infrastructure map with empty collections for all component types.
    /// Useful for testing or as a starting point for building a map programmatically.
    ///
    /// # Returns
    /// An empty infrastructure map
    fn default() -> Self {
        Self {
            topics: HashMap::new(),
            api_endpoints: HashMap::new(),
            tables: HashMap::new(),
            views: HashMap::new(),
            topic_to_table_sync_processes: HashMap::new(),
            topic_to_topic_sync_processes: HashMap::new(),
            function_processes: HashMap::new(),
            block_db_processes: OlapProcess {},
            consumption_api_web_server: ConsumptionApiWebServer {},
            orchestration_workers: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use crate::framework::versions::Version;
    use crate::{
        framework::{
            core::{
                infrastructure::table::{Column, ColumnType, Table},
                infrastructure_map::{
                    compute_table_diff, ColumnChange, PrimitiveSignature, PrimitiveTypes,
                },
                primitive_map::PrimitiveMap,
            },
            data_model::model::DataModel,
            languages::SupportedLanguages,
        },
        project::Project,
    };

    #[tokio::test]
    #[ignore]
    async fn test_infra_map() {
        let project = Project::new(
            Path::new("/Users/nicolas/code/514/test"),
            "test".to_string(),
            SupportedLanguages::Typescript,
        );
        let primitive_map = PrimitiveMap::load(&project).await;
        println!("{:?}", primitive_map);
        assert!(primitive_map.is_ok());

        let infra_map = super::InfrastructureMap::new(&project, primitive_map.unwrap());
        println!("{:?}", infra_map);
    }

    #[tokio::test]
    #[ignore]
    async fn test_infra_diff_map() {
        let project = Project::new(
            Path::new("/Users/nicolas/code/514/test"),
            "test".to_string(),
            SupportedLanguages::Typescript,
        );
        let primitive_map = PrimitiveMap::load(&project).await.unwrap();
        let mut new_target_primitive_map = primitive_map.clone();

        let data_model_name = "test";
        let data_model_version = "1.0.0";

        let new_data_model = DataModel {
            name: data_model_name.to_string(),
            version: Version::from_string(data_model_version.to_string()),
            config: Default::default(),
            columns: vec![],
            abs_file_path: PathBuf::new(),
        };
        // Making some changes to the map
        new_target_primitive_map
            .datamodels
            .add(new_data_model)
            .unwrap();

        new_target_primitive_map
            .datamodels
            .remove(data_model_name, data_model_version);

        println!("Base Primitive Map: {:?} \n", primitive_map);
        println!("Target Primitive Map {:?} \n", new_target_primitive_map);

        let infra_map = super::InfrastructureMap::new(&project, primitive_map);
        let new_infra_map = super::InfrastructureMap::new(&project, new_target_primitive_map);

        let diffs = infra_map.diff(&new_infra_map);

        print!("Diffs: {:?}", diffs);
    }

    #[test]
    fn test_compute_table_diff() {
        let before = Table {
            name: "test_table".to_string(),
            deduplicate: false,
            columns: vec![
                Column {
                    name: "id".to_string(),
                    data_type: ColumnType::Int,
                    required: true,
                    unique: true,
                    primary_key: true,
                    default: None,
                },
                Column {
                    name: "name".to_string(),
                    data_type: ColumnType::String,
                    required: true,
                    unique: false,
                    primary_key: false,
                    default: None,
                },
                Column {
                    name: "to_be_removed".to_string(),
                    data_type: ColumnType::String,
                    required: false,
                    unique: false,
                    primary_key: false,
                    default: None,
                },
            ],
            order_by: vec!["id".to_string()],
            version: Version::from_string("1.0".to_string()),
            source_primitive: PrimitiveSignature {
                name: "test_primitive".to_string(),
                primitive_type: PrimitiveTypes::DataModel,
            },
        };

        let after = Table {
            name: "test_table".to_string(),
            deduplicate: false,
            columns: vec![
                Column {
                    name: "id".to_string(),
                    data_type: ColumnType::BigInt, // Changed type
                    required: true,
                    unique: true,
                    primary_key: true,
                    default: None,
                },
                Column {
                    name: "name".to_string(),
                    data_type: ColumnType::String,
                    required: true,
                    unique: false,
                    primary_key: false,
                    default: None,
                },
                Column {
                    name: "age".to_string(), // New column
                    data_type: ColumnType::Int,
                    required: false,
                    unique: false,
                    primary_key: false,
                    default: None,
                },
            ],
            order_by: vec!["id".to_string(), "name".to_string()], // Changed order_by
            version: Version::from_string("1.1".to_string()),
            source_primitive: PrimitiveSignature {
                name: "test_primitive".to_string(),
                primitive_type: PrimitiveTypes::DataModel,
            },
        };

        let diff = compute_table_diff(&before, &after);

        assert_eq!(diff.len(), 3);
        assert!(
            matches!(&diff[0], ColumnChange::Updated { before, after } if before.name == "id" && matches!(after.data_type, ColumnType::BigInt))
        );
        assert!(matches!(&diff[1], ColumnChange::Added(col) if col.name == "age"));
        assert!(matches!(&diff[2], ColumnChange::Removed(col) if col.name == "to_be_removed"));
    }
}

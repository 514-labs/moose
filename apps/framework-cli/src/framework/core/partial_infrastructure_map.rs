//! Partial Infrastructure Map Module
//!
//! This module provides functionality for loading and converting infrastructure definitions from user code
//! into a complete infrastructure map. It serves as a bridge between user-defined infrastructure specifications
//! (typically written in TypeScript or Python) and the internal Rust representation used by the framework.
//!
//! # Key Components
//!
//! * [`PartialInfrastructureMap`] - The main structure that represents a partially defined infrastructure
//! * [`PartialTable`], [`PartialTopic`], [`PartialIngestApi`], [`PartialEgressApi`] - Components for different infrastructure elements
//! * [`DmV2LoadingError`] - Error type for handling failures during infrastructure loading
//!
//! # Usage
//!
//! The module is primarily used during the framework's initialization phase to:
//! 1. Load infrastructure definitions from user code
//! 2. Validate and transform these definitions
//! 3. Create a complete infrastructure map for the framework to use
//!
//! # Example
//!
//! ```no_run
//! use framework_cli::framework::core::partial_infrastructure_map::PartialInfrastructureMap;
//! use tokio::process::Child;
//! use std::path::Path;
//!
//! async fn load_infrastructure(process: Child, file_name: &str) {
//!     let partial_map = PartialInfrastructureMap::from_subprocess(process, file_name).await.unwrap();
//!     let complete_map = partial_map.into_infra_map(
//!         SupportedLanguages::TypeScript,
//!         Path::new("main.ts")
//!     );
//! }
//! ```

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use log::debug;
use serde::{Deserialize, Serialize};
use tokio::process::Child;

use crate::{
    framework::{
        consumption::model::ConsumptionQueryParam, data_model::config::EndpointIngestionFormat,
        languages::SupportedLanguages, versions::Version,
    },
    utilities::constants,
};

use super::{
    infrastructure::{
        api_endpoint::{APIType, ApiEndpoint, Method},
        consumption_webserver::ConsumptionApiWebServer,
        function_process::FunctionProcess,
        olap_process::OlapProcess,
        orchestration_worker::OrchestrationWorker,
        table::{Column, Table},
        topic::{Topic, DEFAULT_MAX_MESSAGE_BYTES},
        topic_sync_process::{TopicToTableSyncProcess, TopicToTopicSyncProcess},
        view::View,
    },
    infrastructure_map::{InfrastructureMap, PrimitiveSignature, PrimitiveTypes, SqlResource},
};

/// Represents a table definition from user code before it's converted into a complete [`Table`].
///
/// This structure captures the essential properties needed to create a table in the infrastructure,
/// including column definitions, ordering, and deduplication settings.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PartialTable {
    pub name: String,
    pub columns: Vec<Column>,
    pub order_by: Vec<String>,
    pub deduplicate: bool,
    pub engine: Option<String>,
    pub version: Option<String>,
}

/// Represents a topic definition from user code before it's converted into a complete [`Topic`].
///
/// Topics are message queues that can be used for streaming data between different parts of the system.
/// They can have multiple consumers and transformation targets.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PartialTopic {
    pub name: String,
    pub columns: Vec<Column>,
    pub retention_period: u64,
    pub partition_count: usize,
    pub transformation_targets: Vec<TransformationTarget>,
    pub target_table: Option<String>,
    pub target_table_version: Option<String>,
    pub version: Option<String>,
    pub consumers: Vec<Consumer>,
}

/// Specifies the type of destination for write operations.
///
/// Currently only supports stream destinations, but could be extended for other types
/// of write targets in the future.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WriteToKind {
    /// Indicates that data should be written to a stream (topic)
    Stream,
}

/// Represents an ingestion API endpoint definition before conversion to a complete [`ApiEndpoint`].
///
/// Ingestion APIs are HTTP endpoints that accept data and write it to a specified destination
/// (typically a topic).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PartialIngestApi {
    pub name: String,
    pub columns: Vec<Column>,
    pub format: EndpointIngestionFormat,
    pub write_to: WriteTo,
    pub version: Option<String>,
}

/// Represents an egress API endpoint definition before conversion to a complete [`ApiEndpoint`].
///
/// Egress APIs are HTTP endpoints that allow consumers to query and retrieve data from the system.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PartialEgressApi {
    pub name: String,
    pub query_params: Vec<Column>,
    pub response_schema: serde_json::Value,
    pub version: Option<String>,
}

/// Specifies a write destination for data ingestion.
///
/// Contains both the type of destination and its identifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteTo {
    pub kind: WriteToKind,
    pub name: String,
}

/// Specifies a transformation target for topic data.
///
/// Used to define where transformed data should be written and optionally specify a version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformationTarget {
    pub kind: WriteToKind,
    pub name: String,
    pub version: Option<String>,
}

/// Configuration for a topic consumer.
///
/// Currently only contains version information but could be extended with additional
/// consumer-specific configuration in the future.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Consumer {
    pub version: Option<String>,
}

/// Errors that can occur during the loading of Data Model V2 infrastructure definitions.
///
/// This error type follows the Rust error handling best practices and provides
/// specific error variants for different failure modes.
#[derive(Debug, thiserror::Error)]
#[error("Failed to load Data Model V2")]
#[non_exhaustive]
pub enum DmV2LoadingError {
    /// Errors from Tokio async I/O operations
    Tokio(#[from] tokio::io::Error),

    /// Errors when collecting Moose resources from user code
    #[error("Error collecting Moose resources from {user_code_file_name}:\n{message}")]
    StdErr {
        user_code_file_name: String,
        message: String,
    },

    /// JSON parsing errors
    JsonParsing(#[from] serde_json::Error),

    /// Catch-all for other types of errors
    #[error("{message}")]
    Other { message: String },
}

/// Represents a partial infrastructure map loaded from user code.
///
/// This structure is the main entry point for loading and converting infrastructure
/// definitions from user code into the framework's internal representation.
///
/// # Loading Process
///
/// 1. User code is executed in a subprocess
/// 2. The subprocess outputs JSON describing the infrastructure
/// 3. The JSON is parsed into this structure
/// 4. The structure is converted into a complete [`InfrastructureMap`]
///
/// # Fields
///
/// All fields are optional HashMaps containing partial definitions for different
/// infrastructure components. During conversion to a complete map, these partial
/// definitions are validated and transformed into their complete counterparts.
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
    sql_resources: HashMap<String, SqlResource>,
    #[serde(default)]
    topic_to_table_sync_processes: HashMap<String, TopicToTableSyncProcess>,
    #[serde(default)]
    topic_to_topic_sync_processes: HashMap<String, TopicToTopicSyncProcess>,
    #[serde(default)]
    function_processes: HashMap<String, FunctionProcess>,
    block_db_processes: Option<OlapProcess>,
    consumption_api_web_server: Option<ConsumptionApiWebServer>,
}

impl PartialInfrastructureMap {
    /// Creates a new [`PartialInfrastructureMap`] by executing and reading from a subprocess.
    ///
    /// This method is used to load infrastructure definitions from user code written in languages
    /// like TypeScript or Python. The subprocess is expected to output JSON in a specific format
    /// that can be parsed into a [`PartialInfrastructureMap`].
    ///
    /// # Arguments
    ///
    /// * `process` - The subprocess that will output the infrastructure definition
    /// * `user_code_file_name` - Name of the file containing the user's code
    ///
    /// # Errors
    ///
    /// Returns a [`DmV2LoadingError`] if:
    /// * The subprocess fails to execute
    /// * The subprocess output cannot be parsed
    /// * Required dependencies are missing
    /// * The output format is invalid
    pub async fn from_subprocess(
        process: Child,
        user_code_file_name: &str,
    ) -> Result<PartialInfrastructureMap, DmV2LoadingError> {
        let output = process.wait_with_output().await?;

        // needs from_utf8_lossy_owned
        let raw_string_stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !raw_string_stderr.is_empty() {
            let error_message = if raw_string_stderr.contains("MODULE_NOT_FOUND")
                || raw_string_stderr.contains("ModuleNotFoundError")
            {
                let install_command = if user_code_file_name
                    .ends_with(constants::TYPESCRIPT_FILE_EXTENSION)
                {
                    "npm install"
                } else if user_code_file_name.ends_with(constants::PYTHON_FILE_EXTENSION) {
                    "pip install ."
                } else {
                    return Err(DmV2LoadingError::Other {
                        message: format!("Unsupported file extension in: {}", user_code_file_name),
                    });
                };

                format!("Missing dependencies detected. Please run '{}' and try again.\nOriginal error: {}", 
                    install_command,
                    raw_string_stderr
                )
            } else {
                raw_string_stderr
            };

            Err(DmV2LoadingError::StdErr {
                user_code_file_name: user_code_file_name.to_string(),
                message: error_message,
            })
        } else {
            let raw_string_stdout: String = String::from_utf8_lossy(&output.stdout).to_string();

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
            log::info!("load_from_user_code inframap json: {}", json);

            Ok(serde_json::from_str(json)
                .inspect_err(|_| debug!("Invalid JSON from exports: {}", raw_string_stdout))?)
        }
    }

    /// Converts this partial infrastructure map into a complete [`InfrastructureMap`].
    ///
    /// This method performs the final transformation of user-defined infrastructure components
    /// into their complete, validated forms. It ensures all references between components are
    /// valid and sets up the necessary processes and workers.
    ///
    /// # Arguments
    ///
    /// * `language` - The programming language of the user's code
    /// * `main_file` - Path to the main file containing the user's code
    ///
    /// # Returns
    ///
    /// Returns a complete [`InfrastructureMap`] containing all the validated and transformed
    /// infrastructure components.
    pub fn into_infra_map(
        self,
        language: SupportedLanguages,
        main_file: &Path,
    ) -> InfrastructureMap {
        let tables = self.convert_tables();
        let topics = self.convert_topics();
        let api_endpoints = self.convert_api_endpoints(main_file, &topics);
        let topic_to_table_sync_processes =
            self.create_topic_to_table_sync_processes(&tables, &topics);
        let function_processes = self.create_function_processes(main_file, language, &topics);

        // Why does dmv1 InfrastructureMap::new do this?
        let mut orchestration_workers = HashMap::new();
        let orchestration_worker = OrchestrationWorker::new(language);
        orchestration_workers.insert(orchestration_worker.id(), orchestration_worker);

        InfrastructureMap {
            topics,
            api_endpoints,
            tables,
            views: self.views,
            sql_resources: self.sql_resources,
            topic_to_table_sync_processes,
            topic_to_topic_sync_processes: self.topic_to_topic_sync_processes,
            function_processes,
            block_db_processes: self.block_db_processes.unwrap_or(OlapProcess {}),
            consumption_api_web_server: self
                .consumption_api_web_server
                .unwrap_or(ConsumptionApiWebServer {}),
            orchestration_workers,
        }
    }

    /// Converts partial table definitions into complete [`Table`] instances.
    ///
    /// This method handles versioning and naming of tables, ensuring that versioned tables
    /// have appropriate suffixes in their names.
    fn convert_tables(&self) -> HashMap<String, Table> {
        self.tables
            .values()
            .map(|partial_table| {
                let version: Option<Version> = partial_table
                    .version
                    .as_ref()
                    .map(|v_str| Version::from_string(v_str.clone()));

                let table = Table {
                    // In dmv1, DataModel.to_table uses version in the name
                    name: version
                        .as_ref()
                        .map_or(partial_table.name.clone(), |version| {
                            format!("{}_{}", partial_table.name, version.as_suffix())
                        }),
                    columns: partial_table.columns.clone(),
                    order_by: partial_table.order_by.clone(),
                    deduplicate: partial_table.deduplicate,
                    engine: partial_table.engine.clone(),
                    version,
                    source_primitive: PrimitiveSignature {
                        name: partial_table.name.clone(),
                        primitive_type: PrimitiveTypes::DataModel,
                    },
                };
                (table.id(), table)
            })
            .collect()
    }

    /// Converts partial topic definitions into complete [`Topic`] instances.
    ///
    /// Creates topics with appropriate retention periods, partition counts, and other
    /// configuration settings.
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
                    version: partial_topic
                        .version
                        .as_ref()
                        .map(|v_str| Version::from_string(v_str.clone())),
                    source_primitive: PrimitiveSignature {
                        name: partial_topic.name.clone(),
                        primitive_type: PrimitiveTypes::DataModel,
                    },
                };
                (topic.id(), topic)
            })
            .collect()
    }

    /// Converts partial API endpoint definitions into complete [`ApiEndpoint`] instances.
    ///
    /// Handles both ingestion and egress API endpoints, setting up appropriate paths,
    /// methods, and data models.
    ///
    /// # Arguments
    ///
    /// * `main_file` - Path to the main file containing the user's code
    /// * `topics` - Map of available topics that API endpoints might reference
    fn convert_api_endpoints(
        &self,
        main_file: &Path,
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

            // TODO: Remove data model from api endpoints when dmv1 is removed
            let data_model = crate::framework::data_model::model::DataModel {
                name: partial_api.name.clone(),
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
                abs_file_path: main_file.to_path_buf(),
            };

            let api_endpoint = ApiEndpoint {
                name: partial_api.name.clone(),
                api_type: APIType::INGRESS {
                    target_topic_id: target_topic.id(),
                    data_model: Some(data_model),
                    format: partial_api.format,
                },
                path: PathBuf::from_iter(
                    [
                        "ingest",
                        &partial_api.name,
                        partial_api.version.as_deref().unwrap_or_default(),
                    ]
                    .into_iter()
                    .filter(|s| !s.is_empty()),
                ),
                method: Method::POST,
                version: partial_api
                    .version
                    .as_ref()
                    .map(|v_str| Version::from_string(v_str.clone())),
                source_primitive: PrimitiveSignature {
                    name: partial_api.name.clone(),
                    primitive_type: PrimitiveTypes::DataModel,
                },
            };

            api_endpoints.insert(api_endpoint.id(), api_endpoint);
        }

        for partial_api in self.egress_apis.values() {
            let api_endpoint = ApiEndpoint {
                name: partial_api.name.clone(),
                api_type: APIType::EGRESS {
                    query_params: partial_api
                        .query_params
                        .iter()
                        .map(|column| ConsumptionQueryParam {
                            name: column.name.clone(),
                            data_type: column.data_type.clone(),
                            required: column.required,
                        })
                        .collect(),
                    output_schema: partial_api.response_schema.clone(),
                },
                path: PathBuf::from(partial_api.name.clone()),
                method: Method::GET,
                version: partial_api
                    .version
                    .as_ref()
                    .map(|v_str| Version::from_string(v_str.clone())),
                source_primitive: PrimitiveSignature {
                    name: partial_api.name.clone(),
                    primitive_type: PrimitiveTypes::ConsumptionAPI,
                },
            };

            api_endpoints.insert(api_endpoint.id(), api_endpoint);
        }

        api_endpoints
    }

    /// Creates synchronization processes between topics and tables.
    ///
    /// These processes ensure that data from topics is properly synchronized to their
    /// target tables, respecting versioning and other configuration settings.
    ///
    /// # Arguments
    ///
    /// * `tables` - Map of available tables
    /// * `topics` - Map of available topics
    fn create_topic_to_table_sync_processes(
        &self,
        tables: &HashMap<String, Table>,
        topics: &HashMap<String, Topic>,
    ) -> HashMap<String, TopicToTableSyncProcess> {
        let mut sync_processes = self.topic_to_table_sync_processes.clone();

        for (topic_name, partial_topic) in &self.topics {
            if let Some(target_table_name) = &partial_topic.target_table {
                let topic_not_found = &format!("Source topic '{}' not found", topic_name);
                let source_topic = topics
                    .values()
                    .find(|topic| &topic.name == topic_name)
                    .expect(topic_not_found);

                let target_table_version: Option<Version> = partial_topic
                    .target_table_version
                    .as_ref()
                    .map(|v_str| Version::from_string(v_str.clone()));

                let table_not_found = &format!(
                    "Target table '{}' version '{:?}' not found",
                    target_table_name, target_table_version
                );
                let target_table = tables
                    .values()
                    .find(|table| {
                        let name_matches = table.name.starts_with(target_table_name);
                        let version_matches = match &target_table_version {
                            Some(target_v) => table.version.as_ref() == Some(target_v),
                            None => true,
                        };
                        name_matches && version_matches
                    })
                    .expect(table_not_found);

                let sync_process = TopicToTableSyncProcess::new(source_topic, target_table);
                let sync_id = sync_process.id();
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

    /// Creates function processes for transformations and consumers.
    ///
    /// Function processes handle data transformations between topics and process
    /// data for consumers. This method sets up the necessary processes with
    /// appropriate parallelism and versioning.
    ///
    /// # Arguments
    ///
    /// * `main_file` - Path to the main file containing the user's code
    /// * `language` - The programming language of the user's code
    /// * `topics` - Map of available topics
    fn create_function_processes(
        &self,
        main_file: &Path,
        language: SupportedLanguages,
        topics: &HashMap<String, Topic>,
    ) -> HashMap<String, FunctionProcess> {
        let mut function_processes = self.function_processes.clone();

        for (topic_name, source_partial_topic) in &self.topics {
            debug!(
                "source_partial_topic: {:?} with name {}",
                source_partial_topic, topic_name
            );

            let not_found = &format!("Source topic '{}' not found", topic_name);
            let source_topic = topics
                .values()
                .find(|topic| &topic.name == topic_name)
                .expect(not_found);

            for transformation_target in &source_partial_topic.transformation_targets {
                debug!("transformation_target: {:?}", transformation_target);

                // In dmv1, the process name was the file name which had double underscores
                let process_name = format!("{}__{}", topic_name, transformation_target.name);

                let not_found = &format!("Target topic '{}' not found", transformation_target.name);
                let target_topic = topics
                    .values()
                    .find(|topic| topic.name == transformation_target.name)
                    .expect(not_found);

                let function_process = FunctionProcess {
                    name: process_name.clone(),
                    source_topic_id: source_topic.id(),
                    target_topic_id: Some(target_topic.id()),
                    executable: main_file.to_path_buf(),
                    language,
                    parallel_process_count: target_topic.partition_count,
                    version: transformation_target.version.clone(),
                    source_primitive: PrimitiveSignature {
                        name: process_name.clone(),
                        primitive_type: PrimitiveTypes::Function,
                    },
                };

                function_processes.insert(function_process.id(), function_process);
            }

            for consumer in &source_partial_topic.consumers {
                let function_process = FunctionProcess {
                    // In dmv1, consumer process has the id format!("{}_{}_{}", self.name, self.source_topic_id, self.version)
                    name: topic_name.clone(),
                    source_topic_id: source_topic.id(),
                    target_topic_id: None,
                    executable: main_file.to_path_buf(),
                    language,
                    parallel_process_count: source_partial_topic.partition_count,
                    version: consumer.version.clone(),
                    source_primitive: PrimitiveSignature {
                        name: topic_name.clone(),
                        primitive_type: PrimitiveTypes::DataModel,
                    },
                };

                function_processes.insert(function_process.id(), function_process);
            }
        }

        function_processes
    }
}

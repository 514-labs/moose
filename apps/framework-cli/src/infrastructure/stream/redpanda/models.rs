use rdkafka::{metadata::MetadataTopic, producer::FutureProducer};
use serde::{Deserialize, Serialize};

use crate::framework::core::infrastructure::topic::Topic;

use super::constants::NAMESPACE_SEPARATOR;

/// RedpandaStreamConfig represents the configuration for a Redpanda/Kafka topic.
///
/// This struct contains all necessary configuration parameters for working with a
/// Redpanda topic, including name, partitions, retention period, message size limits,
/// and namespace information.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RedpandaStreamConfig {
    /// The name of the stream, this includes the namespace if it exists
    pub name: String,
    /// The number of partitions for the stream
    pub partitions: usize,
    /// The retention period for the stream in milliseconds
    pub retention_ms: u128,
    /// The maximum message size for the stream in bytes
    pub max_message_bytes: usize,
    /// The namespace for the stream, if any
    pub namespace: Option<String>,
}

impl RedpandaStreamConfig {
    /// Extracts the topic name without the namespace prefix.
    ///
    /// If the topic name includes a namespace separator, returns the part after the separator.
    /// Otherwise, returns the full name.
    ///
    /// # Returns
    /// * A String containing the topic name without the namespace
    pub fn get_topic_name_without_namespace(&self) -> String {
        if let Some(pos) = self.name.find(NAMESPACE_SEPARATOR) {
            self.name[pos + 1..].to_string()
        } else {
            self.name.to_owned()
        }
    }

    /// Creates a RedpandaStreamConfig from a core Topic model.
    ///
    /// This function converts a domain model Topic to its Redpanda-specific representation,
    /// applying namespace configuration if available.
    ///
    /// # Arguments
    /// * `redpanda_config` - The RedpandaConfig containing namespace information
    /// * `topic` - The core Topic domain model to convert
    ///
    /// # Returns
    /// * A new RedpandaStreamConfig instance
    pub fn from_topic(redpanda_config: &RedpandaConfig, topic: &Topic) -> Self {
        Self {
            name: topic_name(&redpanda_config.namespace, topic),
            partitions: topic.partition_count,
            retention_ms: topic.retention_period.as_millis(),
            max_message_bytes: topic.max_message_bytes,
            namespace: redpanda_config.namespace.clone(),
        }
    }

    /// Creates a RedpandaStreamConfig from Kafka metadata.
    ///
    /// This constructor builds a RedpandaStreamConfig from metadata returned by the Kafka API,
    /// along with additional configuration parameters.
    ///
    /// # Arguments
    /// * `metadata` - The MetadataTopic from Kafka API
    /// * `retention_ms` - The retention period in milliseconds
    /// * `max_message_bytes` - The maximum message size in bytes
    ///
    /// # Returns
    /// * A new RedpandaStreamConfig instance
    pub fn from_metadata(
        metadata: &MetadataTopic,
        retention_ms: u128,
        max_message_bytes: usize,
    ) -> Self {
        let namespace = get_namespace(metadata);
        Self {
            name: metadata.name().to_string(),
            partitions: metadata.partitions().len(),
            retention_ms: retention_ms,
            max_message_bytes: max_message_bytes,
            namespace,
        }
    }

    /// Serializes the configuration to a JSON string.
    ///
    /// # Returns
    /// * A String containing the JSON representation of this configuration
    ///
    /// # Panics
    /// * Panics if serialization fails
    pub fn as_json_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    /// Deserializes a RedpandaStreamConfig from a JSON string.
    ///
    /// # Arguments
    /// * `s` - The JSON string to parse
    ///
    /// # Returns
    /// * A new RedpandaStreamConfig instance
    ///
    /// # Panics
    /// * Panics if deserialization fails
    pub fn from_json_string(s: &str) -> Self {
        serde_json::from_str(s).unwrap()
    }
}

/// Configuration for connecting to and interacting with a Redpanda/Kafka cluster.
///
/// This struct contains all necessary connection parameters and default settings
/// for working with a Redpanda cluster, including authentication, timeouts, and
/// namespace configuration.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RedpandaConfig {
    /// Broker connection string in format "host:port"
    pub broker: String,
    /// Message timeout in milliseconds
    pub message_timeout_ms: i32,
    /// Default retention period in milliseconds
    pub retention_ms: i32,
    /// Replication factor for topics (defaults to 1)
    #[serde(default = "default_replication_factor")]
    pub replication_factor: i32,
    /// SASL username for authentication, if required
    pub sasl_username: Option<String>,
    /// SASL password for authentication, if required
    pub sasl_password: Option<String>,
    /// SASL mechanism (e.g., "PLAIN", "SCRAM-SHA-256")
    pub sasl_mechanism: Option<String>,
    /// Security protocol (e.g., "SASL_SSL", "PLAINTEXT")
    pub security_protocol: Option<String>,
    /// Namespace for topic isolation
    pub namespace: Option<String>,
}

impl RedpandaConfig {
    /// Gets the namespace prefix for topics.
    ///
    /// If a namespace is configured, returns the namespace followed by a separator.
    /// Otherwise, returns an empty string.
    ///
    /// # Returns
    /// * A String containing the namespace prefix
    pub fn get_namespace_prefix(&self) -> String {
        namespace_prefix(&self.namespace)
    }

    /// Prefixes a value with the namespace.
    ///
    /// Prepends the namespace and separator to the provided value if a namespace is configured.
    ///
    /// # Arguments
    /// * `value` - The string value to prefix
    ///
    /// # Returns
    /// * A String containing the prefixed value
    pub fn prefix_with_namespace(&self, value: &str) -> String {
        format!("{}{}", self.get_namespace_prefix(), value)
    }
}

/// Default replication factor for Redpanda topics.
///
/// # Returns
/// * The default replication factor (1)
fn default_replication_factor() -> i32 {
    1
}

/// Formats a namespace prefix with the appropriate separator.
///
/// # Arguments
/// * `namespace` - Optional namespace
///
/// # Returns
/// * A String containing the namespace prefix with separator if namespace is present,
///   otherwise an empty string
fn namespace_prefix(namespace: &Option<String>) -> String {
    namespace
        .as_ref()
        .map(|ns| format!("{}{}", ns, NAMESPACE_SEPARATOR))
        .unwrap_or_default()
}

/// Extracts the namespace from a topic's metadata.
///
/// Parses the topic name to extract the namespace part before the separator.
///
/// # Arguments
/// * `topic` - The MetadataTopic to extract namespace from
///
/// # Returns
/// * Some(String) containing the namespace if present, None otherwise
fn get_namespace(topic: &MetadataTopic) -> Option<String> {
    topic
        .name()
        .split(NAMESPACE_SEPARATOR)
        .next()
        .map(|ns| ns.to_string())
}

/// Constructs a full topic name with namespace.
///
/// If a namespace is provided, formats the topic name as "{namespace}{separator}{topic.name}".
/// Otherwise, returns just the topic name.
///
/// # Arguments
/// * `namespace` - Optional namespace to prefix
/// * `topic` - The Topic to get the name from
///
/// # Returns
/// * A String containing the full topic name
fn topic_name(namespace: &Option<String>, topic: &Topic) -> String {
    if let Some(ns) = namespace {
        format!("{}{}{}", ns, NAMESPACE_SEPARATOR, topic.name)
    } else {
        topic.name.clone()
    }
}

impl Default for RedpandaConfig {
    fn default() -> Self {
        Self {
            broker: "localhost:19092".to_string(),
            message_timeout_ms: 1000,
            retention_ms: 30000,
            replication_factor: 1,
            sasl_username: None,
            sasl_password: None,
            sasl_mechanism: None,
            security_protocol: None,
            namespace: None,
        }
    }
}

/// A wrapper for a Redpanda producer and its configuration.
///
/// This struct combines a FutureProducer with its associated configuration
/// for convenience in passing both together.
#[derive(Clone)]
pub struct ConfiguredProducer {
    /// The underlying Redpanda/Kafka producer
    pub producer: FutureProducer,
    /// The configuration used to create this producer
    pub config: RedpandaConfig,
}

/// Represents a change that can be applied to a Redpanda topic.
///
/// This enum allows us to track what types of changes need to be
/// made to the Redpanda streaming infrastructure.
#[derive(Debug, Clone)]
pub enum RedpandaChange {
    /// Represents adding a new topic
    Added(RedpandaStreamConfig),
    /// Represents removing an existing topic
    Removed(RedpandaStreamConfig),
    /// Represents updating an existing topic
    Updated {
        /// The topic configuration before the update
        before: RedpandaStreamConfig,
        /// The topic configuration after the update
        after: RedpandaStreamConfig,
    },
}

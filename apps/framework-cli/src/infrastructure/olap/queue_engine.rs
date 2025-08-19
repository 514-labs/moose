use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Generic queue engine abstraction for backend-agnostic queue table support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueEngine {
    pub source: QueueSource,
    pub processing: ProcessingConfig,
    pub coordination: CoordinationConfig,
    pub monitoring: MonitoringConfig,
}

/// Queue data source configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueueSource {
    S3 {
        path: String,
        format: String,
        credentials: Option<S3Credentials>,
        /// Additional S3-specific settings
        extra_settings: HashMap<String, String>,
    },
    Azure {
        container: String,
        path: String,
        format: String,
        /// Azure-specific settings
        extra_settings: HashMap<String, String>,
    },
    // Future: GCS, Kafka, etc.
}

/// S3 credentials configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Credentials {
    pub role_arn: Option<String>,
    pub access_key_id: Option<String>,
    pub secret_access_key: Option<String>,
    pub session_token: Option<String>,
}

/// Processing behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingConfig {
    /// Processing order mode
    pub mode: ProcessingMode,
    /// What to do with files after successful processing
    pub after_processing: AfterProcessing,
    /// Number of retry attempts for failed files
    pub retries: u32,
    /// Number of processing threads (None = backend default)
    pub threads: Option<u32>,
    /// Enable parallel inserts for better throughput
    pub parallel_inserts: bool,
    /// Number of logical processing units for distributed processing
    pub buckets: Option<u32>,
}

/// File processing order mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessingMode {
    /// Files processed in lexicographic order
    Ordered,
    /// Files processed in any order with full tracking
    Unordered,
}

/// Action to take after successful file processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AfterProcessing {
    /// Keep the file in the source
    Keep,
    /// Delete the file from the source
    Delete,
}

/// Coordination and state management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationConfig {
    /// Custom coordination path (None = backend default)
    pub path: Option<String>,
    /// Maximum number of tracked files for unordered mode
    pub tracked_files_limit: Option<u32>,
    /// TTL in seconds for tracked files
    pub tracked_file_ttl_sec: Option<u32>,
    /// Minimum cleanup interval in milliseconds
    pub cleanup_interval_min_ms: Option<u32>,
    /// Maximum cleanup interval in milliseconds
    pub cleanup_interval_max_ms: Option<u32>,
}

/// Monitoring and polling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable logging to system tables
    pub enable_logging: bool,
    /// Minimum polling timeout in milliseconds
    pub polling_min_timeout_ms: Option<u32>,
    /// Maximum polling timeout in milliseconds
    pub polling_max_timeout_ms: Option<u32>,
    /// Additional backoff time when no files found
    pub polling_backoff_ms: Option<u32>,
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        Self {
            mode: ProcessingMode::Unordered, // Safer default for most use cases
            after_processing: AfterProcessing::Keep,
            retries: 0,
            threads: None,
            parallel_inserts: false,
            buckets: None,
        }
    }
}

impl Default for CoordinationConfig {
    fn default() -> Self {
        Self {
            path: None,
            tracked_files_limit: Some(1000),
            tracked_file_ttl_sec: None,
            cleanup_interval_min_ms: Some(10000),
            cleanup_interval_max_ms: Some(30000),
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enable_logging: false,
            polling_min_timeout_ms: Some(1000),
            polling_max_timeout_ms: Some(10000),
            polling_backoff_ms: Some(0),
        }
    }
}

impl ProcessingMode {
    pub fn to_string(&self) -> &'static str {
        match self {
            ProcessingMode::Ordered => "ordered",
            ProcessingMode::Unordered => "unordered",
        }
    }
}

impl AfterProcessing {
    pub fn to_string(&self) -> &'static str {
        match self {
            AfterProcessing::Keep => "keep",
            AfterProcessing::Delete => "delete",
        }
    }
}

/// Validation errors for queue engine configuration
#[derive(Debug, thiserror::Error)]
pub enum QueueEngineValidationError {
    #[error("Missing required field: {field}")]
    MissingField { field: String },
    #[error("Invalid value for {field}: {value}")]
    InvalidValue { field: String, value: String },
    #[error("Unsupported source type for backend")]
    UnsupportedSource,
    #[error("Configuration conflict: {message}")]
    ConfigurationConflict { message: String },
}

impl QueueEngine {
    /// Create a new S3Queue engine with sensible defaults
    pub fn s3_queue(path: String, format: String) -> Self {
        Self {
            source: QueueSource::S3 {
                path,
                format,
                credentials: None,
                extra_settings: HashMap::new(),
            },
            processing: ProcessingConfig::default(),
            coordination: CoordinationConfig::default(),
            monitoring: MonitoringConfig::default(),
        }
    }

    /// Validate the queue engine configuration
    pub fn validate(&self) -> Result<(), QueueEngineValidationError> {
        match &self.source {
            QueueSource::S3 { path, format, .. } => {
                if path.is_empty() {
                    return Err(QueueEngineValidationError::MissingField {
                        field: "path".to_string(),
                    });
                }
                if format.is_empty() {
                    return Err(QueueEngineValidationError::MissingField {
                        field: "format".to_string(),
                    });
                }
            }
            QueueSource::Azure { container, path, format, .. } => {
                if container.is_empty() {
                    return Err(QueueEngineValidationError::MissingField {
                        field: "container".to_string(),
                    });
                }
                if path.is_empty() {
                    return Err(QueueEngineValidationError::MissingField {
                        field: "path".to_string(),
                    });
                }
                if format.is_empty() {
                    return Err(QueueEngineValidationError::MissingField {
                        field: "format".to_string(),
                    });
                }
            }
        }

        // Validate processing config
        if matches!(self.processing.mode, ProcessingMode::Ordered) {
            if let Some(buckets) = self.processing.buckets {
                if buckets == 0 {
                    return Err(QueueEngineValidationError::InvalidValue {
                        field: "buckets".to_string(),
                        value: "0".to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    /// Get the source type as a string
    pub fn source_type(&self) -> &'static str {
        match &self.source {
            QueueSource::S3 { .. } => "s3",
            QueueSource::Azure { .. } => "azure",
        }
    }
}
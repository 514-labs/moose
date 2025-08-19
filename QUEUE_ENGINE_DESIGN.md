# Queue Engine Abstraction Design

## Problem Statement
Need to support queue-based table engines (S3Queue, AzureQueue, etc.) in a backend-agnostic way that doesn't paint us into a ClickHouse-specific corner.

## Design Principles

### 1. Backend Abstraction
- Generic `QueueEngine` concept that can map to different backends
- Backend-specific implementations handle the translation
- User-facing API remains consistent across backends

### 2. Extensibility
- Easy to add new queue sources (S3, Azure, GCS, Kafka, etc.)
- Easy to add new backends (ClickHouse, DuckDB, etc.)
- Parameter validation at the abstraction layer

### 3. Type Safety
- Strong typing for all configuration options
- Compile-time validation where possible
- Clear error messages for invalid configurations

## Proposed Architecture

```rust
// Generic queue engine abstraction
pub struct QueueEngine {
    pub source: QueueSource,
    pub processing: ProcessingConfig,
    pub coordination: CoordinationConfig,
    pub monitoring: MonitoringConfig,
}

pub enum QueueSource {
    S3 {
        path: String,
        format: String,
        credentials: Option<S3Credentials>,
        // S3-specific settings
    },
    Azure {
        container: String,
        path: String,
        format: String,
        // Azure-specific settings  
    },
    // Future: GCS, Kafka, etc.
}

pub struct ProcessingConfig {
    pub mode: ProcessingMode,
    pub after_processing: AfterProcessing,
    pub retries: u32,
    pub threads: Option<u32>,
    pub parallel_inserts: bool,
    pub buckets: Option<u32>,
}

pub enum ProcessingMode {
    Ordered,
    Unordered,
}

pub enum AfterProcessing {
    Keep,
    Delete,
}

pub struct CoordinationConfig {
    pub path: Option<String>,
    pub tracked_files_limit: Option<u32>,
    pub tracked_file_ttl_sec: Option<u32>,
    pub cleanup_interval_min_ms: Option<u32>,
    pub cleanup_interval_max_ms: Option<u32>,
}

pub struct MonitoringConfig {
    pub enable_logging: bool,
    pub polling_min_timeout_ms: Option<u32>,
    pub polling_max_timeout_ms: Option<u32>,
    pub polling_backoff_ms: Option<u32>,
}
```

## User-Facing API

```typescript
// TypeScript API
const queueTable = new QueueTable<DataModel>("raw_data", {
  source: {
    type: "s3",
    path: "s3://bucket/data/*.json",
    format: "JSONEachRow",
    credentials: { roleArn: "arn:aws:iam::..." }
  },
  processing: {
    mode: "unordered",
    afterProcessing: "keep",
    retries: 3,
    threads: 4,
    parallelInserts: true
  },
  coordination: {
    path: "/clickhouse/queue/raw_data",
    trackedFilesLimit: 1000,
    trackedFileTtlSec: 3600
  },
  monitoring: {
    enableLogging: true,
    pollingMinTimeoutMs: 1000,
    pollingMaxTimeoutMs: 10000
  }
});
```

## Backend Translation

Each backend translator converts the generic `QueueEngine` to backend-specific SQL:

```rust
trait QueueEngineTranslator {
    fn translate_to_sql(&self, engine: &QueueEngine) -> Result<String, Error>;
    fn validate_config(&self, engine: &QueueEngine) -> Result<(), ValidationError>;
}

impl QueueEngineTranslator for ClickHouseTranslator {
    fn translate_to_sql(&self, engine: &QueueEngine) -> Result<String, Error> {
        match &engine.source {
            QueueSource::S3 { path, format, .. } => {
                // Generate ClickHouse S3Queue syntax
                let mut settings = Vec::new();
                
                settings.push(format!("mode = '{}'", engine.processing.mode.to_clickhouse()));
                
                if let Some(retries) = engine.processing.retries {
                    settings.push(format!("s3queue_loading_retries = {}", retries));
                }
                
                // ... all other settings
                
                Ok(format!(
                    "ENGINE = S3Queue('{}', '{}') SETTINGS {}",
                    path, format, settings.join(", ")
                ))
            }
            _ => Err(Error::UnsupportedSource)
        }
    }
}
```

## Benefits

1. **Backend Agnostic**: Easy to add DuckDB, PostgreSQL, etc.
2. **Type Safe**: All parameters validated at compile time
3. **Extensible**: Easy to add new queue sources
4. **Maintainable**: Clear separation of concerns
5. **User Friendly**: Consistent API across backends
use crate::infrastructure::olap::queue_engine::{
    QueueEngine, QueueSource, ProcessingMode, AfterProcessing, QueueEngineValidationError
};
use super::errors::ClickhouseError;

/// ClickHouse-specific queue engine translator
pub struct ClickHouseQueueTranslator;

impl ClickHouseQueueTranslator {
    /// Translate a generic QueueEngine to ClickHouse SQL syntax
    pub fn translate_to_sql(&self, engine: &QueueEngine) -> Result<(String, Option<String>), ClickhouseError> {
        // Validate the configuration first
        engine.validate().map_err(|e| ClickhouseError::InvalidParameters {
            message: format!("Queue engine validation failed: {}", e),
        })?;

        match &engine.source {
            QueueSource::S3 { path, format, credentials, extra_settings } => {
                self.translate_s3_queue(engine, path, format, credentials.as_ref(), extra_settings)
            }
            QueueSource::Azure { container, path, format, extra_settings } => {
                self.translate_azure_queue(engine, container, path, format, extra_settings)
            }
        }
    }

    fn translate_s3_queue(
        &self,
        engine: &QueueEngine,
        path: &str,
        format: &str,
        credentials: Option<&crate::infrastructure::olap::queue_engine::S3Credentials>,
        extra_settings: &std::collections::HashMap<String, String>,
    ) -> Result<(String, Option<String>), ClickhouseError> {
        // Build the engine definition
        let engine_def = if let Some(creds) = credentials {
            if let Some(role_arn) = &creds.role_arn {
                format!("S3Queue('{}', '{}', '{}')", path, format, role_arn)
            } else {
                format!("S3Queue('{}', '{}')", path, format)
            }
        } else {
            format!("S3Queue('{}', '{}')", path, format)
        };

        // Build settings
        let mut settings = Vec::new();

        // Core processing settings
        settings.push(format!("mode = '{}'", engine.processing.mode.to_string()));
        settings.push(format!("after_processing = '{}'", engine.processing.after_processing.to_string()));

        if engine.processing.retries > 0 {
            settings.push(format!("s3queue_loading_retries = {}", engine.processing.retries));
        }

        if let Some(threads) = engine.processing.threads {
            settings.push(format!("s3queue_processing_threads_num = {}", threads));
        }

        if engine.processing.parallel_inserts {
            settings.push("s3queue_parallel_inserts = true".to_string());
        }

        if let Some(buckets) = engine.processing.buckets {
            settings.push(format!("s3queue_buckets = {}", buckets));
        }

        // Coordination settings
        if let Some(keeper_path) = &engine.coordination.path {
            settings.push(format!("keeper_path = '{}'", keeper_path));
        }

        if let Some(limit) = engine.coordination.tracked_files_limit {
            settings.push(format!("s3queue_tracked_files_limit = {}", limit));
        }

        if let Some(ttl) = engine.coordination.tracked_file_ttl_sec {
            settings.push(format!("s3queue_tracked_file_ttl_sec = {}", ttl));
        }

        if let Some(min_interval) = engine.coordination.cleanup_interval_min_ms {
            settings.push(format!("s3queue_cleanup_interval_min_ms = {}", min_interval));
        }

        if let Some(max_interval) = engine.coordination.cleanup_interval_max_ms {
            settings.push(format!("s3queue_cleanup_interval_max_ms = {}", max_interval));
        }

        // Monitoring settings
        if engine.monitoring.enable_logging {
            settings.push("s3queue_enable_logging_to_s3queue_log = true".to_string());
        }

        if let Some(min_timeout) = engine.monitoring.polling_min_timeout_ms {
            settings.push(format!("s3queue_polling_min_timeout_ms = {}", min_timeout));
        }

        if let Some(max_timeout) = engine.monitoring.polling_max_timeout_ms {
            settings.push(format!("s3queue_polling_max_timeout_ms = {}", max_timeout));
        }

        if let Some(backoff) = engine.monitoring.polling_backoff_ms {
            settings.push(format!("s3queue_polling_backoff_ms = {}", backoff));
        }

        // Add any extra S3-specific settings
        for (key, value) in extra_settings {
            // Ensure S3-specific settings have the proper prefix if needed
            let setting_key = if key.starts_with("s3queue_") || matches!(key.as_str(), "mode" | "after_processing" | "keeper_path") {
                key.clone()
            } else {
                format!("s3queue_{}", key)
            };
            settings.push(format!("{} = '{}'", setting_key, value));
        }

        let settings_string = if !settings.is_empty() {
            Some(format!("SETTINGS {}", settings.join(", ")))
        } else {
            None
        };

        Ok((engine_def, settings_string))
    }

    fn translate_azure_queue(
        &self,
        engine: &QueueEngine,
        container: &str,
        path: &str,
        format: &str,
        extra_settings: &std::collections::HashMap<String, String>,
    ) -> Result<(String, Option<String>), ClickhouseError> {
        // ClickHouse AzureQueue syntax (if supported)
        let engine_def = format!("AzureQueue('{}', '{}', '{}')", container, path, format);

        // Build settings similar to S3Queue but with Azure-specific prefixes
        let mut settings = Vec::new();

        // Core processing settings
        settings.push(format!("mode = '{}'", engine.processing.mode.to_string()));
        settings.push(format!("after_processing = '{}'", engine.processing.after_processing.to_string()));

        if engine.processing.retries > 0 {
            settings.push(format!("azure_queue_loading_retries = {}", engine.processing.retries));
        }

        // Add keeper path if specified
        if let Some(keeper_path) = &engine.coordination.path {
            settings.push(format!("keeper_path = '{}'", keeper_path));
        }

        // Add extra settings
        for (key, value) in extra_settings {
            settings.push(format!("{} = '{}'", key, value));
        }

        let settings_string = if !settings.is_empty() {
            Some(format!("SETTINGS {}", settings.join(", ")))
        } else {
            None
        };

        Ok((engine_def, settings_string))
    }

    /// Validate ClickHouse-specific constraints
    pub fn validate_clickhouse_config(&self, engine: &QueueEngine) -> Result<(), ClickhouseError> {
        match &engine.source {
            QueueSource::S3 { path, .. } => {
                if !path.starts_with("s3://") {
                    return Err(ClickhouseError::InvalidParameters {
                        message: "S3 path must start with 's3://'".to_string(),
                    });
                }
            }
            QueueSource::Azure { .. } => {
                // Add Azure-specific validations here
            }
        }

        // Validate processing mode constraints
        if matches!(engine.processing.mode, ProcessingMode::Ordered) {
            if engine.processing.buckets.is_some() && engine.processing.threads.is_some() {
                let buckets = engine.processing.buckets.unwrap_or(1);
                let threads = engine.processing.threads.unwrap_or(1);
                
                if buckets < threads {
                    return Err(ClickhouseError::InvalidParameters {
                        message: "For ordered mode, buckets should be >= processing_threads_num".to_string(),
                    });
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::olap::queue_engine::*;
    use std::collections::HashMap;

    #[test]
    fn test_s3queue_basic_translation() {
        let queue_engine = QueueEngine::s3_queue(
            "s3://my-bucket/data/*.json".to_string(),
            "JSONEachRow".to_string(),
        );

        let translator = ClickHouseQueueTranslator;
        let result = translator.translate_to_sql(&queue_engine).unwrap();

        assert_eq!(result.0, "S3Queue('s3://my-bucket/data/*.json', 'JSONEachRow')");
        assert!(result.1.is_some());
        let settings = result.1.unwrap();
        assert!(settings.contains("mode = 'unordered'"));
        assert!(settings.contains("after_processing = 'keep'"));
    }

    #[test]
    fn test_s3queue_comprehensive_settings() {
        let mut queue_engine = QueueEngine::s3_queue(
            "s3://my-bucket/data/*.json".to_string(),
            "JSONEachRow".to_string(),
        );

        // Configure all settings
        queue_engine.processing.mode = ProcessingMode::Ordered;
        queue_engine.processing.retries = 3;
        queue_engine.processing.threads = Some(4);
        queue_engine.processing.parallel_inserts = true;
        queue_engine.processing.buckets = Some(8);

        queue_engine.coordination.path = Some("/clickhouse/s3queue/test".to_string());
        queue_engine.coordination.tracked_files_limit = Some(2000);
        queue_engine.coordination.tracked_file_ttl_sec = Some(3600);

        queue_engine.monitoring.enable_logging = true;
        queue_engine.monitoring.polling_min_timeout_ms = Some(500);
        queue_engine.monitoring.polling_max_timeout_ms = Some(5000);

        let translator = ClickHouseQueueTranslator;
        let result = translator.translate_to_sql(&queue_engine).unwrap();

        let settings = result.1.unwrap();
        assert!(settings.contains("mode = 'ordered'"));
        assert!(settings.contains("s3queue_loading_retries = 3"));
        assert!(settings.contains("s3queue_processing_threads_num = 4"));
        assert!(settings.contains("s3queue_parallel_inserts = true"));
        assert!(settings.contains("s3queue_buckets = 8"));
        assert!(settings.contains("keeper_path = '/clickhouse/s3queue/test'"));
        assert!(settings.contains("s3queue_tracked_files_limit = 2000"));
        assert!(settings.contains("s3queue_tracked_file_ttl_sec = 3600"));
        assert!(settings.contains("s3queue_enable_logging_to_s3queue_log = true"));
        assert!(settings.contains("s3queue_polling_min_timeout_ms = 500"));
        assert!(settings.contains("s3queue_polling_max_timeout_ms = 5000"));
    }
}
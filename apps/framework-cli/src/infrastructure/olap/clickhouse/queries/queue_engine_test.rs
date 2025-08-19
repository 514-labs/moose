#[cfg(test)]
mod queue_engine_tests {
    use crate::infrastructure::olap::clickhouse::queries::*;
    use crate::infrastructure::olap::clickhouse::model::*;
    use crate::infrastructure::olap::queue_engine::*;
    use crate::framework::versions::Version;
    use std::collections::HashMap;

    #[test]
    fn test_s3queue_basic_configuration() {
        let queue_engine = S3QueueEngine::new(
            "s3://my-bucket/data/*.json".to_string(),
            "JSONEachRow".to_string(),
        );

        let table = ClickHouseTable {
            name: "s3_queue_table".to_string(),
            version: Some(Version::from_string("1".to_string())),
            columns: vec![
                ClickHouseColumn {
                    name: "id".to_string(),
                    column_type: ClickHouseColumnType::ClickhouseInt(ClickHouseInt::Int32),
                    required: true,
                    primary_key: false,
                    unique: false,
                    default: None,
                    comment: None,
                },
                ClickHouseColumn {
                    name: "data".to_string(),
                    column_type: ClickHouseColumnType::String,
                    required: true,
                    primary_key: false,
                    unique: false,
                    default: None,
                    comment: None,
                },
                ClickHouseColumn {
                    name: "timestamp".to_string(),
                    column_type: ClickHouseColumnType::DateTime,
                    required: true,
                    primary_key: false,
                    unique: false,
                    default: None,
                    comment: None,
                },
            ],
            engine: ClickhouseEngine::S3Queue(queue_engine),
            order_by: vec![],
        };

        let query = create_table_query("test_db", table).unwrap();
        
        println!("Generated query:");
        println!("{}", query);

        // Verify the query contains expected elements
        assert!(query.contains("ENGINE = S3Queue('s3://my-bucket/data/*.json', 'JSONEachRow')"));
        assert!(query.contains("SETTINGS"));
        assert!(query.contains("mode = 'unordered'"));
        assert!(query.contains("after_processing = 'keep'"));
    }

    #[test]
    fn test_s3queue_comprehensive_configuration() {
        let mut queue_engine = S3QueueEngine::new(
            "s3://blockchain-data/blocks/*.json".to_string(),
            "JSONEachRow".to_string(),
        );

        // Configure for high-throughput Dune Analytics use case
        queue_engine.processing.mode = ProcessingMode::Unordered;
        queue_engine.processing.retries = 3;
        queue_engine.processing.threads = Some(8);
        queue_engine.processing.parallel_inserts = true;
        queue_engine.processing.buckets = Some(16);

        queue_engine.coordination.path = Some("/clickhouse/s3queue/blockchain_data".to_string());
        queue_engine.coordination.tracked_files_limit = Some(5000);
        queue_engine.coordination.tracked_file_ttl_sec = Some(7200); // 2 hours

        queue_engine.monitoring.enable_logging = true;
        queue_engine.monitoring.polling_min_timeout_ms = Some(500);
        queue_engine.monitoring.polling_max_timeout_ms = Some(5000);

        let table = ClickHouseTable {
            name: "blockchain_blocks".to_string(),
            version: Some(Version::from_string("1".to_string())),
            columns: vec![
                ClickHouseColumn {
                    name: "block_number".to_string(),
                    column_type: ClickHouseColumnType::ClickhouseInt(ClickHouseInt::UInt64),
                    required: true,
                    primary_key: false,
                    unique: false,
                    default: None,
                    comment: None,
                },
                ClickHouseColumn {
                    name: "block_hash".to_string(),
                    column_type: ClickHouseColumnType::String,
                    required: true,
                    primary_key: false,
                    unique: false,
                    default: None,
                    comment: None,
                },
            ],
            engine: ClickhouseEngine::S3Queue(queue_engine),
            order_by: vec![],
        };

        let query = create_table_query("dune_analytics", table).unwrap();

        println!("Generated comprehensive S3Queue query:");
        println!("{}", query);

        // Verify all settings are present
        assert!(query.contains("ENGINE = S3Queue('s3://blockchain-data/blocks/*.json', 'JSONEachRow')"));
        assert!(query.contains("mode = 'unordered'"));
        assert!(query.contains("s3queue_loading_retries = 3"));
        assert!(query.contains("s3queue_processing_threads_num = 8"));
        assert!(query.contains("s3queue_parallel_inserts = true"));
        assert!(query.contains("s3queue_buckets = 16"));
        assert!(query.contains("keeper_path = '/clickhouse/s3queue/blockchain_data'"));
        assert!(query.contains("s3queue_enable_logging_to_s3queue_log = true"));
    }

    #[test]
    fn test_s3queue_with_credentials() {
        let mut queue_engine = S3QueueEngine::new(
            "s3://secure-bucket/data/*.parquet".to_string(),
            "Parquet".to_string(),
        );

        // Add S3 credentials
        queue_engine.config.credentials = Some(S3Credentials {
            role_arn: Some("arn:aws:iam::123456789012:role/ClickHouseS3Role".to_string()),
            access_key_id: None,
            secret_access_key: None,
            session_token: None,
        });

        let table = ClickHouseTable {
            name: "secure_data".to_string(),
            version: Some(Version::from_string("1".to_string())),
            columns: vec![
                ClickHouseColumn {
                    name: "event_id".to_string(),
                    column_type: ClickHouseColumnType::Uuid,
                    required: true,
                    primary_key: false,
                    unique: false,
                    default: None,
                    comment: None,
                },
            ],
            engine: ClickhouseEngine::S3Queue(queue_engine),
            order_by: vec![],
        };

        let query = create_table_query("secure_db", table).unwrap();

        println!("Generated S3Queue query with credentials:");
        println!("{}", query);

        // Verify credentials are included
        assert!(query.contains("ENGINE = S3Queue('s3://secure-bucket/data/*.parquet', 'Parquet', 'arn:aws:iam::123456789012:role/ClickHouseS3Role')"));
    }

    #[test]
    fn test_s3queue_validation() {
        // Test invalid S3 path
        let invalid_s3_path = S3QueueEngine::new(
            "invalid-path".to_string(),
            "JSONEachRow".to_string(),
        );

        let table = ClickHouseTable {
            name: "test".to_string(),
            version: None,
            columns: vec![],
            engine: ClickhouseEngine::S3Queue(invalid_s3_path),
            order_by: vec![],
        };

        // This should fail validation in the translator
        let result = create_table_query("test_db", table);
        assert!(result.is_err());
    }

    #[test]
    fn test_extra_settings_support() {
        let mut extra_settings = HashMap::new();
        extra_settings.insert("custom_setting".to_string(), "custom_value".to_string());

        let mut queue_engine = S3QueueEngine::new(
            "s3://custom-bucket/data/*.json".to_string(),
            "JSONEachRow".to_string(),
        );
        queue_engine.config.extra_settings = extra_settings;

        let table = ClickHouseTable {
            name: "custom_settings_table".to_string(),
            version: None,
            columns: vec![
                ClickHouseColumn {
                    name: "data".to_string(),
                    column_type: ClickHouseColumnType::String,
                    required: true,
                    primary_key: false,
                    unique: false,
                    default: None,
                    comment: None,
                },
            ],
            engine: ClickhouseEngine::S3Queue(queue_engine),
            order_by: vec![],
        };

        let query = create_table_query("test_db", table).unwrap();

        println!("Generated query with extra settings:");
        println!("{}", query);

        assert!(query.contains("s3queue_custom_setting = 'custom_value'"));
    }
}
//! # ClickHouse OLAP Implementation
//!
//! This module provides the ClickHouse-specific implementation of OLAP operations.
//! It handles table management, schema changes, and data type conversions between
//! ClickHouse and the framework's type system.
//!
//! ## Features
//! - Table management (create, read, update, delete)
//! - Schema change management
//! - Type system conversion
//! - Version tracking
//! - Table naming conventions
//!
//! ## Dependencies
//! - clickhouse: Client library for ClickHouse database
//! - clickhouse-rs: Alternative ClickHouse client
//! - Framework core types and infrastructure
//!
//! ## Version Support
//! Tables follow the naming convention: {name}_{version}
//! where version is in the format x_y_z (e.g., table_1_0_0)
//!
//! ## Authors
//! - Initial implementation: Framework Team
//! - Last modified: 2024
//!
//! ## Usage Example
//! ```rust
//! let client = create_client(config);
//! let tables = client.list_tables(&config.db_name).await?;
//! ```

use clickhouse::Client;
use clickhouse_rs::ClientHandle;
use errors::ClickhouseError;
use itertools::Itertools;
use log::{debug, info};
use mapper::{std_column_to_clickhouse_column, std_table_to_clickhouse_table};
use queries::{
    basic_field_type_to_string, create_table_query, create_view_query, drop_table_query,
    drop_view_query, update_view_query,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use self::model::ClickHouseSystemTable;
use crate::framework::core::infrastructure::table::{
    Column, ColumnType, DataEnum, EnumMember, EnumValue, Table,
};
use crate::framework::core::infrastructure::view::ViewType;
use crate::framework::core::infrastructure_map::{
    Change, ColumnChange, OlapChange, PrimitiveSignature, PrimitiveTypes, TableChange,
};
use crate::framework::versions::Version;
use crate::infrastructure::olap::clickhouse::model::ClickHouseSystemTableRow;
use crate::infrastructure::olap::{OlapChangesError, OlapOperations};
use crate::project::Project;
use crate::utilities::retry::retry;

pub mod client;
pub mod config;
pub mod errors;
pub mod inserter;
pub mod mapper;
pub mod model;
pub mod queries;
pub mod version_sync;

pub use config::ClickHouseConfig;

/// Type alias for query strings to improve readability
pub type QueryString = String;

/// Represents errors that can occur during ClickHouse operations
#[derive(Debug, thiserror::Error)]
pub enum ClickhouseChangesError {
    /// Error when interacting with ClickHouse database
    #[error("Error interacting with Clickhouse")]
    Clickhouse(#[from] ClickhouseError),

    /// Error from the ClickHouse client library
    #[error("Error interacting with Clickhouse")]
    ClickhouseClient(#[from] clickhouse::error::Error),

    /// Error for unsupported operations
    #[error("Not Supported {0}")]
    NotSupported(String),
}

/// Executes a series of changes to the ClickHouse database schema
///
/// # Arguments
///
/// * `project` - The Project configuration containing ClickHouse connection details
/// * `changes` - A slice of OlapChange representing schema changes to apply
///
/// # Returns
///
/// * `Result<(), ClickhouseChangesError>` - Ok(()) if all changes were successful, or a ClickhouseChangesError if any operation failed
///
/// # Details
///
/// Handles the following types of changes:
/// - Adding/removing/updating tables
/// - Adding/removing/updating views
/// - Column modifications
/// - Order by clause changes
/// - Table engine changes (via deduplication settings)
///
/// Will retry certain operations that return specific ClickHouse error codes indicating retry is possible.
///
/// # Example
/// ```rust
/// let changes = vec![OlapChange::Table(TableChange::Added(table))];
/// execute_changes(&project, &changes).await?;
/// ```
pub async fn execute_changes(
    project: &Project,
    changes: &[OlapChange],
) -> Result<(), ClickhouseChangesError> {
    // TODO refactor this to use the client we want to standardise on in the long term
    // This is currently using the same functions we are using in the current implementation
    let configured_client = create_client(project.clickhouse_config.clone());
    check_ready(&configured_client).await?;

    let db_name = &project.clickhouse_config.db_name;

    for change in changes.iter() {
        match change {
            OlapChange::Table(TableChange::Added(table)) => {
                log::info!("Creating table: {:?}", table.id());

                let clickhouse_table = std_table_to_clickhouse_table(table)?;
                let create_data_table_query = create_table_query(db_name, clickhouse_table)?;
                run_query(&create_data_table_query, &configured_client).await?;
            }
            OlapChange::Table(TableChange::Removed(table)) => {
                log::info!("Removing table: {:?}", table.id());

                let clickhouse_table = std_table_to_clickhouse_table(table)?;
                let drop_query = drop_table_query(db_name, clickhouse_table)?;
                run_query(&drop_query, &configured_client).await?;
            }
            OlapChange::Table(TableChange::Updated {
                name,
                column_changes,
                order_by_change,
                before,
                after,
            }) => {
                if before.deduplicate != after.deduplicate {
                    log::info!("Deduplicate parameter changed for table: {:?}", name);
                    log::info!(
                        "Deleting table: {:?} and recreating it with the proper Clickhouse engine",
                        name
                    );

                    let dropped_table = std_table_to_clickhouse_table(before)?;
                    let drop_query = drop_table_query(db_name, dropped_table)?;
                    run_query(&drop_query, &configured_client).await?;

                    let clickhouse_table = std_table_to_clickhouse_table(after)?;
                    let create_data_table_query = create_table_query(db_name, clickhouse_table)?;
                    run_query(&create_data_table_query, &configured_client).await?;
                } else {
                    log::info!("Updating table: {:?}", name);
                    let mut alter_statements =
                        generate_column_alter_statements(column_changes, db_name, name)?;

                    if order_by_change.before != order_by_change.after {
                        let order_by_alter_statement = generate_order_by_alter_statement(
                            &order_by_change.after,
                            db_name,
                            name,
                        );
                        alter_statements.push(order_by_alter_statement);
                    }

                    for statement in alter_statements {
                        retry(
                            || run_query(&statement, &configured_client),
                            |_, e| match e {
                                clickhouse::error::Error::BadResponse(msg)
                                    if (msg.starts_with("Code: 517.")
                                        && msg.contains("You can retry this error")) =>
                                {
                                    info!("Retrying error {}", e);
                                    true
                                }
                                _ => false,
                            },
                            Duration::from_secs(1),
                        )
                        .await?;
                    }
                }
            }
            OlapChange::View(Change::Added(view)) => match &view.view_type {
                ViewType::TableAlias { source_table_name } => {
                    let create_view_query = create_view_query(
                        db_name,
                        &view.id(),
                        &format!("SELECT * FROM {}.{}", db_name, source_table_name),
                    )?;
                    run_query(&create_view_query, &configured_client).await?;
                }
            },
            OlapChange::View(Change::Removed(view)) => match &view.view_type {
                ViewType::TableAlias { .. } => {
                    let delete_view_query = drop_view_query(db_name, &view.id())?;
                    run_query(&delete_view_query, &configured_client).await?;
                }
            },
            OlapChange::View(Change::Updated { before, after }) => match &after.view_type {
                ViewType::TableAlias { source_table_name } => {
                    let update_view_query = update_view_query(
                        db_name,
                        &before.id(),
                        &format!("SELECT * FROM {}.{}", db_name, source_table_name),
                    )?;
                    run_query(&update_view_query, &configured_client).await?;
                }
            },
        }
    }

    Ok(())
}

pub struct ConfiguredDBClient {
    pub client: Client,
    pub config: ClickHouseConfig,
}

/// Creates a configured ClickHouse client with the provided configuration
///
/// # Arguments
/// * `clickhouse_config` - Configuration for the ClickHouse connection
///
/// # Returns
/// * `ConfiguredDBClient` - A configured client ready for database operations
///
/// # Details
/// Creates a client with:
/// - Proper URL construction (http/https)
/// - Authentication settings
/// - Database selection
/// - Connection options
///
/// # Example
/// ```rust
/// let client = create_client(ClickHouseConfig {
///     host: "localhost".to_string(),
///     host_port: 8123,
///     user: "default".to_string(),
///     password: "".to_string(),
///     db_name: "mydb".to_string(),
///     use_ssl: false,
/// });
/// ```
pub fn create_client(clickhouse_config: ClickHouseConfig) -> ConfiguredDBClient {
    let protocol = if clickhouse_config.use_ssl {
        "https"
    } else {
        "http"
    };
    ConfiguredDBClient {
        client: Client::default()
            .with_url(format!(
                "{}://{}:{}",
                protocol, clickhouse_config.host, clickhouse_config.host_port
            ))
            .with_user(clickhouse_config.user.to_string())
            .with_password(clickhouse_config.password.to_string())
            .with_database(clickhouse_config.db_name.to_string())
            .with_option("flatten_nested", "0"),
        config: clickhouse_config,
    }
}

/// Executes a query against the ClickHouse database
///
/// # Arguments
/// * `query` - The SQL query to execute
/// * `configured_client` - The configured client to use for execution
///
/// # Returns
/// * `Result<(), clickhouse::error::Error>` - Success or error from query execution
///
/// # Details
/// - Logs query for debugging purposes
/// - Executes query using configured client
/// - Handles query execution errors
///
/// # Example
/// ```rust
/// let query = "SELECT 1".to_string();
/// run_query(&query, &client).await?;
/// ```
pub async fn run_query(
    query: &String,
    configured_client: &ConfiguredDBClient,
) -> Result<(), clickhouse::error::Error> {
    debug!("Running query: {:?}", query);
    let client = &configured_client.client;
    client.query(query.as_str()).execute().await
}

/// Checks if the ClickHouse database is ready for operations
///
/// # Arguments
/// * `configured_client` - The configured client to check
///
/// # Returns
/// * `Result<(), clickhouse::error::Error>` - Success if database is ready
///
/// # Details
/// - Executes a simple version query
/// - Implements retry logic for common connection issues
/// - Handles temporary network failures
/// - Maximum 20 retries with 200ms delay
///
/// # Retries
/// Retries on the following conditions:
/// - Connection closed before message completed
/// - Connection reset by peer
/// - Connection not ready
/// - Channel closed
pub async fn check_ready(
    configured_client: &ConfiguredDBClient,
) -> Result<(), clickhouse::error::Error> {
    let dummy_query = "SELECT version()".to_owned();
    crate::utilities::retry::retry(
        || run_query(&dummy_query, configured_client),
        |i, e| {
            i < 20
                && match e {
                    clickhouse::error::Error::Network(v) => {
                        let err_string = v.to_string();
                        debug!("Network error is {}", err_string);
                        err_string.contains("connection closed before message completed")
                            || err_string.contains("connection error: Connection reset by peer")
                            || err_string
                                .contains("operation was canceled: connection was not ready")
                            || err_string.contains("channel closed")
                    }
                    _ => {
                        debug!("Error is {} instead of network error. Will not retry.", e);
                        false
                    }
                }
        },
        tokio::time::Duration::from_millis(200),
    )
    .await
}

/// Fetches tables matching a specific version pattern
///
/// # Arguments
/// * `configured_client` - The configured client to use
/// * `version` - The version pattern to match against table names
///
/// # Returns
/// * `Result<Vec<ClickHouseSystemTable>, clickhouse::error::Error>` - List of matching tables
///
/// # Details
/// - Filters tables by database name and version pattern
/// - Returns full table metadata
/// - Uses parameterized query for safety
pub async fn fetch_tables_with_version(
    configured_client: &ConfiguredDBClient,
    version: &str,
) -> Result<Vec<ClickHouseSystemTable>, clickhouse::error::Error> {
    let client = &configured_client.client;
    let db_name = &configured_client.config.db_name;

    let query = "SELECT uuid, database, name, dependencies_table, engine FROM system.tables WHERE database = ? AND name LIKE ?";

    let tables = client
        .query(query)
        .bind(db_name)
        .bind(version)
        .fetch_all::<ClickHouseSystemTableRow>()
        .await?
        .into_iter()
        .map(|row| row.to_table())
        .collect();

    Ok(tables)
}

/// Gets the number of rows in a table
///
/// # Arguments
/// * `table_name` - Name of the table to check
/// * `config` - ClickHouse configuration
/// * `clickhouse` - Client handle for database operations
///
/// # Returns
/// * `Result<i64, clickhouse_rs::errors::Error>` - Number of rows in the table
///
/// # Details
/// - Uses COUNT(*) for accurate row count
/// - Properly escapes table and database names
/// - Handles empty tables correctly
///
/// # Example
/// ```rust
/// let size = check_table_size("users_1_0_0", &config, &mut client).await?;
/// println!("Table has {} rows", size);
/// ```
pub async fn check_table_size(
    table_name: &str,
    config: &ClickHouseConfig,
    clickhouse: &mut ClientHandle,
) -> Result<i64, clickhouse_rs::errors::Error> {
    info!("Checking size of {} table", table_name);
    let result = clickhouse
        .query(&format!(
            "select count(*) from \"{}\".\"{}\"",
            config.db_name.clone(),
            table_name
        ))
        .fetch_all()
        .await?;
    let rows = result.rows().collect_vec();

    let result: u64 = match rows.len() {
        1 => rows[0].get(0)?,
        _ => panic!("Expected 1 result, got {:?}", rows.len()),
    };
    Ok(result as i64)
}

/// Represents details about a table in ClickHouse
///
/// # Fields
/// * `engine` - The table's engine type
/// * `total_rows` - Optional count of rows in the table
///
/// # Usage
/// Used internally for table metadata operations and checks
#[derive(Debug, Clone, Deserialize, Serialize, clickhouse::Row)]
struct TableDetail {
    pub engine: String,
    pub total_rows: Option<u64>,
}

/// Generates SQL statements for column alterations
///
/// # Arguments
/// * `diff` - List of column changes to apply
/// * `db_name` - Target database name
/// * `table_name` - Target table name
///
/// # Returns
/// * `Result<Vec<String>, ClickhouseError>` - List of ALTER TABLE statements
///
/// # Details
/// Handles three types of column changes:
/// - Added: Creates new columns
/// - Removed: Drops existing columns
/// - Updated: Modifies column type/properties
///
/// # Example
/// ```rust
/// let changes = vec![ColumnChange::Added(Column {
///     name: "new_column".to_string(),
///     data_type: ColumnType::Int,
///     required: true,
///     ..Default::default()
/// })];
/// let statements = generate_column_alter_statements(&changes, "mydb", "mytable")?;
/// ```
fn generate_column_alter_statements(
    diff: &[ColumnChange],
    db_name: &str,
    table_name: &str,
) -> Result<Vec<String>, ClickhouseError> {
    let mut statements: Vec<String> = vec![];

    for column_change in diff {
        match column_change {
            ColumnChange::Added(col) => {
                let clickhouse_column = std_column_to_clickhouse_column(col.clone())?;
                let column_type_string =
                    basic_field_type_to_string(&clickhouse_column.column_type)?;

                statements.push(format!(
                    "ALTER TABLE `{}`.`{}` ADD COLUMN `{}` {}",
                    db_name, table_name, clickhouse_column.name, column_type_string
                ));
            }
            ColumnChange::Removed(col) => {
                statements.push(format!(
                    "ALTER TABLE `{}`.`{}` DROP COLUMN `{}`",
                    db_name, table_name, col.name
                ));
            }
            ColumnChange::Updated { before, after } => {
                let clickhouse_column = std_column_to_clickhouse_column(after.clone())?;
                let column_type_string =
                    basic_field_type_to_string(&clickhouse_column.column_type)?;

                statements.push(format!(
                    "ALTER TABLE `{}`.`{}` MODIFY COLUMN `{}` {}",
                    db_name, table_name, before.name, column_type_string
                ));
            }
        }
    }

    Ok(statements)
}

/// Generates an ORDER BY clause alteration statement
///
/// # Arguments
/// * `order_by` - List of columns for ordering
/// * `db_name` - Target database name
/// * `table_name` - Target table name
///
/// # Returns
/// * `String` - The complete ALTER TABLE statement
///
/// # Details
/// - Generates proper SQL syntax for ORDER BY modifications
/// - Handles empty order by clauses
/// - Properly escapes identifiers
///
/// # Example
/// ```rust
/// let order_by = vec!["id".to_string(), "timestamp".to_string()];
/// let statement = generate_order_by_alter_statement(&order_by, "mydb", "mytable");
/// // Result: ALTER TABLE `mydb`.`mytable` MODIFY ORDER BY (`id`, `timestamp`)
/// ```
fn generate_order_by_alter_statement(
    order_by: &[String],
    db_name: &str,
    table_name: &str,
) -> String {
    format!(
        "ALTER TABLE `{}`.`{}` MODIFY ORDER BY ({})",
        db_name,
        table_name,
        order_by.iter().map(|col| format!("`{}`", col)).join(", ")
    )
}

/// Extracts version information from a table name
///
/// # Arguments
/// * `table_name` - The name of the table to parse
/// * `default_version` - The version to use for tables that don't follow the versioning convention
///
/// # Returns
/// * `(String, Version)` - A tuple containing the base name and version
///
/// # Format
/// For tables following the naming convention: {name}_{version}
/// where version is in the format x_y_z (e.g., 1_0_0)
/// For tables not following the convention: returns the full name and default_version
///
/// # Example
/// ```rust
/// let (base_name, version) = extract_version_from_table_name("users_1_0_0", "0.0.0");
/// assert_eq!(base_name, "users");
/// assert_eq!(version.to_string(), "1.0.0");
///
/// let (base_name, version) = extract_version_from_table_name("my_table", "1.0.0");
/// assert_eq!(base_name, "my_table");
/// assert_eq!(version.to_string(), "1.0.0");
/// ```
fn extract_version_from_table_name(table_name: &str, default_version: &str) -> (String, Version) {
    debug!("Extracting version from table name: {}", table_name);
    debug!("Using default version: {}", default_version);

    // Special case for empty table name
    if table_name.is_empty() {
        debug!("Empty table name, using default version");
        return (
            table_name.to_string(),
            Version::from_string(default_version.to_string()),
        );
    }

    // Special case for tables ending in _MV (materialized views)
    if table_name.ends_with("_MV") {
        debug!("Materialized view detected, using default version");
        return (
            table_name.to_string(),
            Version::from_string(default_version.to_string()),
        );
    }

    let parts: Vec<&str> = table_name.split('_').collect();
    debug!("Split table name into parts: {:?}", parts);

    if parts.len() < 2 {
        debug!("Table name has fewer than 2 parts, using default version");
        // If table doesn't follow naming convention, return full name and default version
        return (
            table_name.to_string(),
            Version::from_string(default_version.to_string()),
        );
    }

    // Find the first numeric part - this marks the start of the version
    let mut version_start_idx = None;
    for (i, part) in parts.iter().enumerate() {
        if part.chars().all(|c| c.is_ascii_digit()) {
            version_start_idx = Some(i);
            debug!("Found version start at index {}: {}", i, part);
            break;
        }
    }

    match version_start_idx {
        Some(idx) => {
            // Filter out empty parts when joining base name
            let base_parts: Vec<&str> = parts[..idx]
                .iter()
                .filter(|p| !p.is_empty())
                .copied()
                .collect();
            let base_name = base_parts.join("_");
            debug!(
                "Base parts: {:?}, joined base name: {}",
                base_parts, base_name
            );

            // Filter out empty parts when joining version
            let version_parts: Vec<&str> = parts[idx..]
                .iter()
                .filter(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
                .copied()
                .collect();
            debug!("Version parts: {:?}", version_parts);

            // If we have no valid version parts, return the original name and default version
            if version_parts.is_empty() {
                debug!("No valid version parts found, using default version");
                return (
                    table_name.to_string(),
                    Version::from_string(default_version.to_string()),
                );
            }

            let version_str = version_parts.join(".");
            debug!("Created version string: {}", version_str);

            (base_name, Version::from_string(version_str))
        }
        None => {
            debug!("No version parts found, using default version");
            (
                table_name.to_string(),
                Version::from_string(default_version.to_string()),
            )
        }
    }
}

#[async_trait::async_trait]
impl OlapOperations for ConfiguredDBClient {
    /// Retrieves all tables from the ClickHouse database and converts them to framework Table objects
    ///
    /// # Arguments
    /// * `db_name` - The name of the database to list tables from
    ///
    /// # Returns
    /// * `Result<Vec<Table>, OlapChangesError>` - A list of Table objects on success
    ///
    /// # Details
    /// This implementation:
    /// 1. Queries system.tables for basic table information
    /// 2. Extracts version information from table names
    /// 3. Queries system.columns for column metadata
    /// 4. Converts ClickHouse types to framework types
    /// 5. Creates Table objects with proper versioning and source primitives
    ///
    /// # Notes
    /// - Tables without proper version information in their names are skipped
    /// - Column types are converted based on ClickHouse to framework type mapping
    /// - Primary key columns are used for order_by clauses
    /// - Tables are sorted by name in the final result
    async fn list_tables(
        &self,
        db_name: &str,
        project: &Project,
    ) -> Result<Vec<Table>, OlapChangesError> {
        debug!("Starting list_tables operation for database: {}", db_name);
        debug!("Using project version: {}", project.cur_version());

        // First get basic table information
        let query = format!(
            r#"
            SELECT 
                name,
                engine,
                create_table_query
            FROM system.tables 
            WHERE database = '{}' 
            AND engine != 'View'
            AND NOT name LIKE '.%'
            ORDER BY name
            "#,
            db_name
        );
        debug!("Executing table query: {}", query);

        let mut cursor = self
            .client
            .query(&query)
            .fetch::<(String, String, String)>()
            .map_err(|e| {
                debug!("Error fetching tables: {}", e);
                OlapChangesError::DatabaseError(e.to_string())
            })?;

        let mut tables = Vec::new();

        while let Some((table_name, engine, create_query)) = cursor
            .next()
            .await
            .map_err(|e| OlapChangesError::DatabaseError(e.to_string()))?
        {
            debug!("Processing table: {}", table_name);
            debug!("Table engine: {}", engine);
            debug!("Create query: {}", create_query);

            // Extract ORDER BY columns from create_query
            let order_by_cols = extract_order_by_from_create_query(&create_query);
            debug!("Extracted ORDER BY columns: {:?}", order_by_cols);

            // Get column information for each table
            let columns_query = format!(
                r#"
                SELECT 
                    name,
                    type,
                    is_in_primary_key,
                    is_in_sorting_key
                FROM system.columns 
                WHERE database = '{}' 
                AND table = '{}'
                ORDER BY name
                "#,
                db_name, table_name
            );
            debug!(
                "Executing columns query for table {}: {}",
                table_name, columns_query
            );

            let mut columns_cursor = self
                .client
                .query(&columns_query)
                .fetch::<(String, String, u8, u8)>()
                .map_err(|e| {
                    debug!("Error fetching columns for table {}: {}", table_name, e);
                    OlapChangesError::DatabaseError(e.to_string())
                })?;

            let mut columns = Vec::new();

            while let Some((col_name, col_type, is_primary, is_sorting)) = columns_cursor
                .next()
                .await
                .map_err(|e| OlapChangesError::DatabaseError(e.to_string()))?
            {
                debug!(
                    "Processing column: {} (type: {}, primary: {}, sorting: {})",
                    col_name, col_type, is_primary, is_sorting
                );

                // Convert ClickHouse types to framework types
                let (data_type, is_nullable) = convert_clickhouse_type_to_column_type(&col_type)
                    .map_err(OlapChangesError::DatabaseError)?;
                debug!(
                    "Converted column type: {:?}, nullable: {}",
                    data_type, is_nullable
                );

                let column = Column {
                    name: col_name.clone(),
                    data_type,
                    required: !is_nullable,
                    unique: false,
                    primary_key: is_primary == 1,
                    default: None,
                };

                columns.push(column);
            }

            // Sort columns by name for consistent ordering
            columns.sort_by(|a, b| a.name.cmp(&b.name));

            debug!("Found {} columns for table {}", columns.len(), table_name);

            // Extract base name and version for source primitive
            let (base_name, version) =
                extract_version_from_table_name(&table_name, project.cur_version().as_str());

            // Create source primitive signature using the base name
            let source_primitive = PrimitiveSignature {
                name: base_name,
                primitive_type: PrimitiveTypes::DataModel,
            };

            // Create the Table object using the original table_name
            let table = Table {
                name: table_name, // Keep the original table name with version
                columns,
                order_by: order_by_cols, // Use the extracted ORDER BY columns
                deduplicate: engine.contains("ReplacingMergeTree"),
                version, // Still store the version for reference
                source_primitive,
            };
            debug!("Created table object: {:?}", table);

            tables.push(table);
        }

        debug!(
            "Completed list_tables operation, found {} tables",
            tables.len()
        );
        Ok(tables)
    }
}

/// Converts a ClickHouse column type string to the framework's ColumnType
///
/// # Arguments
/// * `ch_type` - The ClickHouse type string to convert
///
/// # Returns
/// * `Result<(ColumnType, bool), String>` - A tuple containing:
///   - The converted framework type
///   - A boolean indicating if the type is nullable (true = nullable)
///
/// # Type Mappings
/// - String -> ColumnType::String
/// - UInt8/16/32, Int8/16/32 -> ColumnType::Int
/// - UInt64, Int64 -> ColumnType::BigInt
/// - Float32/64 -> ColumnType::Float
/// - Decimal* -> ColumnType::Decimal
/// - DateTime -> ColumnType::DateTime
/// - DateTime64(precision) -> ColumnType::DateTime
/// - DateTime('timezone') -> ColumnType::DateTime
/// - DateTime64(precision, 'timezone') -> ColumnType::DateTime
/// - Bool/Boolean -> ColumnType::Boolean
/// - Array(*) -> ColumnType::Array
/// - JSON -> ColumnType::Json
/// - Enum8/16 -> ColumnType::Enum
/// - Nullable(*) -> (inner_type, true)
///
/// # Example
/// ```rust
/// let (framework_type, is_nullable) = convert_clickhouse_type_to_column_type("Nullable(Int32)")?;
/// assert_eq!(framework_type, ColumnType::Int);
/// assert!(is_nullable);
/// ```
fn convert_clickhouse_type_to_column_type(ch_type: &str) -> Result<(ColumnType, bool), String> {
    use regex::Regex;

    // Handle Nullable type wrapper
    if ch_type.starts_with("Nullable(") {
        let inner_type = ch_type
            .strip_prefix("Nullable(")
            .and_then(|s| s.strip_suffix(")"))
            .ok_or_else(|| format!("Invalid Nullable type format: {}", ch_type))?;

        let (inner_column_type, _) = convert_clickhouse_type_to_column_type(inner_type)?;
        return Ok((inner_column_type, true));
    }

    // Handle DateTime types with parameters
    if ch_type.starts_with("DateTime") {
        // All DateTime variants map to ColumnType::DateTime
        // We could store precision and timezone as metadata if needed in the future
        return Ok((ColumnType::DateTime, false));
    }

    // Handle Enum types first since they contain parentheses which would interfere with the base type extraction
    if ch_type.starts_with("Enum8(") || ch_type.starts_with("Enum16(") {
        let enum_content = ch_type
            .trim_start_matches("Enum8(")
            .trim_start_matches("Enum16(")
            .trim_end_matches(')');

        // Return error if enum content is empty
        if enum_content.trim().is_empty() {
            return Err(format!("Empty enum definition: {}", ch_type));
        }

        // Use regex to match enum values, handling potential commas in the names
        let re = Regex::new(r"'([^']*)'\s*=\s*(\d+)").map_err(|e| e.to_string())?;
        let values = re
            .captures_iter(enum_content)
            .map(|cap| {
                let name = cap[1].to_string();
                let value = cap[2].parse::<u8>().map_err(|e| e.to_string())?;

                Ok(EnumMember {
                    name: name.clone(),
                    value: EnumValue::Int(value),
                })
            })
            .collect::<Result<Vec<_>, String>>()?;

        // Return error if no valid enum values were found
        if values.is_empty() {
            return Err(format!("No valid enum values found in: {}", ch_type));
        }

        return Ok((
            ColumnType::Enum(DataEnum {
                name: "Unknown".to_string(), // The actual enum name will be set elsewhere
                values,
            }),
            false,
        ));
    }

    // Remove any parameters from type string for other types
    let base_type = ch_type.split('(').next().unwrap_or(ch_type);

    let column_type = match base_type {
        "String" => Ok(ColumnType::String),
        "UInt8" | "UInt16" | "UInt32" | "Int8" | "Int16" | "Int32" => Ok(ColumnType::Int),
        "UInt64" | "Int64" => Ok(ColumnType::BigInt),
        "Float32" | "Float64" => Ok(ColumnType::Float),
        "Decimal" | "Decimal32" | "Decimal64" | "Decimal128" => Ok(ColumnType::Decimal),
        "Bool" | "Boolean" => Ok(ColumnType::Boolean),
        "Array" => {
            // Extract the inner type from Array(...) format
            let inner_type = ch_type
                .strip_prefix("Array(")
                .and_then(|s| s.strip_suffix(")"))
                .ok_or_else(|| format!("Invalid Array type format: {}", ch_type))?;

            let (inner_column_type, inner_nullable) =
                convert_clickhouse_type_to_column_type(inner_type)?;
            Ok(ColumnType::Array {
                element_type: Box::new(inner_column_type),
                element_nullable: inner_nullable,
            })
        }
        "JSON" => Ok(ColumnType::Json),
        _ => Err(format!("Unsupported ClickHouse type: {}", ch_type)),
    }?;

    Ok((column_type, false))
}

/// Extracts ORDER BY columns from a CREATE TABLE query
///
/// # Arguments
/// * `create_query` - The CREATE TABLE query string
///
/// # Returns
/// * `Vec<String>` - List of column names in the ORDER BY clause, or empty vector if none found
///
/// # Example
/// ```rust
/// let query = "CREATE TABLE test (id Int64) ENGINE = MergeTree() ORDER BY (id, timestamp)";
/// let order_by = extract_order_by_from_create_query(query);
/// assert_eq!(order_by, vec!["id".to_string(), "timestamp".to_string()]);
/// ```
fn extract_order_by_from_create_query(create_query: &str) -> Vec<String> {
    debug!("Extracting ORDER BY from query: {}", create_query);

    // Find the ORDER BY clause, being careful not to match PRIMARY KEY
    let mut after_order_by = None;
    for (idx, _) in create_query.to_uppercase().match_indices("ORDER BY") {
        // Check if this is not part of "PRIMARY KEY" by looking at the preceding text
        let preceding_text = &create_query[..idx].trim_end().to_uppercase();
        if !preceding_text.ends_with("PRIMARY KEY") {
            after_order_by = Some(&create_query[idx..]);
            break;
        }
    }

    if let Some(after_order_by) = after_order_by {
        // Find where the ORDER BY clause ends (at SETTINGS or end of string)
        let mut end_idx = after_order_by.len();
        if let Some(settings_idx) = after_order_by.to_uppercase().find("SETTINGS") {
            end_idx = settings_idx;
        }
        if let Some(next_order_by) = after_order_by[8..].to_uppercase().find("ORDER BY") {
            end_idx = std::cmp::min(end_idx, next_order_by + 8);
        }
        let order_by_clause = &after_order_by[..end_idx];

        // Extract the column names
        let order_by_content = order_by_clause
            .trim_start_matches("ORDER BY")
            .trim()
            .trim_matches('(')
            .trim_matches(')');

        debug!("Found ORDER BY content: {}", order_by_content);

        // Split by comma and clean up each column name
        return order_by_content
            .split(',')
            .map(|s| s.trim().trim_matches('`').to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }

    debug!("No explicit ORDER BY clause found");
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framework::core::infrastructure::table::{Column, ColumnType, EnumValue};

    #[test]
    fn test_generate_column_alter_statements_with_array_float() {
        let diff = vec![ColumnChange::Added(Column {
            name: "prices".to_string(),
            data_type: ColumnType::Array {
                element_type: Box::new(ColumnType::Float),
                element_nullable: false,
            },
            required: false,
            unique: false,
            primary_key: false,
            default: None,
        })];

        let statements = generate_column_alter_statements(&diff, "test_db", "test_table").unwrap();

        assert_eq!(statements.len(), 1);
        assert_eq!(
            statements[0],
            "ALTER TABLE `test_db`.`test_table` ADD COLUMN `prices` Array(Float64)"
        );
    }

    #[test]
    fn test_generate_column_alter_statements() {
        let diff = vec![
            ColumnChange::Added(Column {
                name: "age".to_string(),
                data_type: ColumnType::Int,
                required: false,
                unique: false,
                primary_key: false,
                default: None,
            }),
            ColumnChange::Removed(Column {
                name: "old_column".to_string(),
                data_type: ColumnType::String,
                required: false,
                unique: false,
                primary_key: false,
                default: None,
            }),
            ColumnChange::Updated {
                before: Column {
                    name: "id".to_string(),
                    data_type: ColumnType::Int,
                    required: true,
                    unique: true,
                    primary_key: true,
                    default: None,
                },
                after: Column {
                    name: "id".to_string(),
                    data_type: ColumnType::Float,
                    required: true,
                    unique: true,
                    primary_key: true,
                    default: None,
                },
            },
        ];

        let statements = generate_column_alter_statements(&diff, "test_db", "test_table").unwrap();

        assert_eq!(statements.len(), 3);
        assert_eq!(
            statements[0],
            "ALTER TABLE `test_db`.`test_table` ADD COLUMN `age` Int64"
        );
        assert_eq!(
            statements[1],
            "ALTER TABLE `test_db`.`test_table` DROP COLUMN `old_column`"
        );
        assert_eq!(
            statements[2],
            "ALTER TABLE `test_db`.`test_table` MODIFY COLUMN `id` Float64"
        );
    }

    #[test]
    fn test_generate_order_by_alter_statement() {
        let new_order_by = vec!["id".to_string(), "timestamp".to_string()];
        let db_name = "test_db";
        let table_name = "test_table";

        let statement = generate_order_by_alter_statement(&new_order_by, db_name, table_name);

        assert_eq!(
            statement,
            "ALTER TABLE `test_db`.`test_table` MODIFY ORDER BY (`id`, `timestamp`)"
        );
    }

    #[test]
    fn test_generate_order_by_alter_statement_single_column() {
        let new_order_by = vec!["id".to_string()];
        let db_name = "test_db";
        let table_name = "test_table";

        let statement = generate_order_by_alter_statement(&new_order_by, db_name, table_name);

        assert_eq!(
            statement,
            "ALTER TABLE `test_db`.`test_table` MODIFY ORDER BY (`id`)"
        );
    }

    #[test]
    fn test_generate_order_by_alter_statement_empty_order_by() {
        let new_order_by: Vec<String> = vec![];
        let db_name = "test_db";
        let table_name = "test_table";

        let statement = generate_order_by_alter_statement(&new_order_by, db_name, table_name);

        assert_eq!(
            statement,
            "ALTER TABLE `test_db`.`test_table` MODIFY ORDER BY ()"
        );
    }

    #[test]
    fn test_datetime_type_conversion() {
        // Test basic DateTime
        assert_eq!(
            convert_clickhouse_type_to_column_type("DateTime"),
            Ok((ColumnType::DateTime, false))
        );

        // Test DateTime64 with precision
        assert_eq!(
            convert_clickhouse_type_to_column_type("DateTime64(3)"),
            Ok((ColumnType::DateTime, false))
        );

        // Test DateTime with timezone
        assert_eq!(
            convert_clickhouse_type_to_column_type("DateTime('UTC')"),
            Ok((ColumnType::DateTime, false))
        );

        // Test DateTime64 with precision and timezone
        assert_eq!(
            convert_clickhouse_type_to_column_type("DateTime64(6, 'America/New_York')"),
            Ok((ColumnType::DateTime, false))
        );
    }

    #[test]
    fn test_enum_type_conversion() {
        // Test Enum8
        let enum8_type = "Enum8('RED' = 1, 'GREEN' = 2, 'BLUE' = 3)";
        let result = convert_clickhouse_type_to_column_type(enum8_type).unwrap();

        match result {
            (ColumnType::Enum(data_enum), false) => {
                assert_eq!(data_enum.values.len(), 3);
                assert_eq!(data_enum.values[0].name, "RED");
                assert_eq!(data_enum.values[0].value, EnumValue::Int(1));
                assert_eq!(data_enum.values[1].name, "GREEN");
                assert_eq!(data_enum.values[1].value, EnumValue::Int(2));
                assert_eq!(data_enum.values[2].name, "BLUE");
                assert_eq!(data_enum.values[2].value, EnumValue::Int(3));
            }
            _ => panic!("Expected Enum type"),
        }

        // Test Enum16
        let enum16_type = "Enum16('PENDING' = 0, 'ACTIVE' = 1, 'INACTIVE' = 2)";
        let result = convert_clickhouse_type_to_column_type(enum16_type).unwrap();

        match result {
            (ColumnType::Enum(data_enum), false) => {
                assert_eq!(data_enum.values.len(), 3);
                assert_eq!(data_enum.values[0].name, "PENDING");
                assert_eq!(data_enum.values[0].value, EnumValue::Int(0));
                assert_eq!(data_enum.values[1].name, "ACTIVE");
                assert_eq!(data_enum.values[1].value, EnumValue::Int(1));
                assert_eq!(data_enum.values[2].name, "INACTIVE");
                assert_eq!(data_enum.values[2].value, EnumValue::Int(2));
            }
            _ => panic!("Expected Enum type"),
        }

        // Test enum with spaces and special characters in names
        let complex_enum = "Enum8('NOT FOUND' = 1, 'BAD_REQUEST' = 2, 'SERVER ERROR!' = 3)";
        let result = convert_clickhouse_type_to_column_type(complex_enum).unwrap();

        match result {
            (ColumnType::Enum(data_enum), false) => {
                assert_eq!(data_enum.values.len(), 3);
                assert_eq!(data_enum.values[0].name, "NOT FOUND");
                assert_eq!(data_enum.values[1].name, "BAD_REQUEST");
                assert_eq!(data_enum.values[2].name, "SERVER ERROR!");
            }
            _ => panic!("Expected Enum type"),
        }
    }

    #[test]
    fn test_nullable_type_conversion() {
        // Test basic nullable types
        assert_eq!(
            convert_clickhouse_type_to_column_type("Nullable(Int32)"),
            Ok((ColumnType::Int, true))
        );
        assert_eq!(
            convert_clickhouse_type_to_column_type("Nullable(String)"),
            Ok((ColumnType::String, true))
        );
        assert_eq!(
            convert_clickhouse_type_to_column_type("Nullable(Float64)"),
            Ok((ColumnType::Float, true))
        );

        // Test nullable datetime
        assert_eq!(
            convert_clickhouse_type_to_column_type("Nullable(DateTime)"),
            Ok((ColumnType::DateTime, true))
        );
        assert_eq!(
            convert_clickhouse_type_to_column_type("Nullable(DateTime64(3))"),
            Ok((ColumnType::DateTime, true))
        );

        // Test nullable array
        let (array_type, is_nullable) =
            convert_clickhouse_type_to_column_type("Nullable(Array(Int32))").unwrap();
        assert!(is_nullable);
        match array_type {
            ColumnType::Array {
                element_type,
                element_nullable,
            } => {
                assert_eq!(*element_type, ColumnType::Int);
                assert!(!element_nullable);
            }
            _ => panic!("Expected Array type"),
        }

        // Test nullable enum
        let enum_type = "Nullable(Enum8('RED' = 1, 'GREEN' = 2, 'BLUE' = 3))";
        let (column_type, is_nullable) = convert_clickhouse_type_to_column_type(enum_type).unwrap();
        assert!(is_nullable);
        match column_type {
            ColumnType::Enum(data_enum) => {
                assert_eq!(data_enum.values.len(), 3);
                assert_eq!(data_enum.values[0].name, "RED");
                assert_eq!(data_enum.values[0].value, EnumValue::Int(1));
            }
            _ => panic!("Expected Enum type"),
        }

        // Test array with nullable elements
        let array_type = "Array(Nullable(Int32))";
        let (column_type, is_nullable) =
            convert_clickhouse_type_to_column_type(array_type).unwrap();
        assert!(!is_nullable); // The array itself is not nullable
        match column_type {
            ColumnType::Array {
                element_type,
                element_nullable,
            } => {
                assert_eq!(*element_type, ColumnType::Int);
                assert!(element_nullable); // But its elements are nullable
            }
            _ => panic!("Expected Array type"),
        }
    }

    #[test]
    fn test_extract_version_from_table_name() {
        // Test two-part versions
        let (base_name, version) = extract_version_from_table_name("Bar_0_0", "1.0.0");
        assert_eq!(base_name, "Bar");
        assert_eq!(version.to_string(), "0.0");

        let (base_name, version) = extract_version_from_table_name("Foo_0_0", "1.0.0");
        assert_eq!(base_name, "Foo");
        assert_eq!(version.to_string(), "0.0");

        // Test three-part versions
        let (base_name, version) = extract_version_from_table_name("Bar_0_0_0", "1.0.0");
        assert_eq!(base_name, "Bar");
        assert_eq!(version.to_string(), "0.0.0");

        let (base_name, version) = extract_version_from_table_name("Foo_1_2_3", "0.0.0");
        assert_eq!(base_name, "Foo");
        assert_eq!(version.to_string(), "1.2.3");

        // Test table names with underscores
        let (base_name, version) = extract_version_from_table_name("My_Table_0_0", "1.0.0");
        assert_eq!(base_name, "My_Table");
        assert_eq!(version.to_string(), "0.0");

        let (base_name, version) =
            extract_version_from_table_name("Complex_Table_Name_1_0_0", "0.0.0");
        assert_eq!(base_name, "Complex_Table_Name");
        assert_eq!(version.to_string(), "1.0.0");

        // Test invalid formats - should use default version
        let (base_name, version) = extract_version_from_table_name("TableWithoutVersion", "1.0.0");
        assert_eq!(base_name, "TableWithoutVersion");
        assert_eq!(version.to_string(), "1.0.0");

        let (base_name, version) =
            extract_version_from_table_name("Table_WithoutNumericVersion", "1.0.0");
        assert_eq!(base_name, "Table_WithoutNumericVersion");
        assert_eq!(version.to_string(), "1.0.0");

        // Test edge cases
        let (base_name, version) = extract_version_from_table_name("", "1.0.0");
        assert_eq!(base_name, "");
        assert_eq!(version.to_string(), "1.0.0");

        let (base_name, version) = extract_version_from_table_name("_0_0", "1.0.0");
        assert_eq!(base_name, "");
        assert_eq!(version.to_string(), "0.0");

        let (base_name, version) = extract_version_from_table_name("Table_0_0_", "1.0.0");
        assert_eq!(base_name, "Table");
        assert_eq!(version.to_string(), "0.0");

        // Test mixed numeric and non-numeric parts
        let (base_name, version) = extract_version_from_table_name("Table2_0_0", "1.0.0");
        assert_eq!(base_name, "Table2");
        assert_eq!(version.to_string(), "0.0");

        let (base_name, version) = extract_version_from_table_name("V2_Table_1_0_0", "0.0.0");
        assert_eq!(base_name, "V2_Table");
        assert_eq!(version.to_string(), "1.0.0");

        // Test materialized views
        let (base_name, version) = extract_version_from_table_name("BarAggregated_MV", "1.0.0");
        assert_eq!(base_name, "BarAggregated_MV");
        assert_eq!(version.to_string(), "1.0.0");

        // Test non-versioned tables
        let (base_name, version) = extract_version_from_table_name("Foo", "1.0.0");
        assert_eq!(base_name, "Foo");
        assert_eq!(version.to_string(), "1.0.0");

        let (base_name, version) = extract_version_from_table_name("Bar", "1.0.0");
        assert_eq!(base_name, "Bar");
        assert_eq!(version.to_string(), "1.0.0");
    }

    #[test]
    fn test_extract_order_by_from_create_query() {
        // Test with explicit ORDER BY
        let query = "CREATE TABLE test (id Int64) ENGINE = MergeTree() ORDER BY (id, timestamp)";
        let order_by = extract_order_by_from_create_query(query);
        assert_eq!(order_by, vec!["id".to_string(), "timestamp".to_string()]);

        // Test with PRIMARY KEY and ORDER BY being different
        let query =
            "CREATE TABLE test (id Int64) ENGINE = MergeTree PRIMARY KEY id ORDER BY (timestamp)";
        let order_by = extract_order_by_from_create_query(query);
        assert_eq!(order_by, vec!["timestamp".to_string()]);

        // Test with PRIMARY KEY but no explicit ORDER BY (should return empty)
        let query = "CREATE TABLE test (id Int64) ENGINE = MergeTree PRIMARY KEY id";
        let order_by = extract_order_by_from_create_query(query);
        assert_eq!(order_by, Vec::<String>::new());

        // Test with PRIMARY KEY and implicit ORDER BY through PRIMARY KEY
        let query = "CREATE TABLE local.Foo_0_0 (`primaryKey` String, `timestamp` Float64, `optionalText` Nullable(String)) ENGINE = MergeTree PRIMARY KEY primaryKey ORDER BY primaryKey SETTINGS index_granularity = 8192";
        let order_by = extract_order_by_from_create_query(query);
        assert_eq!(order_by, vec!["primaryKey".to_string()]);

        // Test with SETTINGS clause
        let query = "CREATE TABLE test (id Int64) ENGINE = MergeTree() ORDER BY (id, timestamp) SETTINGS index_granularity = 8192";
        let order_by = extract_order_by_from_create_query(query);
        assert_eq!(order_by, vec!["id".to_string(), "timestamp".to_string()]);

        // Test with backticks
        let query =
            "CREATE TABLE test (id Int64) ENGINE = MergeTree() ORDER BY (`id`, `timestamp`)";
        let order_by = extract_order_by_from_create_query(query);
        assert_eq!(order_by, vec!["id".to_string(), "timestamp".to_string()]);

        // Test without parentheses
        let query = "CREATE TABLE test (id Int64) ENGINE = MergeTree() ORDER BY id";
        let order_by = extract_order_by_from_create_query(query);
        assert_eq!(order_by, vec!["id".to_string()]);

        // Test with no ORDER BY clause
        let query = "CREATE TABLE test (id Int64) ENGINE = MergeTree()";
        let order_by = extract_order_by_from_create_query(query);
        assert_eq!(order_by, Vec::<String>::new());
    }

    #[test]
    fn test_extract_order_by_from_create_query_edge_cases() {
        // Test with multiple ORDER BY clauses (should only use the first one)
        let query =
            "CREATE TABLE test (id Int64) ENGINE = MergeTree() ORDER BY (id) ORDER BY (timestamp)";
        let order_by = extract_order_by_from_create_query(query);
        assert_eq!(order_by, vec!["id".to_string()]);

        // Test with malformed ORDER BY clause (missing closing parenthesis)
        let query = "CREATE TABLE test (id Int64) ENGINE = MergeTree() ORDER BY (id, timestamp";
        let order_by = extract_order_by_from_create_query(query);
        assert_eq!(order_by, vec!["id".to_string(), "timestamp".to_string()]);

        // Test with empty ORDER BY clause
        let query = "CREATE TABLE test (id Int64) ENGINE = MergeTree() ORDER BY ()";
        let order_by = extract_order_by_from_create_query(query);
        assert_eq!(order_by, Vec::<String>::new());

        // Test with ORDER BY clause containing only spaces
        let query = "CREATE TABLE test (id Int64) ENGINE = MergeTree() ORDER BY (   )";
        let order_by = extract_order_by_from_create_query(query);
        assert_eq!(order_by, Vec::<String>::new());

        // Test with ORDER BY clause containing empty entries
        let query = "CREATE TABLE test (id Int64) ENGINE = MergeTree() ORDER BY (id,,timestamp)";
        let order_by = extract_order_by_from_create_query(query);
        assert_eq!(order_by, vec!["id".to_string(), "timestamp".to_string()]);

        // Test with complex expressions in ORDER BY
        let query = "CREATE TABLE test (id Int64) ENGINE = MergeTree() ORDER BY (id, cityId, `user.id`, nested.field)";
        let order_by = extract_order_by_from_create_query(query);
        assert_eq!(
            order_by,
            vec![
                "id".to_string(),
                "cityId".to_string(),
                "user.id".to_string(),
                "nested.field".to_string()
            ]
        );

        // Test with PRIMARY KEY in column definition and ORDER BY
        let query = "CREATE TABLE test (`PRIMARY KEY` Int64) ENGINE = MergeTree() ORDER BY (`id`)";
        let order_by = extract_order_by_from_create_query(query);
        assert_eq!(order_by, vec!["id".to_string()]);
    }

    #[test]
    fn test_convert_clickhouse_type_error_handling() {
        // Test invalid Nullable format
        assert!(convert_clickhouse_type_to_column_type("Nullable").is_err());
        assert!(convert_clickhouse_type_to_column_type("Nullable()").is_err());
        assert!(convert_clickhouse_type_to_column_type("Nullable(").is_err());

        // Test invalid Array format
        assert!(convert_clickhouse_type_to_column_type("Array").is_err());
        assert!(convert_clickhouse_type_to_column_type("Array()").is_err());
        assert!(convert_clickhouse_type_to_column_type("Array(").is_err());

        // Test invalid Enum format
        assert!(convert_clickhouse_type_to_column_type("Enum8").is_err());
        assert!(convert_clickhouse_type_to_column_type("Enum8()").is_err());
        assert!(convert_clickhouse_type_to_column_type("Enum8(").is_err());
        assert!(convert_clickhouse_type_to_column_type("Enum8('RED' = )").is_err());
        assert!(convert_clickhouse_type_to_column_type("Enum8('RED' = x)").is_err());

        // Test unsupported types
        assert!(convert_clickhouse_type_to_column_type("UUID").is_err());
        assert!(convert_clickhouse_type_to_column_type("IPv4").is_err());
        assert!(convert_clickhouse_type_to_column_type("IPv6").is_err());

        // Test nested type combinations
        assert!(convert_clickhouse_type_to_column_type("Array(Nullable())").is_err());
        assert!(convert_clickhouse_type_to_column_type("Nullable(Array())").is_err());
    }

    #[test]
    fn test_complex_type_combinations() {
        // Test Array of Nullable types
        let (array_type, is_nullable) =
            convert_clickhouse_type_to_column_type("Array(Nullable(Int32))").unwrap();
        assert!(!is_nullable); // Array itself is not nullable
        match array_type {
            ColumnType::Array {
                element_type,
                element_nullable,
            } => {
                assert_eq!(*element_type, ColumnType::Int);
                assert!(element_nullable); // Elements are nullable
            }
            _ => panic!("Expected Array type"),
        }

        // Test Nullable Array of Nullable types
        let (array_type, is_nullable) =
            convert_clickhouse_type_to_column_type("Nullable(Array(Nullable(Int32)))").unwrap();
        assert!(is_nullable); // Array itself is nullable
        match array_type {
            ColumnType::Array {
                element_type,
                element_nullable,
            } => {
                assert_eq!(*element_type, ColumnType::Int);
                assert!(element_nullable); // Elements are nullable
            }
            _ => panic!("Expected Array type"),
        }

        // Test Array of Enums
        let (array_type, is_nullable) =
            convert_clickhouse_type_to_column_type("Array(Enum8('RED' = 1, 'GREEN' = 2))").unwrap();
        assert!(!is_nullable);
        match array_type {
            ColumnType::Array {
                element_type,
                element_nullable,
            } => {
                match *element_type {
                    ColumnType::Enum(data_enum) => {
                        assert_eq!(data_enum.values.len(), 2);
                        assert_eq!(data_enum.values[0].name, "RED");
                        assert_eq!(data_enum.values[0].value, EnumValue::Int(1));
                    }
                    _ => panic!("Expected Enum type"),
                }
                assert!(!element_nullable);
            }
            _ => panic!("Expected Array type"),
        }

        // Test Nullable Array of Enums
        let (array_type, is_nullable) =
            convert_clickhouse_type_to_column_type("Nullable(Array(Enum8('RED' = 1)))").unwrap();
        assert!(is_nullable);
        match array_type {
            ColumnType::Array {
                element_type,
                element_nullable,
            } => {
                match *element_type {
                    ColumnType::Enum(data_enum) => {
                        assert_eq!(data_enum.values.len(), 1);
                        assert_eq!(data_enum.values[0].name, "RED");
                    }
                    _ => panic!("Expected Enum type"),
                }
                assert!(!element_nullable);
            }
            _ => panic!("Expected Array type"),
        }
    }
}

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
    drop_view_query,
};
use serde::{Deserialize, Serialize};

use self::model::ClickHouseSystemTable;
use crate::framework::core::infrastructure::sql_resource::SqlResource;
use crate::framework::core::infrastructure::table::{Column, ColumnType, Table};
use crate::framework::core::infrastructure::view::{View, ViewType};
use crate::framework::core::infrastructure_map::{PrimitiveSignature, PrimitiveTypes};
use crate::framework::core::partial_infrastructure_map::LifeCycle;
use crate::framework::versions::Version;
use crate::infrastructure::olap::clickhouse::model::ClickHouseSystemTableRow;
use crate::infrastructure::olap::{OlapChangesError, OlapOperations};
use crate::project::Project;

pub mod client;
pub mod config;
pub mod errors;
pub mod inserter;
pub mod mapper;
pub mod model;
pub mod queries;
pub mod type_parser;

pub use config::ClickHouseConfig;

use super::ddl_ordering::AtomicOlapOperation;

/// Type alias for query strings to improve readability
pub type QueryString = String;

/// Represents errors that can occur during ClickHouse operations
#[derive(Debug, thiserror::Error)]
pub enum ClickhouseChangesError {
    /// Error when interacting with ClickHouse database
    #[error("Error interacting with Clickhouse")]
    Clickhouse(#[from] ClickhouseError),

    /// Error from the ClickHouse client library
    #[error("Error interacting with Clickhouse{}", .resource.as_ref().map(|t| format!(" for '{t}'")).unwrap_or_default())]
    ClickhouseClient {
        #[source]
        error: clickhouse::error::Error,
        resource: Option<String>,
    },

    /// Error for unsupported operations
    #[error("Not Supported {0}")]
    NotSupported(String),
}

/// Executes a series of changes to the ClickHouse database schema
///
/// # Arguments
///
/// * `project` - The Project configuration containing ClickHouse connection details
/// * `teardown_plan` - A slice of AtomicOlapOperation representing the teardown plan
/// * `setup_plan` - A slice of AtomicOlapOperation representing the setup plan
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
    teardown_plan: &[AtomicOlapOperation],
    setup_plan: &[AtomicOlapOperation],
) -> Result<(), ClickhouseChangesError> {
    // Setup the client
    let client = create_client(project.clickhouse_config.clone());
    check_ready(&client)
        .await
        .map_err(|e| ClickhouseChangesError::ClickhouseClient {
            error: e,
            resource: None,
        })?;

    let db_name = &project.clickhouse_config.db_name;

    // Execute Teardown Plan
    info!(
        "Executing OLAP Teardown Plan with {} operations",
        teardown_plan.len()
    );
    debug!("Ordered Teardown plan: {:?}", teardown_plan);
    for op in teardown_plan {
        debug!("Teardown operation: {:?}", op);
        execute_atomic_operation(db_name, op, &client).await?;
    }

    // Execute Setup Plan
    info!(
        "Executing OLAP Setup Plan with {} operations",
        setup_plan.len()
    );
    debug!("Ordered Setup plan: {:?}", setup_plan);
    for op in setup_plan {
        debug!("Setup operation: {:?}", op);
        execute_atomic_operation(db_name, op, &client).await?;
    }

    info!("OLAP Change execution complete");
    Ok(())
}

/// Executes a single atomic OLAP operation.
async fn execute_atomic_operation(
    db_name: &str,
    operation: &AtomicOlapOperation,
    client: &ConfiguredDBClient,
) -> Result<(), ClickhouseChangesError> {
    match operation {
        AtomicOlapOperation::CreateTable { table, .. } => {
            execute_create_table(db_name, table, client).await?;
        }
        AtomicOlapOperation::DropTable { table, .. } => {
            execute_drop_table(db_name, table, client).await?;
        }
        AtomicOlapOperation::AddTableColumn {
            table,
            column,
            after_column,
            dependency_info: _,
        } => {
            execute_add_table_column(db_name, table, column, after_column, client).await?;
        }
        AtomicOlapOperation::DropTableColumn {
            table, column_name, ..
        } => {
            execute_drop_table_column(db_name, table, column_name, client).await?;
        }
        AtomicOlapOperation::ModifyTableColumn {
            table,
            column_name,
            before_data_type,
            after_data_type,
            before_required: _,
            after_required: _,
            dependency_info: _,
        } => {
            execute_modify_table_column(
                db_name,
                table,
                column_name,
                before_data_type,
                after_data_type,
                client,
            )
            .await?;
        }
        AtomicOlapOperation::CreateView {
            view,
            dependency_info: _,
        } => {
            execute_create_view(db_name, view, client).await?;
        }
        AtomicOlapOperation::DropView {
            view,
            dependency_info: _,
        } => {
            execute_drop_view(db_name, view, client).await?;
        }
        AtomicOlapOperation::RunSetupSql {
            resource,
            dependency_info: _,
        } => {
            execute_run_setup_sql(resource, client).await?;
        }
        AtomicOlapOperation::RunTeardownSql {
            resource,
            dependency_info: _,
        } => {
            execute_run_teardown_sql(resource, client).await?;
        }
    }
    Ok(())
}

async fn execute_create_table(
    db_name: &str,
    table: &Table,
    client: &ConfiguredDBClient,
) -> Result<(), ClickhouseChangesError> {
    log::info!("Executing CreateTable: {:?}", table.id());
    let clickhouse_table = std_table_to_clickhouse_table(table)?;
    let create_data_table_query = create_table_query(db_name, clickhouse_table)?;
    run_query(&create_data_table_query, client)
        .await
        .map_err(|e| ClickhouseChangesError::ClickhouseClient {
            error: e,
            resource: Some(table.name.clone()),
        })?;
    Ok(())
}

async fn execute_drop_table(
    db_name: &str,
    table: &Table,
    client: &ConfiguredDBClient,
) -> Result<(), ClickhouseChangesError> {
    log::info!("Executing DropTable: {:?}", table.id());
    let clickhouse_table = std_table_to_clickhouse_table(table)?;
    let drop_query = drop_table_query(db_name, clickhouse_table)?;
    run_query(&drop_query, client)
        .await
        .map_err(|e| ClickhouseChangesError::ClickhouseClient {
            error: e,
            resource: Some(table.name.clone()),
        })?;
    Ok(())
}

async fn execute_add_table_column(
    db_name: &str,
    table: &Table,
    column: &Column,
    after_column: &Option<String>,
    client: &ConfiguredDBClient,
) -> Result<(), ClickhouseChangesError> {
    log::info!(
        "Executing AddTableColumn for table: {}, column: {}, after: {:?}",
        table.id(),
        column.name,
        after_column
    );
    let clickhouse_column = std_column_to_clickhouse_column(column.clone())?;
    let column_type_string = basic_field_type_to_string(&clickhouse_column.column_type)?;

    let add_column_query = format!(
        "ALTER TABLE `{}`.`{}` ADD COLUMN `{}` {} {}",
        db_name,
        table.name,
        clickhouse_column.name,
        column_type_string,
        match after_column {
            None => "FIRST".to_string(),
            Some(after_col) => format!("AFTER `{after_col}`"),
        }
    );
    log::debug!("Adding column: {}", add_column_query);
    run_query(&add_column_query, client).await.map_err(|e| {
        ClickhouseChangesError::ClickhouseClient {
            error: e,
            resource: Some(table.name.clone()),
        }
    })?;
    Ok(())
}

async fn execute_drop_table_column(
    db_name: &str,
    table: &Table,
    column_name: &str,
    client: &ConfiguredDBClient,
) -> Result<(), ClickhouseChangesError> {
    log::info!(
        "Executing DropTableColumn for table: {}, column: {}",
        table.id(),
        column_name
    );
    let drop_column_query = format!(
        "ALTER TABLE `{}`.`{}` DROP COLUMN IF EXISTS `{}`",
        db_name, table.name, column_name
    );
    log::debug!("Dropping column: {}", drop_column_query);
    run_query(&drop_column_query, client).await.map_err(|e| {
        ClickhouseChangesError::ClickhouseClient {
            error: e,
            resource: Some(table.name.clone()),
        }
    })?;
    Ok(())
}

/// Execute a ModifyTableColumn operation
async fn execute_modify_table_column(
    db_name: &str,
    table: &Table,
    column_name: &str,
    before_data_type: &ColumnType,
    after_data_type: &ColumnType,
    client: &ConfiguredDBClient,
) -> Result<(), ClickhouseChangesError> {
    log::info!(
        "Executing ModifyTableColumn for table: {}, column: {} ({}→{})",
        table.id(),
        column_name,
        before_data_type.to_string(),
        after_data_type.to_string()
    );

    // Reconstruct the 'after' Column to convert it to ClickHouse type
    let reconstructed_after_column = Column {
        name: column_name.to_string(),
        data_type: after_data_type.clone(),
        required: false,
        unique: false,
        primary_key: false,
        default: None,
        annotations: vec![],
    };

    let clickhouse_column = std_column_to_clickhouse_column(reconstructed_after_column)?;
    let column_type_string = basic_field_type_to_string(&clickhouse_column.column_type)?;
    // TODO: Fix the string conversion issue here - expected String, found &str
    let modify_column_query = format!(
        "ALTER TABLE `{}`.`{}` MODIFY COLUMN IF EXISTS `{}` {}",
        db_name, table.name, clickhouse_column.name, column_type_string
    );
    log::debug!("Modifying column: {}", modify_column_query);
    run_query(&modify_column_query, client).await.map_err(|e| {
        ClickhouseChangesError::ClickhouseClient {
            error: e,
            resource: Some(table.name.clone()),
        }
    })?;
    Ok(())
}

async fn execute_create_view(
    db_name: &str,
    view: &View,
    client: &ConfiguredDBClient,
) -> Result<(), ClickhouseChangesError> {
    log::info!("Executing CreateView: {:?}", view.id());
    match &view.view_type {
        ViewType::TableAlias { source_table_name } => {
            let create_view_query = create_view_query(
                db_name,
                &view.id(),
                &format!("SELECT * FROM `{db_name}`.`{source_table_name}`"),
            )?;
            run_query(&create_view_query, client).await.map_err(|e| {
                ClickhouseChangesError::ClickhouseClient {
                    error: e,
                    resource: Some(view.id()),
                }
            })?;
        }
    }
    Ok(())
}

async fn execute_drop_view(
    db_name: &str,
    view: &View,
    client: &ConfiguredDBClient,
) -> Result<(), ClickhouseChangesError> {
    log::info!("Executing DropView: {:?}", view.id());
    match &view.view_type {
        ViewType::TableAlias { .. } => {
            let delete_view_query = drop_view_query(db_name, &view.id())?;
            run_query(&delete_view_query, client).await.map_err(|e| {
                ClickhouseChangesError::ClickhouseClient {
                    error: e,
                    resource: Some(view.id()),
                }
            })?;
        }
    }
    Ok(())
}

async fn execute_run_setup_sql(
    resource: &SqlResource,
    client: &ConfiguredDBClient,
) -> Result<(), ClickhouseChangesError> {
    log::info!("Executing RunSetupSql for resource: {:?}", resource.name);
    for query in &resource.setup {
        run_query(query, client)
            .await
            .map_err(|e| ClickhouseChangesError::ClickhouseClient {
                error: e,
                resource: Some(resource.name.clone()),
            })?;
    }
    Ok(())
}

async fn execute_run_teardown_sql(
    resource: &SqlResource,
    client: &ConfiguredDBClient,
) -> Result<(), ClickhouseChangesError> {
    log::info!("Executing RunTeardownSql for resource: {:?}", resource.name);
    for query in &resource.teardown {
        run_query(query, client)
            .await
            .map_err(|e| ClickhouseChangesError::ClickhouseClient {
                error: e,
                resource: Some(resource.name.clone()),
            })?;
    }
    Ok(())
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
fn extract_version_from_table_name(table_name: &str) -> (String, Option<Version>) {
    debug!("Extracting version from table name: {}", table_name);

    // Special case for empty table name
    if table_name.is_empty() {
        debug!("Empty table name, no version");
        return (table_name.to_string(), None);
    }

    // Special case for tables ending in _MV (materialized views)
    if table_name.ends_with("_MV") {
        debug!("Materialized view detected, skipping version parsing");
        return (table_name.to_string(), None);
    }

    let parts: Vec<&str> = table_name.split('_').collect();
    debug!("Split table name into parts: {:?}", parts);

    if parts.len() < 2 {
        debug!("Table name has fewer than 2 parts, no version");
        // If table doesn't follow naming convention, return full name and default version
        return (table_name.to_string(), None);
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
                debug!("No valid version parts found.");
                return (table_name.to_string(), None);
            }

            let version_str = version_parts.join(".");
            debug!("Created version string: {}", version_str);

            (base_name, Some(Version::from_string(version_str)))
        }
        None => {
            debug!("No version parts found");
            (table_name.to_string(), None)
        }
    }
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
            .with_option("enable_json_type", "1")
            .with_option("flatten_nested", "0"),
        config: clickhouse_config,
    }
}

/// Executes a SQL query against the ClickHouse database
///
/// # Arguments
/// * `query` - The SQL query to execute
/// * `configured_client` - The client to use for execution
///
/// # Returns
/// * `Result<(), clickhouse::error::Error>` - Success if query executes without error
///
/// # Example
/// ```
/// let query = "SELECT 1";
/// run_query(query, &client).await?;
/// ```
pub async fn run_query(
    query: &str,
    configured_client: &ConfiguredDBClient,
) -> Result<(), clickhouse::error::Error> {
    debug!("Running query: {:?}", query);
    let client = &configured_client.client;
    client.query(query).execute().await
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

pub struct TableWithUnsupportedType {
    pub name: String,
    pub col_name: String,
    pub col_type: String,
}

#[async_trait::async_trait]
impl OlapOperations for ConfiguredDBClient {
    /// Retrieves all tables from the ClickHouse database and converts them to framework Table objects
    ///
    /// # Arguments
    /// * `db_name` - The name of the database to list tables from
    ///
    /// # Returns
    /// * `Result<(Vec<Table>, Vec<TableWithUnsupportedType>), OlapChangesError>` -
    /// A list of Table objects and a list of TableWithUnsupportedType on success
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
    ) -> Result<(Vec<Table>, Vec<TableWithUnsupportedType>), OlapChangesError> {
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
            WHERE database = '{db_name}' 
            AND engine != 'View' 
            AND engine != 'MaterializedView'
            AND NOT name LIKE '.%'
            ORDER BY name
            "#
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
        let mut unsupported_tables = Vec::new();

        'table_loop: while let Some((table_name, engine, create_query)) = cursor
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

            // Check if the CREATE TABLE statement has an explicit PRIMARY KEY clause
            let has_explicit_primary_key = create_query.to_uppercase().contains("PRIMARY KEY");
            debug!(
                "Table {} has explicit PRIMARY KEY: {}",
                table_name, has_explicit_primary_key
            );

            // Get column information for each table
            let columns_query = format!(
                r#"
                SELECT
                    name,
                    type,
                    is_in_primary_key,
                    is_in_sorting_key
                FROM system.columns
                WHERE database = '{db_name}'
                AND table = '{table_name}'
                ORDER BY position
                "#
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

                let (data_type, is_nullable) =
                    match type_parser::convert_clickhouse_type_to_column_type(&col_type) {
                        Ok(pair) => pair,
                        Err(_) => {
                            debug!(
                                "Column type not recognized: {} of field {} in table {}",
                                col_type, col_name, table_name
                            );
                            unsupported_tables.push(TableWithUnsupportedType {
                                name: table_name,
                                col_name,
                                col_type,
                            });
                            continue 'table_loop;
                        }
                    };

                // Only set primary_key=true if there's an explicit PRIMARY KEY clause
                // When only ORDER BY is specified (no PRIMARY KEY), ClickHouse internally
                // treats ORDER BY columns as primary key, but we shouldn't mark them as such
                // since they come from orderByFields configuration, not Key<T> annotations
                let is_actual_primary_key = has_explicit_primary_key && is_primary == 1;

                let column = Column {
                    name: col_name.clone(),
                    data_type,
                    required: !is_nullable,
                    unique: false,
                    primary_key: is_actual_primary_key,
                    default: None,
                    annotations: Default::default(),
                };

                columns.push(column);
            }

            debug!("Found {} columns for table {}", columns.len(), table_name);

            // Extract base name and version for source primitive
            let (base_name, version) = extract_version_from_table_name(&table_name);

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
                engine: Some(engine),
                version,
                source_primitive,
                metadata: None,
                // this does not matter as we refer to the lifecycle in infra map
                life_cycle: LifeCycle::ExternallyManaged,
            };
            debug!("Created table object: {:?}", table);

            tables.push(table);
        }

        debug!(
            "Completed list_tables operation, found {} tables",
            tables.len()
        );
        Ok((tables, unsupported_tables))
    }
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

    #[test]
    fn test_extract_version_from_table_name() {
        // Test two-part versions
        let (base_name, version) = extract_version_from_table_name("Bar_0_0");
        assert_eq!(base_name, "Bar");
        assert_eq!(version.unwrap().to_string(), "0.0");

        let (base_name, version) = extract_version_from_table_name("Foo_0_0");
        assert_eq!(base_name, "Foo");
        assert_eq!(version.unwrap().to_string(), "0.0");

        // Test three-part versions
        let (base_name, version) = extract_version_from_table_name("Bar_0_0_0");
        assert_eq!(base_name, "Bar");
        assert_eq!(version.unwrap().to_string(), "0.0.0");

        let (base_name, version) = extract_version_from_table_name("Foo_1_2_3");
        assert_eq!(base_name, "Foo");
        assert_eq!(version.unwrap().to_string(), "1.2.3");

        // Test table names with underscores
        let (base_name, version) = extract_version_from_table_name("My_Table_0_0");
        assert_eq!(base_name, "My_Table");
        assert_eq!(version.unwrap().to_string(), "0.0");

        let (base_name, version) = extract_version_from_table_name("Complex_Table_Name_1_0_0");
        assert_eq!(base_name, "Complex_Table_Name");
        assert_eq!(version.unwrap().to_string(), "1.0.0");

        // Test invalid formats - should use default version
        let (base_name, version) = extract_version_from_table_name("TableWithoutVersion");
        assert_eq!(base_name, "TableWithoutVersion");
        assert!(version.is_none());

        let (base_name, version) = extract_version_from_table_name("Table_WithoutNumericVersion");
        assert_eq!(base_name, "Table_WithoutNumericVersion");
        assert!(version.is_none());

        // Test edge cases
        let (base_name, version) = extract_version_from_table_name("");
        assert_eq!(base_name, "");
        assert!(version.is_none());

        let (base_name, version) = extract_version_from_table_name("_0_0");
        assert_eq!(base_name, "");
        assert_eq!(version.unwrap().to_string(), "0.0");

        let (base_name, version) = extract_version_from_table_name("Table_0_0_");
        assert_eq!(base_name, "Table");
        assert_eq!(version.unwrap().to_string(), "0.0");

        // Test mixed numeric and non-numeric parts
        let (base_name, version) = extract_version_from_table_name("Table2_0_0");
        assert_eq!(base_name, "Table2");
        assert_eq!(version.unwrap().to_string(), "0.0");

        let (base_name, version) = extract_version_from_table_name("V2_Table_1_0_0");
        assert_eq!(base_name, "V2_Table");
        assert_eq!(version.unwrap().to_string(), "1.0.0");

        // Test materialized views
        let (base_name, version) = extract_version_from_table_name("BarAggregated_MV");
        assert_eq!(base_name, "BarAggregated_MV");
        assert!(version.is_none());

        // Test non-versioned tables
        let (base_name, version) = extract_version_from_table_name("Foo");
        assert_eq!(base_name, "Foo");
        assert!(version.is_none());

        let (base_name, version) = extract_version_from_table_name("Bar");
        assert_eq!(base_name, "Bar");
        assert!(version.is_none());
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
}

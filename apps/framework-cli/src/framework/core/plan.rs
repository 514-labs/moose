/// # Infrastructure Planning Module
///
/// This module is responsible for planning infrastructure changes by comparing the current
/// infrastructure state with the target state. It generates a plan that describes the
/// changes needed to transition from the current state to the target state.
///
/// The planning process involves:
/// 1. Loading the current infrastructure map from Redis
/// 2. Reconciling the infrastructure map with the actual database state
/// 3. Building the target infrastructure map from the project configuration
/// 4. Computing the difference between the reconciled and target maps
/// 5. Creating a plan that describes the changes to be applied
///
/// The resulting plan is then used by the execution module to apply the changes.
use crate::framework::core::infra_reality_checker::{InfraRealityChecker, RealityCheckError};
use crate::framework::core::infrastructure_map::{
    InfraChanges, InfrastructureMap, OlapChange, TableChange,
};
use crate::framework::core::primitive_map::PrimitiveMap;
use crate::infrastructure::olap::OlapOperations;
use crate::infrastructure::{olap::clickhouse, redis::redis_client::RedisClient};
use crate::project::Project;
use log::{debug, error, info};
use rdkafka::error::KafkaError;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Errors that can occur during the planning process.
#[derive(Debug, thiserror::Error)]
pub enum PlanningError {
    /// Error occurred while loading the primitive map
    #[error("Failed to load primitive map")]
    PrimitiveMapLoading(#[from] crate::framework::core::primitive_map::PrimitiveMapLoadingError),

    /// Error occurred while connecting to the Clickhouse database
    #[error("Failed to connect to state storage")]
    Clickhouse(#[from] clickhouse_rs::errors::Error),

    /// Error occurred while connecting to Kafka
    #[error("Failed to connect to streaming engine")]
    Kafka(#[from] KafkaError),

    /// Error occurred during reality check
    #[error("Failed during reality check")]
    RealityCheck(#[from] RealityCheckError),

    /// Other unspecified errors
    #[error("Unknown error")]
    Other(#[from] anyhow::Error),
}
/// Reconciles an infrastructure map with the actual state from the database.
///
/// This function uses the InfraRealityChecker to determine the actual state of the database
/// and updates the provided infrastructure map to match reality. This ensures that any
/// external changes made to the database are properly reflected in the infrastructure map
/// before planning and applying new changes.
///
/// We only want to look at differences for tables that are already in the infrastructure map.
/// This is because if new external tables appear, they might not be in the code, yet. As such
/// we don't want those to be deleted as a consequence of the diff
///
/// # Arguments
/// * `project` - The project configuration
/// * `infra_map` - The infrastructure map to update
/// * `olap_client` - The OLAP client to use for checking reality
///
/// # Returns
/// * `Result<InfrastructureMap, PlanningError>` - The reconciled infrastructure map or an error
async fn reconcile_with_reality<T: OlapOperations>(
    project: &Project,
    infra_map: &InfrastructureMap,
    olap_client: T,
) -> Result<InfrastructureMap, PlanningError> {
    info!("Reconciling infrastructure map with actual database state");

    // Create the reality checker with the provided client
    let reality_checker = InfraRealityChecker::new(olap_client);

    // Get the discrepancies between the infra map and the actual database
    let discrepancies = reality_checker.check_reality(project, infra_map).await?;

    // If there are no discrepancies, return the original map
    if discrepancies.is_empty() {
        debug!("No discrepancies found between infrastructure map and actual database state");
        return Ok(infra_map.clone());
    }

    debug!(
        "Reconciling {} missing tables and {} mismatched tables",
        discrepancies.missing_tables.len(),
        discrepancies.mismatched_tables.len(),
    );

    // Clone the map so we can modify it
    let mut reconciled_map = infra_map.clone();

    // Remove missing tables from the map so that they can be re-created
    // if they are added to the codebase
    for table_name in &discrepancies.missing_tables {
        debug!(
            "Removing missing table {} from infrastructure map",
            table_name
        );
        // Find the table by name and remove it by ID
        if let Some((id, _)) = reconciled_map
            .tables
            .iter()
            .find(|(_, table)| &table.name == table_name)
            .map(|(id, _)| (id.clone(), ()))
        {
            reconciled_map.tables.remove(&id);
        }
    }

    // Update mismatched tables
    for change in &discrepancies.mismatched_tables {
        match change {
            OlapChange::Table(table_change) => {
                match table_change {
                    TableChange::Updated { before, .. } => {
                        debug!(
                            "Updating table {} in infrastructure map to match reality",
                            before.name
                        );
                        reconciled_map.tables.insert(before.id(), before.clone());
                    }
                    _ => {
                        // Other table changes (Add/Remove) are already handled by unmapped/missing
                        debug!("Skipping table change: {:?}", table_change);
                    }
                }
            }
            _ => {
                // We only handle table changes for now
                debug!("Skipping non-table change: {:?}", change);
            }
        }
    }

    info!("Infrastructure map successfully reconciled with actual database state");
    Ok(reconciled_map)
}

/// Represents a plan for infrastructure changes.
///
/// This struct contains the target infrastructure map and the changes needed
/// to transition from the current state to the target state.
#[derive(Debug, Serialize, Deserialize)]
pub struct InfraPlan {
    /// The target infrastructure map that we want to achieve
    pub target_infra_map: InfrastructureMap,

    /// The changes needed to transition from the current state to the target state
    pub changes: InfraChanges,
}

/// Plans infrastructure changes by comparing the current state with the target state.
///
/// This function loads the current infrastructure map from Redis, reconciles it with the
/// actual database state, and compares it with the target infrastructure map derived
/// from the project configuration. It then generates a plan that describes the changes
/// needed to transition from the current state to the target state.
///
/// # Arguments
/// * `client` - Redis client for loading the current infrastructure map
/// * `project` - Project configuration for building the target infrastructure map
///
/// # Returns
/// * `Result<InfraPlan, PlanningError>` - The infrastructure plan or an error
pub async fn plan_changes(
    client: &RedisClient,
    project: &Project,
) -> Result<InfraPlan, PlanningError> {
    let json_path = Path::new(".moose/infrastructure_map.json");
    let target_infra_map = if project.is_production && json_path.exists() {
        InfrastructureMap::load_from_json(json_path).map_err(|e| PlanningError::Other(e.into()))?
    } else {
        if project.is_production && project.is_docker_image() {
            error!("Docker Build images should have the infrastructure map already created and embedded");
        }

        if project.features.data_model_v2 {
            InfrastructureMap::load_from_user_code(project).await?
        } else {
            let primitive_map = PrimitiveMap::load(project).await?;
            InfrastructureMap::new(project, primitive_map)
        }
    };

    let current_infra_map = InfrastructureMap::load_from_redis(client).await?;

    debug!(
        "Current infrastructure map: {}",
        serde_json::to_string(&current_infra_map)
            .unwrap_or("Could not serialize current infrastructure map".to_string())
    );

    // Plan changes, reconciling with reality if we have a current infrastructure map
    let plan = match &current_infra_map {
        Some(current_map) => {
            // Create the OLAP client for reality checks
            let olap_client = clickhouse::create_client(project.clickhouse_config.clone());

            // Reconcile the current map with reality before diffing
            let reconciled_map = reconcile_with_reality(project, current_map, olap_client).await?;

            debug!(
                "Reconciled infrastructure map: {}",
                serde_json::to_string(&reconciled_map)
                    .unwrap_or("Could not serialize reconciled infrastructure map".to_string())
            );

            // Use the reconciled map for diffing
            InfraPlan {
                target_infra_map: target_infra_map.clone(),
                changes: reconciled_map.diff(&target_infra_map),
            }
        }
        None => {
            // No current map, so we're initializing from scratch
            InfraPlan {
                target_infra_map: target_infra_map.clone(),
                changes: target_infra_map.init(project),
            }
        }
    };

    debug!(
        "Plan Changes: {}",
        serde_json::to_string(&plan.changes)
            .unwrap_or("Could not serialize plan changes".to_string())
    );

    Ok(plan)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framework::core::infrastructure::table::{Column, ColumnType, IntType, Table};
    use crate::framework::core::infrastructure_map::{PrimitiveSignature, PrimitiveTypes};
    use crate::framework::versions::Version;
    use crate::infrastructure::olap::clickhouse::TableWithUnsupportedType;
    use crate::infrastructure::olap::OlapChangesError;
    use crate::infrastructure::olap::OlapOperations;
    use async_trait::async_trait;

    // Mock OLAP client for testing
    struct MockOlapClient {
        tables: Vec<Table>,
    }

    #[async_trait]
    impl OlapOperations for MockOlapClient {
        async fn list_tables(
            &self,
            _db_name: &str,
            _project: &Project,
        ) -> Result<(Vec<Table>, Vec<TableWithUnsupportedType>), OlapChangesError> {
            Ok((self.tables.clone(), vec![]))
        }
    }

    // Helper function to create a test table
    fn create_test_table(name: &str) -> Table {
        Table {
            name: name.to_string(),
            columns: vec![Column {
                name: "id".to_string(),
                data_type: ColumnType::Int(IntType::Int64),
                required: true,
                unique: true,
                primary_key: true,
                default: None,
                annotations: vec![],
            }],
            order_by: vec!["id".to_string()],
            engine: None,
            deduplicate: false,
            version: Some(Version::from_string("1.0.0".to_string())),
            source_primitive: PrimitiveSignature {
                name: "test".to_string(),
                primitive_type: PrimitiveTypes::DataModel,
            },
            metadata: None,
        }
    }

    // Helper function to create a test project
    fn create_test_project() -> Project {
        Project {
            language: crate::framework::languages::SupportedLanguages::Typescript,
            redpanda_config: crate::infrastructure::stream::kafka::models::KafkaConfig::default(),
            clickhouse_config: crate::infrastructure::olap::clickhouse::ClickHouseConfig {
                db_name: "test".to_string(),
                user: "test".to_string(),
                password: "test".to_string(),
                use_ssl: false,
                host: "localhost".to_string(),
                host_port: 18123,
                native_port: 9000,
                host_data_path: None,
            },
            http_server_config: crate::cli::local_webserver::LocalWebserverConfig::default(),
            redis_config: crate::infrastructure::redis::redis_client::RedisConfig::default(),
            git_config: crate::utilities::git::GitConfig::default(),
            temporal_config:
                crate::infrastructure::orchestration::temporal::TemporalConfig::default(),
            language_project_config: crate::project::LanguageProjectConfig::default(),
            project_location: std::path::PathBuf::new(),
            is_production: false,
            supported_old_versions: std::collections::HashMap::new(),
            jwt: None,
            authentication: crate::project::AuthenticationConfig::default(),

            features: crate::project::ProjectFeatures::default(),
        }
    }

    #[tokio::test]
    async fn test_reconcile_with_reality_unmapped_table() {
        // Create a test table that exists in the database but not in the infra map
        let table = create_test_table("unmapped_table");

        // Create mock OLAP client with one table
        let mock_client = MockOlapClient {
            tables: vec![table.clone()],
        };

        // Create empty infrastructure map (no tables)
        let infra_map = InfrastructureMap::default();

        // Replace the normal check_reality function with our mock
        let reality_checker = InfraRealityChecker::new(mock_client);

        // Create test project
        let project = create_test_project();

        // Get the discrepancies
        let discrepancies = reality_checker
            .check_reality(&project, &infra_map)
            .await
            .unwrap();

        // There should be one unmapped table
        assert_eq!(discrepancies.unmapped_tables.len(), 1);
        assert_eq!(discrepancies.unmapped_tables[0].name, "unmapped_table");

        // Create another mock client for the reconciliation
        let reconcile_mock_client = MockOlapClient {
            tables: vec![table.clone()],
        };

        // Reconcile the infrastructure map
        let reconciled = reconcile_with_reality(&project, &infra_map, reconcile_mock_client)
            .await
            .unwrap();

        // The reconciled map should not contain the unmapped table (ignoring unmapped tables)
        assert_eq!(reconciled.tables.len(), 0);
    }

    #[tokio::test]
    async fn test_reconcile_with_reality_missing_table() {
        // Create a test table that exists in the infra map but not in the database
        let table = create_test_table("missing_table");

        // Create mock OLAP client with no tables
        let mock_client = MockOlapClient { tables: vec![] };

        // Create infrastructure map with one table
        let mut infra_map = InfrastructureMap::default();
        infra_map.tables.insert(table.id(), table.clone());

        // Replace the normal check_reality function with our mock
        let reality_checker = InfraRealityChecker::new(mock_client);

        // Create test project
        let project = create_test_project();

        // Get the discrepancies
        let discrepancies = reality_checker
            .check_reality(&project, &infra_map)
            .await
            .unwrap();

        // There should be one missing table
        assert_eq!(discrepancies.missing_tables.len(), 1);
        assert_eq!(discrepancies.missing_tables[0], "missing_table");

        // Create another mock client for the reconciliation
        let reconcile_mock_client = MockOlapClient { tables: vec![] };

        // Reconcile the infrastructure map
        let reconciled = reconcile_with_reality(&project, &infra_map, reconcile_mock_client)
            .await
            .unwrap();

        // The reconciled map should have no tables
        assert_eq!(reconciled.tables.len(), 0);
    }

    #[tokio::test]
    async fn test_reconcile_with_reality_mismatched_table() {
        // Create two versions of the same table with different columns
        let infra_table = create_test_table("mismatched_table");
        let mut actual_table = create_test_table("mismatched_table");

        // Add an extra column to the actual table that's not in infra map
        actual_table.columns.push(Column {
            name: "extra_column".to_string(),
            data_type: ColumnType::String,
            required: false,
            unique: false,
            primary_key: false,
            default: None,
            annotations: vec![],
        });

        // Create mock OLAP client with the actual table
        let mock_client = MockOlapClient {
            tables: vec![actual_table.clone()],
        };

        // Create infrastructure map with the infra table (no extra column)
        let mut infra_map = InfrastructureMap::default();
        infra_map
            .tables
            .insert(infra_table.id(), infra_table.clone());

        // Replace the normal check_reality function with our mock
        let reality_checker = InfraRealityChecker::new(mock_client);

        // Create test project
        let project = create_test_project();

        // Get the discrepancies
        let discrepancies = reality_checker
            .check_reality(&project, &infra_map)
            .await
            .unwrap();

        // There should be one mismatched table
        assert_eq!(discrepancies.mismatched_tables.len(), 1);

        // Create another mock client for reconciliation
        let reconcile_mock_client = MockOlapClient {
            tables: vec![actual_table.clone()],
        };

        // Reconcile the infrastructure map
        let reconciled = reconcile_with_reality(&project, &infra_map, reconcile_mock_client)
            .await
            .unwrap();

        // The reconciled map should have one table with the extra column
        assert_eq!(reconciled.tables.len(), 1);
        let reconciled_table = reconciled.tables.values().next().unwrap();
        assert_eq!(reconciled_table.columns.len(), 2); // id + extra_column
        assert!(reconciled_table
            .columns
            .iter()
            .any(|c| c.name == "extra_column"));
    }

    #[tokio::test]
    async fn test_reconcile_with_reality_no_changes() {
        // Create a test table that exists in both the infra map and the database
        let table = create_test_table("unchanged_table");

        // Create mock OLAP client with the table
        let mock_client = MockOlapClient {
            tables: vec![table.clone()],
        };

        // Create infrastructure map with the same table
        let mut infra_map = InfrastructureMap::default();
        infra_map.tables.insert(table.id(), table.clone());

        // Replace the normal check_reality function with our mock
        let reality_checker = InfraRealityChecker::new(mock_client);

        // Create test project
        let project = create_test_project();

        // Get the discrepancies
        let discrepancies = reality_checker
            .check_reality(&project, &infra_map)
            .await
            .unwrap();

        // There should be no discrepancies
        assert!(discrepancies.is_empty());

        // Create another mock client for reconciliation
        let reconcile_mock_client = MockOlapClient {
            tables: vec![table.clone()],
        };

        // Reconcile the infrastructure map
        let reconciled = reconcile_with_reality(&project, &infra_map, reconcile_mock_client)
            .await
            .unwrap();

        // The reconciled map should be unchanged
        assert_eq!(reconciled.tables.len(), 1);
        assert!(reconciled
            .tables
            .values()
            .any(|t| t.name == "unchanged_table"));
        // Compare the tables to ensure they are identical
        assert_eq!(reconciled.tables.values().next().unwrap(), &table);
    }
}

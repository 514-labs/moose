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
use super::{
    infra_reality_checker::{InfraDiscrepancies, InfraRealityChecker, RealityCheckError},
    infrastructure_map::{InfraChanges, InfrastructureMap},
    primitive_map::PrimitiveMap,
};
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
/// # Arguments
/// * `project` - The project configuration
/// * `infra_map` - The infrastructure map to update
///
/// # Returns
/// * `Result<InfrastructureMap, PlanningError>` - The reconciled infrastructure map or an error
async fn reconcile_with_reality(
    project: &Project,
    infra_map: &InfrastructureMap,
) -> Result<InfrastructureMap, PlanningError> {
    info!("Reconciling infrastructure map with actual database state");

    // Create the OLAP client and reality checker
    let olap_client = clickhouse::create_client(project.clickhouse_config.clone());
    let reality_checker = InfraRealityChecker::new(olap_client);

    // Get the discrepancies between the infra map and the actual database
    let discrepancies = reality_checker.check_reality(project, infra_map).await?;

    // If there are no discrepancies, return the original map
    if discrepancies.is_empty() {
        debug!("No discrepancies found between infrastructure map and actual database state");
        return Ok(infra_map.clone());
    }

    debug!(
        "Reconciling {} unmapped tables, {} missing tables, and {} mismatched tables",
        discrepancies.unmapped_tables.len(),
        discrepancies.missing_tables.len(),
        discrepancies.mismatched_tables.len()
    );

    // Clone the map so we can modify it
    let mut reconciled_map = infra_map.clone();

    // Add unmapped tables to the map
    for table in &discrepancies.unmapped_tables {
        debug!("Adding unmapped table {} to infrastructure map", table.name);
        reconciled_map.tables.insert(table.id(), table.clone());
    }

    // Remove missing tables from the map
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
            super::infrastructure_map::OlapChange::Table(table_change) => {
                match table_change {
                    super::infrastructure_map::TableChange::Updated { before, .. } => {
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
            // Reconcile the current map with reality before diffing
            let reconciled_map = reconcile_with_reality(project, current_map).await?;

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

/// Plans infrastructure changes using provided infrastructure maps.
///
/// This function compares the current infrastructure map with the target infrastructure map,
/// reconciling the current map with reality if provided, and generates a plan that describes
/// the changes needed to transition from the current state to the target state.
///
/// # Arguments
/// * `project` - Project configuration
/// * `current_infra_map` - The current infrastructure map (if any)
/// * `target_infra_map` - The target infrastructure map
///
/// # Returns
/// * `Result<InfraPlan, PlanningError>` - The infrastructure plan or an error
pub async fn plan_changes_from_infra_map(
    project: &Project,
    current_infra_map: &Option<InfrastructureMap>,
    target_infra_map: &InfrastructureMap,
) -> Result<InfraPlan, PlanningError> {
    let changes = match current_infra_map {
        Some(current_infra_map) => {
            // Reconcile with reality before diffing
            let reconciled_map = reconcile_with_reality(project, current_infra_map).await?;
            reconciled_map.diff(target_infra_map)
        }
        None => target_infra_map.init(project),
    };

    Ok(InfraPlan {
        target_infra_map: target_infra_map.clone(),
        changes,
    })
}

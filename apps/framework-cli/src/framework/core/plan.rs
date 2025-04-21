/// # Infrastructure Planning Module
///
/// This module is responsible for planning infrastructure changes by comparing the current
/// infrastructure state with the target state. It generates a plan that describes the
/// changes needed to transition from the current state to the target state.
///
/// The planning process involves:
/// 1. Loading the current infrastructure map from Redis
/// 2. Building the target infrastructure map from the project configuration
/// 3. Computing the difference between the current and target maps
/// 4. Creating a plan that describes the changes to be applied
///
/// The resulting plan is then used by the execution module to apply the changes.
use super::{
    infrastructure_map::{InfraChanges, InfrastructureMap},
    primitive_map::PrimitiveMap,
};
use crate::infrastructure::redis::redis_client::RedisClient;
use crate::project::Project;
use log::{debug, error};
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

    /// Other unspecified errors
    // TODO: refactor the called functions
    #[error("Unknown error")]
    Other(#[from] anyhow::Error),
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
/// This function loads the current infrastructure map from Redis and compares it with
/// the target infrastructure map derived from the project configuration. It then
/// generates a plan that describes the changes needed to transition from the current
/// state to the target state.
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
        "Current infrastructure map: {:?}",
        serde_json::to_string(&current_infra_map)
    );

    plan_changes_from_infra_map(project, &current_infra_map, &target_infra_map).await
}

/// Plans infrastructure changes using provided infrastructure maps.
///
/// This function compares the current infrastructure map with the target infrastructure map
/// and generates a plan that describes the changes needed to transition from the current
/// state to the target state.
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
        Some(current_infra_map) => current_infra_map.diff(target_infra_map),
        None => target_infra_map.init(project),
    };

    Ok(InfraPlan {
        // current_infra_map,
        target_infra_map: target_infra_map.clone(),
        changes,
    })
}

use clickhouse_rs::ClientHandle;

use crate::{
    infrastructure::olap::clickhouse_alt_client::{retrieve_infrastructure_map, StateStorageError},
    project::Project,
};

use super::{
    infrastructure_map::{InfraChanges, InfrastructureMap},
    primitive_map::PrimitiveMap,
};

#[derive(Debug, thiserror::Error)]
pub enum PlanningError {
    #[error("Failed to communicate with state storage")]
    StateStorage(#[from] StateStorageError),

    #[error("Failed to load primitive map")]
    PrimitiveMapLoading(#[from] crate::framework::core::primitive_map::PrimitiveMapLoadingError),

    #[error("Failed to connect to state storage")]
    Clickhouse(#[from] clickhouse_rs::errors::Error),
}

pub struct InfraPlan {
    // pub current_infra_map: Option<InfrastructureMap>,
    pub target_infra_map: InfrastructureMap,

    pub changes: InfraChanges,
}

pub async fn plan_changes(
    client: &mut ClientHandle,
    project: &Project,
) -> Result<InfraPlan, PlanningError> {
    let primitive_map = PrimitiveMap::load(project)?;
    let target_infra_map = InfrastructureMap::new(primitive_map);

    let current_infra_map = retrieve_infrastructure_map(client, &project.clickhouse_config).await?;

    let changes = match &current_infra_map {
        Some(current_infra_map) => current_infra_map.diff(&target_infra_map),
        None => target_infra_map.init(),
    };

    Ok(InfraPlan {
        // current_infra_map,
        target_infra_map,
        changes,
    })
}

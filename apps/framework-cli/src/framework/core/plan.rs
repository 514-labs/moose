use super::{
    infrastructure_map::{InfraChanges, InfrastructureMap},
    primitive_map::PrimitiveMap,
};
use crate::framework::controller::{
    check_topic_fully_populated, InitialDataLoad, InitialDataLoadStatus,
};
use crate::{
    infrastructure::olap::clickhouse_alt_client::{retrieve_infrastructure_map, StateStorageError},
    project::Project,
};
use clickhouse_rs::ClientHandle;
use rdkafka::error::KafkaError;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum PlanningError {
    #[error("Failed to communicate with state storage")]
    StateStorage(#[from] StateStorageError),

    #[error("Failed to load primitive map")]
    PrimitiveMapLoading(#[from] crate::framework::core::primitive_map::PrimitiveMapLoadingError),

    #[error("Failed to connect to state storage")]
    Clickhouse(#[from] clickhouse_rs::errors::Error),

    #[error("Failed to connect to streaming engine")]
    Kafka(#[from] KafkaError),

    // TODO: refactor the called functions
    #[error("Unknown error")]
    Other(#[from] anyhow::Error),
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
    let json_path = Path::new(".moose/infrastructure_map.json");
    let mut target_infra_map = if project.is_production && json_path.exists() {
        InfrastructureMap::load_from_json(json_path).map_err(|e| PlanningError::Other(e.into()))?
    } else {
        let primitive_map = PrimitiveMap::load(project).await?;
        InfrastructureMap::new(primitive_map)
    };

    target_infra_map.with_topic_namespace(&project.redpanda_config);

    let current_infra_map = {
        // in the rest of this block of code,
        // we check the actual configurations and compare it to the stored state
        let mut current = retrieve_infrastructure_map(client, &project.clickhouse_config).await?;

        // currently we check only the initial data load statuses
        let existing_data_loads: &mut HashMap<String, InitialDataLoad> = match &mut current {
            None => &mut HashMap::new(),
            Some(existing) => &mut existing.initial_data_loads,
        };

        for (id, load) in target_infra_map.initial_data_loads.iter() {
            match existing_data_loads.get(id) {
                Some(load) if load.status == InitialDataLoadStatus::Completed => {}
                // there might be existing loads that is not written to the DB
                _ => {
                    match check_topic_fully_populated(
                        &load.table.name,
                        &project.clickhouse_config,
                        client,
                        &load.topic,
                        &project.redpanda_config,
                    )
                    .await?
                    {
                        None => {
                            // None means completed
                            // the load variable is the target state, which is to completed
                            existing_data_loads.insert(id.clone(), load.clone());
                        }
                        Some(progress) => {
                            existing_data_loads.insert(
                                id.clone(),
                                InitialDataLoad {
                                    status: InitialDataLoadStatus::InProgress(progress),
                                    ..load.clone()
                                },
                            );
                        }
                    };
                }
            };
        }

        current
    };

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

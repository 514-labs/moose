use super::{RoutineFailure, RoutineSuccess};
use crate::utilities::constants::{
    CLICKHOUSE_CONTAINER_NAME, CONSOLE_CONTAINER_NAME, REDPANDA_CONTAINER_NAME,
};
use crate::{cli::display::Message, utilities::docker};

fn validate_container_run(container_name: &str) -> Result<RoutineSuccess, RoutineFailure> {
    let containers = docker::list_containers().map_err(|err| {
        RoutineFailure::new(
            Message::new("Failed".to_string(), "to get the containers".to_string()),
            err,
        )
    })?;

    // check that the clickhouse container exists
    containers
        .iter()
        .find(|container| container.names.contains(container_name))
        .ok_or_else(|| {
            RoutineFailure::error(Message::new(
                "Failed".to_string(),
                format!("to find {} docker container", container_name),
            ))
        })?;
    Ok(RoutineSuccess::success(Message::new(
        "Successfully".to_string(),
        format!("validated {} docker container", container_name),
    )))
}

pub fn validate_clickhouse_run() -> Result<RoutineSuccess, RoutineFailure> {
    validate_container_run(CLICKHOUSE_CONTAINER_NAME)
}

pub fn validate_redpanda_run() -> Result<RoutineSuccess, RoutineFailure> {
    validate_container_run(REDPANDA_CONTAINER_NAME)
}

pub fn validate_console_run() -> Result<RoutineSuccess, RoutineFailure> {
    validate_container_run(CONSOLE_CONTAINER_NAME)
}

pub fn validate_redpanda_cluster(project_name: String) -> Result<RoutineSuccess, RoutineFailure> {
    docker::run_rpk_cluster_info(&project_name).map_err(|err| {
        RoutineFailure::new(
            Message::new(
                "Failed".to_string(),
                format!("to validate red panda cluster, {}", err),
            ),
            err,
        )
    })?;

    Ok(RoutineSuccess::success(Message::new(
        "Successfully".to_string(),
        "validated red panda cluster".to_string(),
    )))
}

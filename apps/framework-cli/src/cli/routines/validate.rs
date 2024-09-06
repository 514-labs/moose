use super::{RoutineFailure, RoutineSuccess};
use crate::project::Project;
use crate::utilities::constants::{CLICKHOUSE_CONTAINER_NAME, REDPANDA_CONTAINER_NAME};
use crate::utilities::docker::ContainerRow;
use crate::{cli::display::Message, utilities::docker};
use std::thread::sleep;
use std::time::Duration;

fn find_container(project: &Project, container_name: &str) -> Result<ContainerRow, RoutineFailure> {
    let containers = docker::list_containers(project).map_err(|err| {
        RoutineFailure::new(
            Message::new("Failed".to_string(), "to get the containers".to_string()),
            err,
        )
    })?;

    containers
        .into_iter()
        .find(|container| container.names.contains(container_name))
        .ok_or_else(|| {
            RoutineFailure::error(Message::new(
                "Failed".to_string(),
                format!("to find {} docker container", container_name),
            ))
        })
}

fn validate_container_run(
    project: &Project,
    container_name: &str,
    health: Option<&str>,
) -> Result<RoutineSuccess, RoutineFailure> {
    let mut container = find_container(project, container_name)?;

    if let Some(expected) = health {
        for _ in 0..20 {
            if container.health == expected {
                break;
            }

            container = find_container(project, container_name)?;
            sleep(Duration::from_secs(1));
        }
    }

    Ok(RoutineSuccess::success(Message::new(
        "Validated".to_string(),
        format!("{} docker container", container_name),
    )))
}

pub fn validate_clickhouse_run(project: &Project) -> Result<RoutineSuccess, RoutineFailure> {
    validate_container_run(project, CLICKHOUSE_CONTAINER_NAME, Some("healthy"))
}

pub fn validate_redpanda_run(project: &Project) -> Result<RoutineSuccess, RoutineFailure> {
    validate_container_run(project, REDPANDA_CONTAINER_NAME, None)
}

pub fn validate_redpanda_cluster(project_name: String) -> Result<RoutineSuccess, RoutineFailure> {
    docker::run_rpk_cluster_info(&project_name, 10).map_err(|err| {
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

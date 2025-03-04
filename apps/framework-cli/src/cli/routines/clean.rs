use crate::utilities::docker::DockerClient;
use crate::{cli::display::Message, project::Project};
use log::info;

use super::util::ensure_docker_running;
use super::{RoutineFailure, RoutineSuccess};

pub fn clean_project(
    project: &Project,
    docker_client: &DockerClient,
) -> Result<RoutineSuccess, RoutineFailure> {
    ensure_docker_running(docker_client)?;

    if std::env::var("MOOSE_SKIP_CONTAINER_SHUTDOWN").is_err() {
        docker_client.stop_containers(project).map_err(|err| {
            RoutineFailure::new(
                Message::new("Failed".to_string(), "to stop containers".to_string()),
                err,
            )
        })?;
    } else {
        info!(
            "Skipping container shutdown due to MOOSE_SKIP_CONTAINER_SHUTDOWN environment variable"
        );
    }

    Ok(RoutineSuccess::success(Message::new(
        "Cleaned".to_string(),
        "project".to_string(),
    )))
}

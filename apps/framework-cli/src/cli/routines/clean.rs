use crate::utilities::docker::DockerClient;
use crate::{cli::display::Message, project::Project};
use log::error;

use super::util::ensure_docker_running;
use super::{RoutineFailure, RoutineSuccess};

pub fn clean_project(
    project: &Project,
    docker_client: &DockerClient,
) -> Result<RoutineSuccess, RoutineFailure> {
    ensure_docker_running(docker_client)?;
    docker_client.stop_containers(project).map_err(|err| {
        RoutineFailure::new(
            Message::new("Failed".to_string(), "to stop containers".to_string()),
            err,
        )
    })?;

    project.delete_old_versions().map_err(|err| {
        error!("Failed to remove .moose directory: {:?}", err);
        RoutineFailure::new(
            Message::new(
                "Failed".to_string(),
                "to remove .moose directory".to_string(),
            ),
            err,
        )
    })?;

    Ok(RoutineSuccess::success(Message::new(
        "Cleaned".to_string(),
        "project".to_string(),
    )))
}

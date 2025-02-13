use crate::cli::display::Message;
use crate::cli::routines::RoutineFailure;
use crate::utilities::docker::DockerClient;

pub fn ensure_docker_running(docker_client: &DockerClient) -> Result<(), RoutineFailure> {
    let errors = docker_client.check_status().map_err(|err| {
        RoutineFailure::new(
            Message::new("Failed".to_string(), "to run `docker info`".to_string()),
            err,
        )
    })?;

    if errors.is_empty() {
        Ok(())
    } else if errors
        .iter()
        .any(|s| s.ends_with("Is the docker daemon running?"))
    {
        Err(RoutineFailure::error(Message::new(
            "Failed".to_string(),
            "to run docker commands. Is docker running?".to_string(),
        )))
    } else {
        Err(RoutineFailure::error(Message::new(
            "Failed".to_string(),
            errors.join("\n"),
        )))
    }
}

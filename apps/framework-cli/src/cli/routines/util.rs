use log::info;

use crate::cli::display::Message;
use crate::cli::routines::RoutineFailure;
use crate::utilities::docker;

pub fn ensure_docker_running() -> Result<(), RoutineFailure> {
    let errors = docker::check_status().map_err(|err| {
        info!("Failed to run docker info");
        RoutineFailure::new(
            Message::new("Failed".to_string(), "to run `docker info`".to_string()),
            err,
        )
    })?;

    if errors.is_empty() {
        info!("Docker is running");
        Ok(())
    } else if errors
        .iter()
        .any(|s| s.ends_with("Is the docker daemon running?"))
    {
        info!("Failed to run docker commands. Is docker running?");
        Err(RoutineFailure::error(Message::new(
            "Failed".to_string(),
            "to run docker commands. Is docker running?".to_string(),
        )))
    } else {
        info!("Failed {}", errors.join("\n"));
        Err(RoutineFailure::error(Message::new(
            "Failed".to_string(),
            errors.join("\n"),
        )))
    }
}

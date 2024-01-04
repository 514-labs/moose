use crate::cli::display::Message;
use crate::cli::routines::RoutineFailure;
use crate::utilities::docker;

pub fn ensure_docker_running() -> Result<(), RoutineFailure> {
    let errors = docker::check_status().map_err(|err| {
        RoutineFailure::new(
            Message::new("Failed".to_string(), "to run `docker info`".to_string()),
            Some(err),
        )
    })?;

    if errors.is_empty() {
        Ok(())
    } else if errors
        .iter()
        .any(|s| s.ends_with("Is the docker daemon running?"))
    {
        Err(RoutineFailure::new(
            Message::new(
                "Failed".to_string(),
                "to run docker commands. Is docker running?".to_string(),
            ),
            None,
        ))
    } else {
        Err(RoutineFailure::new(
            Message::new("Failed".to_string(), errors.join("\n")),
            None,
        ))
    }
}

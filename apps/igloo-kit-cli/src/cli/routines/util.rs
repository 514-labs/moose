use crate::cli::display::Message;
use crate::cli::routines::RoutineFailure;
use crate::utilities::docker;

pub fn ensure_docker_running() -> Result<(), RoutineFailure> {
    let errors = docker::check_status().map_err(|err| {
        RoutineFailure::new(
            Message::new("Failed".to_string(), "to run `docker info`".to_string()),
            err,
        )
    })?;

    if errors.is_empty() {
        Ok(())
    } else {
        Err(RoutineFailure::new(
            Message::new("Failed".to_string(), errors.join("\n")),
            std::io::Error::new(std::io::ErrorKind::Other, "docker server error".to_string()),
        ))
    }
}

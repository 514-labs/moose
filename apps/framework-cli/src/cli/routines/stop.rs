use crate::cli::display::{with_spinner, Message};
use crate::cli::routines::util::ensure_docker_running;
use crate::utilities::docker;

use super::{Routine, RoutineFailure, RoutineSuccess, RunMode};

pub struct StopLocalInfrastructure {
    run_mode: RunMode,
}
impl StopLocalInfrastructure {
    pub fn new(run_mode: RunMode) -> Self {
        Self { run_mode }
    }
}
impl Routine for StopLocalInfrastructure {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        ensure_docker_running()?;
        with_spinner("Stopping local infrastructure", || {
            docker::stop_containers().map_err(|err| {
                RoutineFailure::new(
                    Message::new(
                        "Failed".to_string(),
                        "to stop local infrastructure".to_string(),
                    ),
                    err,
                )
            })
        })?;

        Ok(RoutineSuccess::success(Message::new(
            "Successfully".to_string(),
            "stopped local infrastructure".to_string(),
        )))
    }
}

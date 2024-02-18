use crate::cli::display::{with_spinner, Message};
use crate::cli::routines::util::ensure_docker_running;
use crate::project::Project;
use crate::utilities::docker;

use super::{Routine, RoutineFailure, RoutineSuccess};

pub struct StopLocalInfrastructure {
    project: Project,
}
impl StopLocalInfrastructure {
    pub fn new(project: Project) -> Self {
        Self { project }
    }
}
impl Routine for StopLocalInfrastructure {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        ensure_docker_running()?;
        with_spinner("Stopping local infrastructure", || {
            docker::stop_containers(&self.project).map_err(|err| {
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

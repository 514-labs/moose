use log::error;
use std::sync::Arc;

use crate::cli::routines::util::ensure_docker_running;
use crate::utilities::docker;
use crate::{cli::display::Message, project::Project};

use super::{Routine, RoutineFailure, RoutineSuccess};

pub struct CleanProject {
    project: Arc<Project>,
}
impl CleanProject {
    pub fn new(project: Arc<Project>) -> Self {
        Self { project }
    }
}

impl Routine for CleanProject {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        ensure_docker_running()?;
        docker::stop_containers(&self.project).map_err(|err| {
            RoutineFailure::new(
                Message::new("Failed".to_string(), "to stop containers".to_string()),
                err,
            )
        })?;

        self.project.delete_old_versions().map_err(|err| {
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
}

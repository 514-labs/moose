use log::error;
use std::sync::Arc;

use crate::cli::routines::util::ensure_docker_running;
use crate::{cli::display::Message, project::Project};

use super::{stop::StopLocalInfrastructure, Routine, RoutineFailure, RoutineSuccess, RunMode};

pub struct CleanProject {
    project: Arc<Project>,
    run_mode: RunMode,
}
impl CleanProject {
    pub fn new(project: Arc<Project>, run_mode: RunMode) -> Self {
        Self { project, run_mode }
    }
}

impl Routine for CleanProject {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        let run_mode = self.run_mode;

        ensure_docker_running()?;
        StopLocalInfrastructure::new(self.project.clone()).run(run_mode)?;

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

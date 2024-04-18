use std::sync::Arc;

use crate::utilities::constants::{SAMPLE_FLOWS_DEST, SAMPLE_FLOWS_SOURCE};
use crate::{cli::display::Message, project::Project};

use super::flow::create_flow_directory;
use super::{Routine, RoutineFailure, RoutineSuccess, RunMode};

pub struct InitializeProject {
    run_mode: RunMode,
    project: Arc<Project>,
}
impl InitializeProject {
    pub fn new(run_mode: RunMode, project: Arc<Project>) -> Self {
        Self { run_mode, project }
    }
}
impl Routine for InitializeProject {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        let run_mode: RunMode = self.run_mode;

        self.project.setup_app_dir().map_err(|err| {
            RoutineFailure::new(
                Message::new(
                    "Failed".to_string(),
                    "to create app directory. Check permissions or contact us`".to_string(),
                ),
                err,
            )
        })?;

        create_flow_directory(
            &self.project,
            SAMPLE_FLOWS_SOURCE.to_string(),
            SAMPLE_FLOWS_DEST.to_string(),
        )?;

        CreateBaseAppFiles::new(self.project.clone()).run(run_mode)?;

        Ok(RoutineSuccess::success(Message::new(
            "Created".to_string(),
            "Moose app with file scaffolding".to_string(),
        )))
    }
}

pub struct CreateBaseAppFiles {
    project: Arc<Project>,
}

impl CreateBaseAppFiles {
    pub fn new(project: Arc<Project>) -> Self {
        Self { project }
    }
}

impl Routine for CreateBaseAppFiles {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        self.project.create_base_app_files().map_err(|err| {
            RoutineFailure::new(Message::new("Failed".to_string(), "".to_string()), err)
        })?;

        Ok(RoutineSuccess::success(Message::new(
            "Created".to_string(),
            "base app files".to_string(),
        )))
    }
}

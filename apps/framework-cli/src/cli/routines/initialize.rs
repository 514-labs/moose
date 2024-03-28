use std::fs;
use std::sync::Arc;

use crate::utilities::constants::SAMPLE_FLOWS_DIR;
use crate::{cli::display::Message, project::Project};

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

        CreateSampleFlowDirectory::new(self.project.clone()).run(run_mode)?;
        CreateBaseAppFiles::new(self.project.clone()).run(run_mode)?;

        Ok(RoutineSuccess::success(Message::new(
            "Created".to_string(),
            "Moose directory with Red Panda and Clickhouse mount volumes".to_string(),
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

pub struct CreateSampleFlowDirectory {
    project: Arc<Project>,
}

impl CreateSampleFlowDirectory {
    pub fn new(project: Arc<Project>) -> Self {
        Self { project }
    }
}

impl Routine for CreateSampleFlowDirectory {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        let flows_dir = self.project.flows_dir();
        let sample_flows_dirs = flows_dir.join(SAMPLE_FLOWS_DIR);

        match fs::create_dir_all(sample_flows_dirs.clone()) {
            Ok(_) => Ok(RoutineSuccess::success(Message::new(
                "Created".to_string(),
                "sample flow directory".to_string(),
            ))),
            Err(err) => Err(RoutineFailure::new(
                Message::new(
                    "Failed".to_string(),
                    format!(
                        "to create sample flow directory in {}",
                        sample_flows_dirs.display()
                    ),
                ),
                err,
            )),
        }
    }
}

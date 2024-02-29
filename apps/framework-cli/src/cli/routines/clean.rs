use std::sync::Arc;
use std::{fs, path::PathBuf};

use log::info;

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
        // TODO this pattern of mapping errors is repeated - could be refactored into a helper
        let internal_dir = self.project.internal_dir().map_err(|err| {
            RoutineFailure::new(
                Message::new(
                    "Failed".to_string(),
                    "to get internal directory for project".to_string(),
                ),
                err,
            )
        })?;

        info!("Checking docker");
        ensure_docker_running()?;
        info!("Stopping local infrastructure");
        StopLocalInfrastructure::new(self.project.clone()).run(run_mode)?;
        info!("Deleting redpanda volume)");
        DeleteRedpandaMountVolume::new(internal_dir.clone()).run(run_mode)?;
        info!("Deleting clickhouse volume");
        DeleteClickhouseMountVolume::new(internal_dir.clone()).run(run_mode)?;
        info!("Deleting model volume");
        DeleteModelVolume::new(internal_dir.clone()).run(run_mode)?;

        Ok(RoutineSuccess::success(Message::new(
            "Cleaned".to_string(),
            "project".to_string(),
        )))
    }
}

struct DeleteRedpandaMountVolume {
    internal_dir: PathBuf,
}
impl DeleteRedpandaMountVolume {
    fn new(internal_dir: PathBuf) -> Self {
        Self { internal_dir }
    }
}

impl Routine for DeleteRedpandaMountVolume {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        let mount_dir = self.internal_dir.join(".panda_house");
        fs::remove_dir_all(&mount_dir).map_err(|err| {
            RoutineFailure::new(
                Message::new(
                    "Failed".to_string(),
                    format!(
                        "to remove Red Panda mount volume at {}",
                        mount_dir.display()
                    ),
                ),
                err,
            )
        })?;

        Ok(RoutineSuccess::success(Message::new(
            "Successfully".to_string(),
            "removed Red Panda mount volume".to_string(),
        )))
    }
}

struct DeleteClickhouseMountVolume {
    internal_dir: PathBuf,
}
impl DeleteClickhouseMountVolume {
    fn new(internal_dir: PathBuf) -> Self {
        Self { internal_dir }
    }
}
impl Routine for DeleteClickhouseMountVolume {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        let mount_dir = self.internal_dir.join(".clickhouse");
        fs::remove_dir_all(&mount_dir).map_err(|err| {
            RoutineFailure::new(
                Message::new(
                    "Failed".to_string(),
                    format!(
                        "to remove Clickhouse mount volume at {}",
                        mount_dir.display()
                    ),
                ),
                err,
            )
        })?;

        Ok(RoutineSuccess::success(Message::new(
            "Successfully".to_string(),
            "removed Clickhouse mount volume".to_string(),
        )))
    }
}

struct DeleteModelVolume {
    internal_dir: PathBuf,
}

impl DeleteModelVolume {
    fn new(internal_dir: PathBuf) -> Self {
        Self { internal_dir }
    }
}

impl Routine for DeleteModelVolume {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        let mount_dir = self.internal_dir.join("models");
        fs::remove_dir_all(&mount_dir).map_err(|err| {
            RoutineFailure::new(
                Message::new(
                    "Failed".to_string(),
                    format!("to remove Model mount volume at {}", mount_dir.display()),
                ),
                err,
            )
        })?;

        Ok(RoutineSuccess::success(Message::new(
            "Successfully".to_string(),
            "removed Model mount volume".to_string(),
        )))
    }
}

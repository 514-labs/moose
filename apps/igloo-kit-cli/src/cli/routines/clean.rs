use std::{fs, path::PathBuf};

use crate::cli::routines::util::ensure_docker_running;
use crate::{cli::display::Message, constants::PANDA_NETWORK, project::Project, utilities::docker};

use super::{stop::StopLocalInfrastructure, Routine, RoutineFailure, RoutineSuccess, RunMode};

pub struct CleanProject {
    project: Project,
    run_mode: RunMode,
}
impl CleanProject {
    pub fn new(project: Project, run_mode: RunMode) -> Self {
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
                Some(err),
            )
        })?;

        ensure_docker_running()?;
        StopLocalInfrastructure::new(run_mode).run(run_mode)?;
        RemoveDockerNetwork::new(PANDA_NETWORK).run(run_mode)?;
        DeleteRedpandaMountVolume::new(internal_dir.clone()).run(run_mode)?;
        DeleteClickhouseMountVolume::new(internal_dir.clone()).run(run_mode)?;
        DeleteModelVolume::new(internal_dir.clone()).run(run_mode)?;

        Ok(RoutineSuccess::success(Message::new(
            "Cleaned".to_string(),
            "project".to_string(),
        )))
    }
}

struct RemoveDockerNetwork {
    network_name: String,
}
impl RemoveDockerNetwork {
    fn new(network_name: &str) -> Self {
        Self {
            network_name: network_name.to_string(),
        }
    }
}

impl Routine for RemoveDockerNetwork {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        docker::remove_network(&self.network_name).map_err(|err| {
            RoutineFailure::new(
                Message::new("Failed".to_string(), "to remove docker network".to_string()),
                Some(err),
            )
        })?;

        Ok(RoutineSuccess::success(Message::new(
            "Successfully".to_string(),
            "removed docker network".to_string(),
        )))
    }
}

struct DeleteRedpandaMountVolume {
    igloo_dir: PathBuf,
}
impl DeleteRedpandaMountVolume {
    fn new(igloo_dir: PathBuf) -> Self {
        Self { igloo_dir }
    }
}

impl Routine for DeleteRedpandaMountVolume {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        let mount_dir = self.igloo_dir.join(".panda_house");
        fs::remove_dir_all(&mount_dir).map_err(|err| {
            RoutineFailure::new(
                Message::new(
                    "Failed".to_string(),
                    format!(
                        "to remove Red Panda mount volume at {}",
                        mount_dir.display()
                    ),
                ),
                Some(err),
            )
        })?;

        Ok(RoutineSuccess::success(Message::new(
            "Successfully".to_string(),
            "removed Red Panda mount volume".to_string(),
        )))
    }
}

struct DeleteClickhouseMountVolume {
    igloo_dir: PathBuf,
}
impl DeleteClickhouseMountVolume {
    fn new(igloo_dir: PathBuf) -> Self {
        Self { igloo_dir }
    }
}
impl Routine for DeleteClickhouseMountVolume {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        let mount_dir = self.igloo_dir.join(".clickhouse");
        fs::remove_dir_all(&mount_dir).map_err(|err| {
            RoutineFailure::new(
                Message::new(
                    "Failed".to_string(),
                    format!(
                        "to remove Clickhouse mount volume at {}",
                        mount_dir.display()
                    ),
                ),
                Some(err),
            )
        })?;

        Ok(RoutineSuccess::success(Message::new(
            "Successfully".to_string(),
            "removed Clickhouse mount volume".to_string(),
        )))
    }
}

struct DeleteModelVolume {
    igloo_dir: PathBuf,
}

impl DeleteModelVolume {
    fn new(igloo_dir: PathBuf) -> Self {
        Self { igloo_dir }
    }
}

impl Routine for DeleteModelVolume {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        let mount_dir = self.igloo_dir.join("models");
        fs::remove_dir_all(&mount_dir).map_err(|err| {
            RoutineFailure::new(
                Message::new(
                    "Failed".to_string(),
                    format!("to remove Model mount volume at {}", mount_dir.display()),
                ),
                Some(err),
            )
        })?;

        Ok(RoutineSuccess::success(Message::new(
            "Successfully".to_string(),
            "removed Model mount volume".to_string(),
        )))
    }
}

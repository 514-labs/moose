use std::{path::PathBuf, fs};

use crate::{cli::display::Message, infrastructure::PANDA_NETWORK, utilities::docker};

use super::{Routine, RoutineSuccess, RoutineFailure, stop::StopLocalInfrastructure, RunMode};


pub struct CleanProject {
    igloo_dir: PathBuf,
    run_mode: RunMode,
}
impl CleanProject {
    pub fn new(igloo_dir: PathBuf, run_mode: RunMode) -> Self {
        Self { igloo_dir, run_mode }
    }
}

impl Routine for CleanProject {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        let run_mode = self.run_mode.clone();
        StopLocalInfrastructure::new(run_mode.clone()).run(run_mode.clone())?;
        RemoveDockerNetwork::new(PANDA_NETWORK).run(run_mode.clone())?;
        DeleteRedpandaMountVolume::new(self.igloo_dir.clone()).run(run_mode.clone())?;
        DeleteClickhouseMountVolume::new(self.igloo_dir.clone()).run(run_mode.clone())?;

        Ok(RoutineSuccess::success(Message::new("Cleaned".to_string(), "project".to_string())))
    }
}


struct RemoveDockerNetwork {
    network_name: String,
}
impl RemoveDockerNetwork {
    fn new(network_name: &str) -> Self {
        Self { network_name: network_name.to_string() }
    }
}

impl Routine for RemoveDockerNetwork {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        docker::remove_network(&self.network_name).map_err(|err| {
            RoutineFailure::new(Message::new("Failed".to_string(), "to remove docker network".to_string()), err)
        })?;

        Ok(RoutineSuccess::success(Message::new("Successfully".to_string(), "removed docker network".to_string())))
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
            RoutineFailure::new(Message::new("Failed".to_string(), format!("to remove Red Panda mount volume at {}", mount_dir.display())), err)
        })?;

        Ok(RoutineSuccess::success(Message::new("Successfully".to_string(), "removed Red Panda mount volume".to_string())))    
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
            RoutineFailure::new(Message::new("Failed".to_string(), format!("to remove Clickhouse mount volume at {}", mount_dir.display())), err)
        })?;

        Ok(RoutineSuccess::success(Message::new("Successfully".to_string(), "removed Clickhouse mount volume".to_string())))    
    }
}
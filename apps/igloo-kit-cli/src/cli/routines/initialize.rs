use std::{io::{Error, ErrorKind}, path::PathBuf, fs};

use crate::{cli::{CommandTerminal, display::{show_message, MessageType, Message}}, framework::{self, directories::{create_igloo_directory, get_igloo_directory, create_app_directories}}, infrastructure::PANDA_NETWORK, utilities::docker};

use super::{Routine, RoutineFailure, RoutineSuccess};


struct InitializeProject;
impl Routine for InitializeProject {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        CreateIglooTempDirectoryTree.run_silent()?;
        let igloo_dir = get_igloo_directory().map_err(|err| {
            RoutineFailure::new(Message::new("Failed", "to create .igloo directory. Check permissions or contact us`"), err)
        })?;
        create_app_directories().map_err(|err| {
            RoutineFailure::new(Message::new("Failed", "to create app directory. Check permissions or contact us`"), err)
        })?;
        CreateDockerNetwork::new(PANDA_NETWORK).run_silent()?;
        CreateVolumes::new(igloo_dir).run_silent()?;

        Ok(RoutineSuccess::success(Message::new("Created", "Igloo directory with Red Panda and Clickhouse mount volumes")))
    }
}


struct CreateVolumes {
    igloo_dir: PathBuf,
}

impl CreateVolumes {
    fn new(igloo_dir: PathBuf) -> Self {
        Self { igloo_dir }
    }
}

impl Routine for CreateVolumes {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        CreateRedPandaMountVolume::new(self.igloo_dir).run_silent()?;
        CreateClickhouseMountVolume::new(self.igloo_dir).run_silent()?;

        Ok(RoutineSuccess::success(Message::new("Created", "Red Panda and Clickhouse mount volumes")))
    }
}


struct ValidateMountVolumes<'a> {
    igloo_dir: &'a PathBuf,
}
impl Routine for ValidateMountVolumes<'_> {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        let panda_house = self.igloo_dir.join(".panda_house").exists();
        let clickhouse = self.igloo_dir.join(".clickhouse").exists();

        if panda_house && clickhouse {
            Ok(RoutineSuccess::success(Message::new("Valid", "Red Panda and Clickhouse mount volumes exist.")))
        } else {
            let message = format!("redpanda: {panda_house}, clickhouse: {clickhouse}");
            Err(RoutineFailure::new(Message::new("Mount volume status", &message), Error::new(ErrorKind::NotFound, message)))
        }
    }
}


struct CreateIglooTempDirectoryTree;
impl Routine for CreateIglooTempDirectoryTree {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        let igloo_dir = create_igloo_directory().map_err(|err| {
            RoutineFailure::new(Message::new("Failed", "to create .igloo directory. Check permissions or contact us`"), err)
        })?;

        CreateTempDataVolumes.run_silent()?;

        // Add validate data volume routine here

        Ok(RoutineSuccess::success(Message::new("Created", "Igloo directory with Red Panda and Clickhouse mount volumes")))
    }
}


struct CreateTempDataVolumes;
impl Routine for CreateTempDataVolumes {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        if let Ok(igloo_dir) = get_igloo_directory() {
            CreateVolumes::new(igloo_dir).run_silent()?;
            Ok(RoutineSuccess::success(Message::new("Created", "Red Panda and Clickhouse mount volumes")))
        } else {
            let igloo_dir = create_igloo_directory().map_err(|err| {
                RoutineFailure::new(Message::new("Failed", "to create .igloo directory. Check permissions or contact us`"), err)
            })?;
            CreateVolumes::new(igloo_dir).run_silent()?;
            Ok(RoutineSuccess::success(Message::new("Created", "Red Panda and Clickhouse mount volumes")))
        }
    }}


struct CreateRedPandaMountVolume {
    igloo_dir: PathBuf,
}

impl CreateRedPandaMountVolume {
    fn new(igloo_dir: PathBuf) -> Self {
        Self { igloo_dir }
    }
}

impl Routine for CreateRedPandaMountVolume {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        let mount_dir = self.igloo_dir.join(".panda_house");
        match fs::create_dir_all(mount_dir.clone()) {
            Ok(_) => Ok(RoutineSuccess::success(Message::new("Created", "Red Panda mount volume"))),
            Err(err) => {
                Err(RoutineFailure::new(Message::new("Failed", &format!("to create Red Panda mount volume in {}", mount_dir.display())), err))
            }
        }
    }
}

struct CreateClickhouseMountVolume {
    igloo_dir: PathBuf,
}

impl CreateClickhouseMountVolume {
    fn new(igloo_dir: PathBuf) -> Self {
        Self { igloo_dir }
    }
}

impl Routine for CreateClickhouseMountVolume {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        let mount_dir = self.igloo_dir.join(".clickhouse");
        
        // fs::create_dir_all(&server_config_path)?;
        fs::create_dir_all(&mount_dir).map_err(|err| {
            RoutineFailure::new(Message::new("Failed", &format!("to create Clickhouse mount volume {}", mount_dir.display())), err)
        })?;
        fs::create_dir_all(&mount_dir.join("data")).map_err(|err| {
            RoutineFailure::new(Message::new("Failed", &format!("to create Clickhouse data mount volume in {}", mount_dir.display())), err)
        })?;
        fs::create_dir_all(&mount_dir.join("logs")).map_err(|err| {
            RoutineFailure::new(Message::new("Failed", &format!("to create Clickhouse logs mount volume in {}", mount_dir.display())), err)
        })?;
        // database::create_server_config_file(&server_config_path)?;
        // database::create_user_config_file(&user_config_path)?;
        // database::create_init_script(&scripts_path)?;

        Ok(RoutineSuccess::success(Message::new("Created", "Clickhouse mount volumes")))
    }
        
}

struct CreateDockerNetwork<'a> {
    network_name: &'a str,
}

impl CreateDockerNetwork<'_> {
    fn new(network_name: &str) -> Self {
        Self { network_name }
    }
}

impl Routine for CreateDockerNetwork<'_> {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        let output = docker::create_network(&self.network_name);

        match output {
            Ok(_) => {
                Ok(RoutineSuccess::success(Message::new("Created", &format!("docker network {}", &self.network_name))))
            },
            Err(err) => {
                Err(RoutineFailure::new(Message::new("Failed", &format!("to create docker network {}", &self.network_name)), err))
            },
        }
    }
}

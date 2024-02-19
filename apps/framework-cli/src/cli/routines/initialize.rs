use std::{fs, path::PathBuf};

use crate::{
    cli::display::Message,
    framework::{languages::create_models_dir, typescript::create_typescript_models_dir},
    project::Project,
    utilities::docker,
};

use super::{Routine, RoutineFailure, RoutineSuccess, RunMode};

pub struct InitializeProject {
    run_mode: RunMode,
    project: Project,
}
impl InitializeProject {
    pub fn new(run_mode: RunMode, project: Project) -> Self {
        Self { run_mode, project }
    }
}
impl Routine for InitializeProject {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        let run_mode: RunMode = self.run_mode;

        CreateInternalTempDirectoryTree::new(run_mode, self.project.clone()).run(run_mode)?;

        self.project.setup_app_dir().map_err(|err| {
            RoutineFailure::new(
                Message::new(
                    "Failed".to_string(),
                    "to create app directory. Check permissions or contact us`".to_string(),
                ),
                err,
            )
        })?;

        CreateModelsVolume::new(self.project.clone()).run(run_mode)?;
        CreateDockerComposeFile::new(self.project.clone()).run(run_mode)?;

        Ok(RoutineSuccess::success(Message::new(
            "Created".to_string(),
            "Moose directory with Red Panda and Clickhouse mount volumes".to_string(),
        )))
    }
}

pub struct CreateVolumes {
    internal_dir: PathBuf,
    run_mode: RunMode,
}

impl CreateVolumes {
    fn new(internal_dir: PathBuf, run_mode: RunMode) -> Self {
        Self {
            internal_dir,
            run_mode,
        }
    }
}

impl Routine for CreateVolumes {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        let internal_dir = self.internal_dir.clone();
        let run_mode = self.run_mode;
        CreateRedPandaMountVolume::new(internal_dir.clone()).run(run_mode)?;
        CreateClickhouseMountVolume::new(internal_dir.clone()).run(run_mode)?;

        Ok(RoutineSuccess::success(Message::new(
            "Created".to_string(),
            "Red Panda and Clickhouse mount volumes".to_string(),
        )))
    }
}

pub struct ValidateMountVolumes {
    internal_dir: PathBuf,
}
impl ValidateMountVolumes {
    pub fn new(internal_dir: PathBuf) -> Self {
        Self { internal_dir }
    }
}
impl Routine for ValidateMountVolumes {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        let panda_house = self.internal_dir.join(".panda_house").exists();
        let clickhouse = self.internal_dir.join(".clickhouse").exists();

        if panda_house && clickhouse {
            Ok(RoutineSuccess::success(Message::new(
                "Valid".to_string(),
                "Red Panda and Clickhouse mount volumes exist.".to_string(),
            )))
        } else {
            let message = format!("redpanda: {panda_house}, clickhouse: {clickhouse}");
            Err(RoutineFailure::error(Message::new(
                "Mount volume status".to_string(),
                message.clone(),
            )))
        }
    }
}

pub struct CreateInternalTempDirectoryTree {
    run_mode: RunMode,
    project: Project,
}
impl CreateInternalTempDirectoryTree {
    pub fn new(run_mode: RunMode, project: Project) -> Self {
        Self { run_mode, project }
    }
}
impl Routine for CreateInternalTempDirectoryTree {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        let internal_dir = self.project.internal_dir().map_err(|err| {
            RoutineFailure::new(
                Message::new(
                    "Failed".to_string(),
                    "to create .moose directory. Check permissions or contact us`".to_string(),
                ),
                err,
            )
        })?;
        let run_mode = self.run_mode;

        CreateTempDataVolumes::new(run_mode, self.project.clone()).run(run_mode)?;
        ValidateMountVolumes::new(internal_dir).run(run_mode)?;

        Ok(RoutineSuccess::success(Message::new(
            "Created".to_string(),
            "Moose directory with Red Panda and Clickhouse mount volumes".to_string(),
        )))
    }
}

pub struct CreateTempDataVolumes {
    run_mode: RunMode,
    project: Project,
}

impl CreateTempDataVolumes {
    fn new(run_mode: RunMode, project: Project) -> Self {
        Self { run_mode, project }
    }
}
impl Routine for CreateTempDataVolumes {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        let internal_dir = self.project.internal_dir().map_err(|err| {
            RoutineFailure::new(
                Message::new(
                    "Failed".to_string(),
                    "to create .moose directory. Check permissions or contact us`".to_string(),
                ),
                err,
            )
        })?;

        let run_mode = self.run_mode;
        CreateVolumes::new(internal_dir, run_mode).run(run_mode)?;
        Ok(RoutineSuccess::success(Message::new(
            "Created".to_string(),
            "Red Panda and Clickhouse mount volumes".to_string(),
        )))
    }
}

pub struct CreateRedPandaMountVolume {
    internal_dir: PathBuf,
}

impl CreateRedPandaMountVolume {
    fn new(internal_dir: PathBuf) -> Self {
        Self { internal_dir }
    }
}

impl Routine for CreateRedPandaMountVolume {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        let mount_dir = self.internal_dir.join(".panda_house");
        match fs::create_dir_all(mount_dir.clone()) {
            Ok(_) => Ok(RoutineSuccess::success(Message::new(
                "Created".to_string(),
                "Red Panda mount volume".to_string(),
            ))),
            Err(err) => Err(RoutineFailure::new(
                Message::new(
                    "Failed".to_string(),
                    format!(
                        "to create Red Panda mount volume in {}",
                        mount_dir.display()
                    ),
                ),
                err,
            )),
        }
    }
}

pub struct CreateClickhouseMountVolume {
    internal_dir: PathBuf,
}

impl CreateClickhouseMountVolume {
    fn new(internal_dir: PathBuf) -> Self {
        Self { internal_dir }
    }
}

impl Routine for CreateClickhouseMountVolume {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        let mount_dir = self.internal_dir.join(".clickhouse");

        // fs::create_dir_all(&server_config_path)?;
        fs::create_dir_all(&mount_dir).map_err(|err| {
            RoutineFailure::new(
                Message::new(
                    "Failed".to_string(),
                    format!("to create Clickhouse mount volume {}", mount_dir.display()),
                ),
                err,
            )
        })?;
        fs::create_dir_all(mount_dir.join("data")).map_err(|err| {
            RoutineFailure::new(
                Message::new(
                    "Failed".to_string(),
                    format!(
                        "to create Clickhouse data mount volume in {}",
                        mount_dir.display()
                    ),
                ),
                err,
            )
        })?;
        fs::create_dir_all(mount_dir.join("logs")).map_err(|err| {
            RoutineFailure::new(
                Message::new(
                    "Failed".to_string(),
                    format!(
                        "to create Clickhouse logs mount volume in {}",
                        mount_dir.display()
                    ),
                ),
                err,
            )
        })?;
        // database::create_server_config_file(&server_config_path)?;
        // database::create_user_config_file(&user_config_path)?;
        // database::create_init_script(&scripts_path)?;

        Ok(RoutineSuccess::success(Message::new(
            "Created".to_string(),
            "Clickhouse mount volumes".to_string(),
        )))
    }
}

pub struct CreateModelsVolume {
    project: Project,
}

impl CreateModelsVolume {
    fn new(project: Project) -> Self {
        Self { project }
    }
}

impl Routine for CreateModelsVolume {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        create_models_dir(self.project.clone()).map_err(|err| {
            RoutineFailure::new(
                Message::new(
                    "Failed".to_string(),
                    format!("to create models volume in {}", err),
                ),
                err,
            )
        })?;

        create_typescript_models_dir(self.project.clone()).map_err(|err| {
            RoutineFailure::new(
                Message::new(
                    "Failed".to_string(),
                    format!("to create models volume in {}", err),
                ),
                err,
            )
        })?;

        Ok(RoutineSuccess::success(Message::new(
            "Created".to_string(),
            "Models volume".to_string(),
        )))
    }
}

pub struct CreateDockerComposeFile {
    project: Project,
}

impl CreateDockerComposeFile {
    pub fn new(project: Project) -> Self {
        Self { project }
    }
}

impl Routine for CreateDockerComposeFile {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        let output = docker::create_compose_file(&self.project);

        match output {
            Ok(_) => Ok(RoutineSuccess::success(Message::new(
                "Created".to_string(),
                "docker compose file".to_string(),
            ))),
            Err(err) => Err(RoutineFailure::new(
                Message::new(
                    "Failed".to_string(),
                    "to create docker compose file".to_string(),
                ),
                err,
            )),
        }
    }
}

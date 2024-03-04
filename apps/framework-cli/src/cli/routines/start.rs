use lazy_static::lazy_static;
use std::fs;

use crate::cli::display::with_spinner;
use crate::cli::routines::initialize::{CreateInternalTempDirectoryTree, CreateModelsVolume};
use crate::cli::routines::util::ensure_docker_running;
use crate::utilities::constants::CLI_PROJECT_INTERNAL_DIR;
use crate::utilities::docker;
use crate::utilities::git::dump_old_version_schema;
use crate::{cli::display::Message, project::Project};
use log::debug;
use std::sync::Arc;

use super::{
    initialize::ValidateMountVolumes,
    validate::{ValidateClickhouseRun, ValidateConsoleRun, ValidateRedPandaRun},
    Routine, RoutineFailure, RoutineSuccess, RunMode,
};

pub struct RunLocalInfrastructure {
    project: Arc<Project>,
}
impl RunLocalInfrastructure {
    pub fn new(project: Arc<Project>) -> Self {
        Self { project }
    }
}

lazy_static! {
    static ref FAILED_TO_CREATE_INTERNAL_DIR: Message = Message::new(
        "Failed".to_string(),
        format!(
            "to create {} directory. Check permissions or contact us`",
            CLI_PROJECT_INTERNAL_DIR
        ),
    );
}

impl Routine for RunLocalInfrastructure {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        ensure_docker_running()?;
        let internal_dir = self
            .project
            .internal_dir()
            .map_err(|err| RoutineFailure::new(FAILED_TO_CREATE_INTERNAL_DIR.clone(), err))?;
        // Model this after the `spin_up` function in `apps/framework-cli/src/cli/routines/start.rs` but use routines instead
        CreateInternalTempDirectoryTree::new(RunMode::Explicit {}, self.project.clone())
            .run_explicit()?;
        CreateModelsVolume::new(self.project.clone()).run_explicit()?;
        ValidateMountVolumes::new(internal_dir).run_explicit()?;

        RunContainers::new(self.project.clone()).run_silent()?;
        ValidateRedPandaRun::new().run_explicit()?;
        ValidateClickhouseRun::new().run_explicit()?;
        ValidateConsoleRun::new().run_explicit()?;
        Ok(RoutineSuccess::success(Message::new(
            "Successfully".to_string(),
            "ran local infrastructure".to_string(),
        )))
    }
}

struct RunContainers {
    project: Arc<Project>,
}

impl RunContainers {
    fn new(project: Arc<Project>) -> Self {
        Self { project }
    }
}

impl Routine for RunContainers {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        let docker_compose_res = with_spinner("Starting local infrastructure", || {
            docker::start_containers(&self.project)
        });

        match docker_compose_res {
            Ok(_) => Ok(RoutineSuccess::success(Message::new(
                "Successfully".to_string(),
                "started containers".to_string(),
            ))),
            Err(err) => Err(RoutineFailure::new(
                Message::new("Failed".to_string(), "to start containers".to_string()),
                err,
            )),
        }
    }
}

pub struct CopyOldSchema {
    project: Arc<Project>,
}
impl CopyOldSchema {
    pub fn new(project: Arc<Project>) -> Self {
        Self { project }
    }
}
impl Routine for CopyOldSchema {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        for (version, commit_hash) in self.project.supported_old_versions.iter() {
            let dest = &self
                .project
                .internal_dir()
                .unwrap()
                .join("versions")
                .join(version);
            fs::create_dir_all(dest).map_err(|err| {
                RoutineFailure::new(
                    Message::new("Failed".to_string(), "to create directory".to_string()),
                    err,
                )
            })?;
            dump_old_version_schema(&self.project, commit_hash.clone(), &dest).map_err(
                |git_err| {
                    debug!("Failed to retrieve old schema: {}", git_err);
                    RoutineFailure::error(Message::new(
                        "Failed".to_string(),
                        "to retrieve old schema".to_string(),
                    ))
                },
            )?;
        }

        Ok(RoutineSuccess::success(Message::new(
            "Successfully".to_string(),
            "loaded old schema".to_string(),
        )))
    }
}

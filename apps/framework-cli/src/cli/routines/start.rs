use log::debug;

use super::{
    initialize::ValidateMountVolumes,
    validate::{
        ValidateClickhouseRun, ValidateConsoleRun, ValidatePandaHouseNetwork, ValidateRedPandaRun,
    },
    Routine, RoutineFailure, RoutineSuccess, RunMode,
};
use crate::cli::display::with_spinner;
use crate::cli::routines::initialize::{CreateDockerNetwork, CreateInternalTempDirectoryTree};
use crate::cli::routines::util::ensure_docker_running;
use crate::utilities::constants::{CLI_PROJECT_INTERNAL_DIR, PANDA_NETWORK};
use crate::{
    cli::display::Message,
    project::Project,
    utilities::docker::{self},
};
use lazy_static::lazy_static;

pub struct RunLocalInfrastructure {
    project: Project,
}
impl RunLocalInfrastructure {
    pub fn new(project: Project) -> Self {
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
        ValidateMountVolumes::new(internal_dir).run_explicit()?;
        CreateDockerNetwork::new(PANDA_NETWORK).run_explicit()?;
        ValidatePandaHouseNetwork::new().run_explicit()?;
        RunRedPandaContainer::new(self.project.clone()).run_explicit()?;
        ValidateRedPandaRun::new().run_explicit()?;
        RunClickhouseContainer::new(self.project.clone()).run_explicit()?;
        ValidateClickhouseRun::new().run_explicit()?;
        RunConsoleContainer::new(self.project.clone()).run_explicit()?;
        ValidateConsoleRun::new().run_explicit()?;
        Ok(RoutineSuccess::success(Message::new(
            "Successfully".to_string(),
            "ran local infrastructure".to_string(),
        )))
    }
}

pub struct RunRedPandaContainer {
    project: Project,
}
impl RunRedPandaContainer {
    pub fn new(project: Project) -> Self {
        Self { project }
    }
}

impl Routine for RunRedPandaContainer {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        let internal_dir = self
            .project
            .internal_dir()
            .map_err(|err| RoutineFailure::new(FAILED_TO_CREATE_INTERNAL_DIR.clone(), err))?;

        let output = with_spinner("Starting redpanda container", || {
            docker::safe_start_redpanda_container(internal_dir)
        })
        .map_err(|err| {
            RoutineFailure::new(
                Message::new(
                    "Failed".to_string(),
                    format!(
                        "to run redpanda container with following config: {:#?}",
                        self.project.redpanda_config
                    ),
                ),
                err,
            )
        })?;

        debug!("Redpanda container output: {:?}", output);

        Ok(RoutineSuccess::success(Message::new(
            "Successfully".to_string(),
            "ran redpanda container".to_string(),
        )))
    }
}

pub struct RunClickhouseContainer {
    project: Project,
}
impl RunClickhouseContainer {
    pub fn new(project: Project) -> Self {
        Self { project }
    }
}

impl Routine for RunClickhouseContainer {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        let internal_dir = self
            .project
            .internal_dir()
            .map_err(|err| RoutineFailure::new(FAILED_TO_CREATE_INTERNAL_DIR.clone(), err))?;

        let output = with_spinner("Starting clickhouse container", || {
            docker::safe_start_clickhouse_container(
                internal_dir,
                self.project.clickhouse_config.clone(),
            )
        })
        .map_err(|err| {
            RoutineFailure::new(
                Message::new(
                    "Failed".to_string(),
                    "to run clickhouse container".to_string(),
                ),
                err,
            )
        })?;

        debug!("Clickhouse container output: {:?}", output);

        Ok(RoutineSuccess::success(Message::new(
            "Successfully".to_string(),
            "ran clickhouse container".to_string(),
        )))
    }
}

pub struct RunConsoleContainer {
    project: Project,
}
impl RunConsoleContainer {
    pub fn new(project: Project) -> Self {
        Self { project }
    }
}

impl Routine for RunConsoleContainer {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        let output = with_spinner("Starting console container", || {
            docker::safe_start_console_container(
                &self.project.console_config,
                &self.project.clickhouse_config,
            )
        })
        .map_err(|err| {
            RoutineFailure::new(
                Message::new("Failed".to_string(), "to run console container".to_string()),
                err,
            )
        })?;

        debug!("Console container output: {:?}", output);

        Ok(RoutineSuccess::success(Message::new(
            "Successfully".to_string(),
            "ran console container".to_string(),
        )))
    }
}

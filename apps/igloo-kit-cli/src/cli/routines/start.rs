use log::debug;

use super::{
    initialize::ValidateMountVolumes,
    validate::{ValidateClickhouseRun, ValidatePandaHouseNetwork, ValidateRedPandaRun},
    Routine, RoutineFailure, RoutineSuccess, RunMode,
};
use crate::cli::routines::initialize::CreateIglooTempDirectoryTree;
use crate::cli::routines::util::ensure_docker_running;
use crate::{
    cli::display::Message,
    project::Project,
    utilities::docker::{self},
};

pub struct RunLocalInfrastructure {
    project: Project,
}
impl RunLocalInfrastructure {
    pub fn new(project: Project) -> Self {
        Self { project }
    }
}

impl Routine for RunLocalInfrastructure {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        ensure_docker_running()?;
        let igloo_dir = self.project.internal_dir().map_err(|err| {
            RoutineFailure::new(
                Message::new(
                    "Failed".to_string(),
                    "to create .igloo directory. Check permissions or contact us`".to_string(),
                ),
                err,
            )
        })?;
        // Model this after the `spin_up` function in `apps/igloo-kit-cli/src/cli/routines/start.rs` but use routines instead
        CreateIglooTempDirectoryTree::new(RunMode::Explicit {}, self.project.clone())
            .run_explicit()?;
        ValidateMountVolumes::new(igloo_dir).run_explicit()?;
        ValidatePandaHouseNetwork::new().run_explicit()?;
        RunRedPandaContainer::new(self.project.clone()).run_explicit()?;
        ValidateRedPandaRun::new().run_explicit()?;
        RunClickhouseContainer::new(self.project.clone()).run_explicit()?;
        ValidateClickhouseRun::new().run_explicit()?;
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
        let igloo_dir = self.project.internal_dir().map_err(|err| {
            RoutineFailure::new(
                Message::new(
                    "Failed".to_string(),
                    "to create .igloo directory. Check permissions or contact us`".to_string(),
                ),
                err,
            )
        })?;

        let output = docker::safe_start_redpanda_container(igloo_dir).map_err(|err| {
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
        let igloo_dir = self.project.internal_dir().map_err(|err| {
            RoutineFailure::new(
                Message::new(
                    "Failed".to_string(),
                    "to create .igloo directory. Check permissions or contact us`".to_string(),
                ),
                err,
            )
        })?;

        let output = docker::safe_start_clickhouse_container(
            igloo_dir,
            self.project.clickhouse_config.clone(),
        )
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

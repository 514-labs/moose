use super::{
    initialize::ValidateMountVolumes,
    validate::{ValidateClickhouseRun, ValidatePandaHouseNetwork, ValidateRedPandaRun},
    Routine, RoutineFailure, RoutineSuccess,
};
use crate::{
    cli::{display::Message, DebugStatus},
    infrastructure::{
        olap::clickhouse::config::ClickhouseConfig, stream::redpanda::RedpandaConfig,
    },
    project::Project,
    utilities::docker::{self},
};
use std::io::{self, Write};

pub struct RunLocalInfratructure {
    debug: DebugStatus,
    clickhouse_config: ClickhouseConfig,
    redpanda_config: RedpandaConfig,
    project: Project,
}
impl RunLocalInfratructure {
    pub fn new(
        debug: DebugStatus,
        clickhouse_config: ClickhouseConfig,
        redpanda_config: RedpandaConfig,
        project: Project,
    ) -> Self {
        Self {
            debug,
            clickhouse_config,
            redpanda_config,
            project,
        }
    }
}

impl Routine for RunLocalInfratructure {
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
        // Model this after the `spin_up` function in `apps/igloo-kit-cli/src/cli/routines/start.rs` but use routines instead
        ValidateMountVolumes::new(igloo_dir).run_silent()?;
        ValidatePandaHouseNetwork::new(self.debug).run_silent()?;
        RunRedPandaContainer::new(
            self.debug,
            self.redpanda_config.clone(),
            self.project.clone(),
        )
        .run_silent()?;
        ValidateRedPandaRun::new(self.debug).run_silent()?;
        RunClickhouseContainer::new(
            self.debug,
            self.clickhouse_config.clone(),
            self.project.clone(),
        )
        .run_silent()?;
        ValidateClickhouseRun::new(self.debug).run_silent()?;
        Ok(RoutineSuccess::success(Message::new(
            "Successfully".to_string(),
            "ran local infrastructure".to_string(),
        )))
    }
}

pub struct RunRedPandaContainer {
    debug: DebugStatus,
    redpanda_config: RedpandaConfig,
    project: Project,
}
impl RunRedPandaContainer {
    pub fn new(debug: DebugStatus, redpanda_config: RedpandaConfig, project: Project) -> Self {
        Self {
            debug,
            redpanda_config,
            project,
        }
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

        let output = docker::run_red_panda(igloo_dir).map_err(|err| {
            RoutineFailure::new(
                Message::new(
                    "Failed".to_string(),
                    format!(
                        "to run redpanda container with following config: {:#?}",
                        self.redpanda_config
                    ),
                ),
                err,
            )
        })?;

        if self.debug == DebugStatus::Debug {
            println!("Debugging red panda container run");
            println!("{}", &output.status);
            io::stdout().write_all(&output.stdout).unwrap();
        }

        Ok(RoutineSuccess::success(Message::new(
            "Successfully".to_string(),
            "ran redpanda container".to_string(),
        )))
    }
}

pub struct RunClickhouseContainer {
    debug: DebugStatus,
    clickhouse_config: ClickhouseConfig,
    project: Project,
}
impl RunClickhouseContainer {
    pub fn new(debug: DebugStatus, clickhouse_config: ClickhouseConfig, project: Project) -> Self {
        Self {
            debug,
            clickhouse_config,
            project,
        }
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

        let output =
            docker::run_clickhouse(igloo_dir, self.clickhouse_config.clone()).map_err(|err| {
                RoutineFailure::new(
                    Message::new(
                        "Failed".to_string(),
                        "to run clickhouse container".to_string(),
                    ),
                    err,
                )
            })?;

        if self.debug == DebugStatus::Debug {
            println!("Debugging clickhouse container run");
            io::stdout().write_all(&output.stdout).unwrap();
        }

        Ok(RoutineSuccess::success(Message::new(
            "Successfully".to_string(),
            "ran clickhouse container".to_string(),
        )))
    }
}

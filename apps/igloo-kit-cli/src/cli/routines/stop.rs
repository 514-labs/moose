use super::{Routine, RoutineFailure, RoutineSuccess, RunMode};
use crate::cli::routines::util::ensure_docker_running;
use crate::utilities::constants::{
    CLICKHOUSE_CONTAINER_NAME, CONSOLE_CONTAINER_NAME, REDPANDA_CONTAINER_NAME,
};
use crate::{cli::display::Message, utilities::docker};

pub struct StopLocalInfrastructure {
    run_mode: RunMode,
}
impl StopLocalInfrastructure {
    pub fn new(run_mode: RunMode) -> Self {
        Self { run_mode }
    }
}
impl Routine for StopLocalInfrastructure {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        ensure_docker_running()?;
        let run_mode = self.run_mode;

        StopRedPandaContainer::new().run(run_mode)?;
        StopClickhouseContainer::new().run(run_mode)?;
        StopConsoleContainer::new().run(run_mode)?;

        Ok(RoutineSuccess::success(Message::new(
            "Successfully".to_string(),
            "stopped local infrastructure".to_string(),
        )))
    }
}

pub struct StopRedPandaContainer;
impl StopRedPandaContainer {
    pub fn new() -> Self {
        Self
    }
}
impl Routine for StopRedPandaContainer {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        docker::stop_container(REDPANDA_CONTAINER_NAME).map_err(|err| {
            RoutineFailure::new(
                Message::new(
                    "Failed".to_string(),
                    "to stop redpanda container".to_string(),
                ),
                err,
            )
        })?;

        Ok(RoutineSuccess::success(Message::new(
            "Successfully".to_string(),
            "stopped redpanda container".to_string(),
        )))
    }
}

pub struct StopClickhouseContainer;
impl StopClickhouseContainer {
    pub fn new() -> Self {
        Self
    }
}
impl Routine for StopClickhouseContainer {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        docker::stop_container(CLICKHOUSE_CONTAINER_NAME).map_err(|err| {
            RoutineFailure::new(
                Message::new(
                    "Failed".to_string(),
                    "to stop clickhouse container".to_string(),
                ),
                err,
            )
        })?;

        Ok(RoutineSuccess::success(Message::new(
            "Successfully".to_string(),
            "stopped clickhouse container".to_string(),
        )))
    }
}

pub struct StopConsoleContainer;
impl StopConsoleContainer {
    pub fn new() -> Self {
        Self
    }
}
impl Routine for StopConsoleContainer {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        docker::stop_container(CONSOLE_CONTAINER_NAME).map_err(|err| {
            RoutineFailure::new(
                Message::new(
                    "Failed".to_string(),
                    "to stop console container".to_string(),
                ),
                err,
            )
        })?;

        Ok(RoutineSuccess::success(Message::new(
            "Successfully".to_string(),
            "stopped console container".to_string(),
        )))
    }
}

use super::{Routine, RoutineFailure, RoutineSuccess};
use crate::utilities::constants::{
    CLICKHOUSE_CONTAINER_NAME, CONSOLE_CONTAINER_NAME, REDPANDA_CONTAINER_NAME,
};
use crate::{cli::display::Message, utilities::constants::PANDA_NETWORK, utilities::docker};

pub struct ValidateClickhouseRun;
impl ValidateClickhouseRun {
    pub fn new() -> Self {
        Self
    }
}
impl Routine for ValidateClickhouseRun {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        let containers = docker::list_containers().map_err(|err| {
            RoutineFailure::new(
                Message::new("Failed".to_string(), "to get the containers".to_string()),
                err,
            )
        })?;

        // check that the clickhouse container exists
        containers
            .iter()
            .find(|container| container.names.contains(CLICKHOUSE_CONTAINER_NAME))
            .ok_or_else(|| {
                RoutineFailure::error(Message::new(
                    "Failed".to_string(),
                    "to find clickhouse docker container".to_string(),
                ))
            })?;
        Ok(RoutineSuccess::success(Message::new(
            "Successfully".to_string(),
            "validated clickhouse docker container".to_string(),
        )))
    }
}

pub struct ValidateRedPandaRun;
impl ValidateRedPandaRun {
    pub fn new() -> Self {
        Self
    }
}

impl Routine for ValidateRedPandaRun {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        let containers = docker::list_containers().map_err(|err| {
            RoutineFailure::new(
                Message::new("Failed".to_string(), "to get the containers".to_string()),
                err,
            )
        })?;

        // check that the clickhouse container exists
        containers
            .iter()
            .find(|container| container.names.contains(REDPANDA_CONTAINER_NAME))
            .ok_or_else(|| {
                RoutineFailure::error(Message::new(
                    "Failed".to_string(),
                    "to find redpanda docker container".to_string(),
                ))
            })?;
        Ok(RoutineSuccess::success(Message::new(
            "Successfully".to_string(),
            "validated redpanda docker container".to_string(),
        )))
    }
}

pub struct ValidatePandaHouseNetwork;
impl ValidatePandaHouseNetwork {
    pub fn new() -> Self {
        Self
    }
}
impl Routine for ValidatePandaHouseNetwork {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        let docker_networks = docker::network_list().map_err(|err| {
            RoutineFailure::new(
                Message::new(
                    "Failed".to_string(),
                    "to get list of docker networks".to_string(),
                ),
                err,
            )
        })?;

        docker_networks
            .iter()
            .find(|network| network.name == PANDA_NETWORK)
            .ok_or_else(|| {
                RoutineFailure::error(Message::new(
                    "Failed".to_string(),
                    "to find panda house docker network".to_string(),
                ))
            })?;

        Ok(RoutineSuccess::success(Message::new(
            "Successfully".to_string(),
            "validated panda house docker network".to_string(),
        )))
    }
}

pub struct ValidateRedPandaCluster;
impl ValidateRedPandaCluster {
    pub fn new() -> Self {
        Self
    }
}
impl Routine for ValidateRedPandaCluster {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        let output = docker::run_rpk_cluster_info().map_err(|err| {
            RoutineFailure::new(
                Message::new(
                    "Failed".to_string(),
                    "to validate red panda cluster".to_string(),
                ),
                err,
            )
        })?;

        if output.contains("redpanda-1") {
            Ok(RoutineSuccess::success(Message::new(
                "Successfully".to_string(),
                "validated red panda cluster".to_string(),
            )))
        } else {
            Err(RoutineFailure::error(Message::new(
                "Failed".to_string(),
                "to validate red panda cluster".to_string(),
            )))
        }
    }
}

pub struct ValidateConsoleRun;
impl ValidateConsoleRun {
    pub fn new() -> Self {
        Self
    }
}
impl Routine for ValidateConsoleRun {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        let containers = docker::list_containers().map_err(|err| {
            RoutineFailure::new(
                Message::new("Failed".to_string(), "to get the containers".to_string()),
                err,
            )
        })?;

        // check that the clickhouse container exists
        containers
            .iter()
            .find(|container| container.names.contains(CONSOLE_CONTAINER_NAME))
            .ok_or_else(|| {
                RoutineFailure::error(Message::new(
                    "Failed".to_string(),
                    "to find console docker container".to_string(),
                ))
            })?;
        Ok(RoutineSuccess::success(Message::new(
            "Successfully".to_string(),
            "validated console docker container".to_string(),
        )))
    }
}

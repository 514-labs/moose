use super::{Routine, RoutineFailure, RoutineSuccess};
use crate::{
    cli::{display::Message, DebugStatus},
    constants::PANDA_NETWORK,
    utilities::docker,
};
use std::io::{self, Error, ErrorKind, Write};

pub struct ValidateClickhouseRun(DebugStatus);
impl ValidateClickhouseRun {
    pub fn new(debug: DebugStatus) -> Self {
        Self(debug)
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
            .find(|container| container.names.contains("clickhousedb-1"))
            .ok_or_else(|| {
                RoutineFailure::new(
                    Message::new(
                        "Failed".to_string(),
                        "to find clickhouse docker container".to_string(),
                    ),
                    Error::new(
                        ErrorKind::Other,
                        "Failed to validate clickhouse container exists",
                    ),
                )
            })?;
        Ok(RoutineSuccess::success(Message::new(
            "Successfully".to_string(),
            "validated clickhouse docker container".to_string(),
        )))
    }
}

pub struct ValidateRedPandaRun(DebugStatus);
impl ValidateRedPandaRun {
    pub fn new(debug: DebugStatus) -> Self {
        Self(debug)
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
            .find(|container| container.names.contains("redpanda-1"))
            .ok_or_else(|| {
                RoutineFailure::new(
                    Message::new(
                        "Failed".to_string(),
                        "to find redpanda docker container".to_string(),
                    ),
                    Error::new(
                        ErrorKind::Other,
                        "Failed to validate redpanda container exists",
                    ),
                )
            })?;
        Ok(RoutineSuccess::success(Message::new(
            "Successfully".to_string(),
            "validated redpanda docker container".to_string(),
        )))
    }
}

pub struct ValidatePandaHouseNetwork(DebugStatus);
impl ValidatePandaHouseNetwork {
    pub fn new(debug: DebugStatus) -> Self {
        Self(debug)
    }
}
impl Routine for ValidatePandaHouseNetwork {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        let output = docker::network_list().map_err(|err| {
            RoutineFailure::new(
                Message::new(
                    "Failed".to_string(),
                    "to validate panda house docker network".to_string(),
                ),
                err,
            )
        })?;

        if self.0 == DebugStatus::Debug {
            io::stdout().write_all(&output.stdout).unwrap();
        }

        let string_output = String::from_utf8(output.stdout).unwrap();

        if string_output.contains(PANDA_NETWORK) {
            Ok(RoutineSuccess::success(Message::new(
                "Successfully".to_string(),
                "validated panda house docker network".to_string(),
            )))
        } else {
            Err(RoutineFailure::new(
                Message::new(
                    "Failed".to_string(),
                    "to validate panda house docker network".to_string(),
                ),
                Error::new(
                    ErrorKind::Other,
                    "Failed to validate panda house network exists",
                ),
            ))
        }
    }
}

pub struct ValidateRedPandaCluster(DebugStatus);
impl ValidateRedPandaCluster {
    pub fn new(debug: DebugStatus) -> Self {
        Self(debug)
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

        if self.0 == DebugStatus::Debug {
            io::stdout().write_all(&output.stdout).unwrap();
        }

        let string_output = String::from_utf8(output.stdout).unwrap();

        if string_output.contains("redpanda-1") {
            Ok(RoutineSuccess::success(Message::new(
                "Successfully".to_string(),
                "validated red panda cluster".to_string(),
            )))
        } else {
            Err(RoutineFailure::new(
                Message::new(
                    "Failed".to_string(),
                    "to validate red panda cluster".to_string(),
                ),
                Error::new(ErrorKind::Other, "Failed to validate red panda cluster"),
            ))
        }
    }
}

use std::io::{self, Write};
use crate::{cli::{display::Message, DebugStatus}, framework::directories, utilities::docker::{self}, infrastructure::{olap::clickhouse::ClickhouseConfig, stream::redpanda::RedpandaConfig}};
use super::{RoutineFailure, RoutineSuccess, Routine, initialize::ValidateMountVolumes, validate::{ValidateRedPandaRun, ValidateClickhouseRun, ValidatePandaHouseNetwork}};

pub struct RunLocalInfratructure {
    debug: DebugStatus,
    clickhouse_config: ClickhouseConfig,
    redpanda_config: RedpandaConfig,
}
impl RunLocalInfratructure {
    pub fn new(debug: DebugStatus, clickhouse_config: ClickhouseConfig, redpanda_config: RedpandaConfig) -> Self {
        Self { debug, clickhouse_config, redpanda_config }
    }
}

impl Routine for RunLocalInfratructure {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        let igloo_dir = directories::get_igloo_directory().map_err(|err| {
            RoutineFailure::new(Message::new("Failed".to_string(), "to get .igloo directory. Try running `igloo init`".to_string()), err)
        })?;
        // Model this after the `spin_up` function in `apps/igloo-kit-cli/src/cli/routines/start.rs` but use routines instead
        ValidateMountVolumes::new(igloo_dir).run_silent()?;
        ValidatePandaHouseNetwork::new(self.debug).run_silent()?;
        RunRedPandaContainer::new(self.debug, self.redpanda_config).run_silent()?;
        ValidateRedPandaRun::new(self.debug).run_silent()?;
        RunClickhouseContainer::new(self.debug, self.clickhouse_config.clone()).run_silent()?;
        ValidateClickhouseRun::new(self.debug).run_silent()?;
        Ok(RoutineSuccess::success(Message::new("Successfully".to_string(), "ran local infrastructure".to_string())))
    }
}

    
pub struct RunRedPandaContainer { debug: DebugStatus, redpanda_config: RedpandaConfig }
impl RunRedPandaContainer {
    pub fn new(debug: DebugStatus, redpanda_config: RedpandaConfig) -> Self {
        Self { debug, redpanda_config }
    }
}

impl Routine for RunRedPandaContainer {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        let igloo_dir = directories::get_igloo_directory().map_err(|err| {
            RoutineFailure::new(Message::new("Failed".to_string(), "to get .igloo directory. Try running `igloo init`".to_string()), err)
        })?;

        let output = docker::run_red_panda(igloo_dir).map_err(|err| {
            RoutineFailure::new(Message::new("Failed".to_string(), "to run redpanda container".to_string()), err)
        })?;

        if self.debug == DebugStatus::Debug {
            println!("Debugging red panda container run");
            println!("{}", &output.status);
            io::stdout().write_all(&output.stdout).unwrap();
        } 
        
        Ok(RoutineSuccess::success(Message::new("Successfully".to_string(), "ran redpanda container".to_string())))
    }
}


pub struct RunClickhouseContainer{ debug: DebugStatus, clickhouse_config: ClickhouseConfig }
impl RunClickhouseContainer {
    pub fn new(debug: DebugStatus, clickhouse_config: ClickhouseConfig) -> Self {
        Self { debug, clickhouse_config }
    }
}

impl Routine for RunClickhouseContainer {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        let igloo_dir = directories::get_igloo_directory().map_err(|err| {
            RoutineFailure::new(Message::new("Failed".to_string(), "to get .igloo directory. Try running `igloo init`".to_string()), err)
        })?;

        let output = docker::run_clickhouse(igloo_dir, self.clickhouse_config.clone()).map_err(|err| {
            RoutineFailure::new(Message::new("Failed".to_string(), "to run clickhouse container".to_string()), err)
        })?;

        if self.debug == DebugStatus::Debug {
            println!("Debugging clickhouse container run");
            io::stdout().write_all(&output.stdout).unwrap();
        } 
        
        Ok(RoutineSuccess::success(Message::new("Successfully".to_string(), "ran clickhouse container".to_string())))
    }
}


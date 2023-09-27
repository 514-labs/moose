mod commands;
mod routines;
mod config;
mod watcher;
mod local_webserver;
mod display;

use std::sync::{RwLock, Arc};

use commands::Commands;
use config::{read_config, Config};
use clap::Parser;
use crate::{framework::{AddableObjects, directories::get_igloo_directory}, infrastructure::{olap::clickhouse::ClickhouseConfig, PANDA_NETWORK, stream::redpanda::RedpandaConfig}};
use self::{commands::AddArgs, display::{MessageType, Message, show_message, CommandTerminal}, routines::{initialize::InitializeProject, validate::ValidateRedPandaCluster, RoutineController, RunMode, start::RunLocalInfratructure, Routine, stop::StopLocalInfrastructure, clean::CleanProject}};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
    name: Option<String>,

    // TODD: Add a config file option
    /// Sets a custom config file
    // #[arg(short, long, value_name = "FILE")]
    // config: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long)]
    debug: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

fn add_handler(add_arg: &AddArgs) {
    match &add_arg.command {
        Some(AddableObjects::IngestPoint) => {
            todo!("add ingestion point object");
        }
        Some(AddableObjects::Flow) => {
            todo!("add flow object");
        }
        Some(AddableObjects::Dataframe) => {
            todo!("add dataframe object");
        }
        Some(AddableObjects::Metric) => {
            todo!("add metric object");
        }
        Some(AddableObjects::Dashboard) => {
            todo!("add dashboard object")
        }
        Some(AddableObjects::Model) => {
            todo!("add model object")
        }
        None => {
            todo!("add a great, piffy, and helpful message here")
        }
    }
}




#[derive(PartialEq, Clone, Copy)]
pub enum DebugStatus {
    Debug,
    Silent,
}

async fn top_command_handler(term: Arc<RwLock<CommandTerminal>>, config: Config, commands: &Option<Commands>, debug: DebugStatus) {
    let clickhouse_config = ClickhouseConfig {
        db_name: "local".to_string(),
        user: "panda".to_string(),
        password: "pandapass".to_string(),
        host: "localhost".to_string(),
        host_port: 18123,
        postgres_port: 9005,
        kafka_port: 9092,
        cluster_network: PANDA_NETWORK.to_owned(),
    };

    let redpanda_config: RedpandaConfig = RedpandaConfig {
        broker: "localhost:19092",
        message_timeout_ms: 1000,

    };

    if !config.features.coming_soon_wall {
        match commands {
            Some(Commands::Init {}) => {
                let mut controller = RoutineController::new();
                let run_mode = RunMode::Explicit { term };
                controller.add_routine(Box::new(InitializeProject::new(run_mode.clone())));
                controller.run_routines(run_mode);
            }
            Some(Commands::Dev{}) => {
                let mut controller = RoutineController::new();
                let run_mode = RunMode::Explicit { term };
                controller.add_routine(Box::new(RunLocalInfratructure::new(DebugStatus::Debug, clickhouse_config.clone(), redpanda_config.clone())));
                controller.add_routine(Box::new(ValidateRedPandaCluster::new(DebugStatus::Debug)));
                controller.run_routines(run_mode);
                routines::start_development_mode(clickhouse_config.clone(), redpanda_config.clone()).await;      

            }
            Some(Commands::Update{}) => {
                // This command may not be needed if we have incredible automation
                todo!("Will update the project's underlying infrascructure based on any added objects")
            }
            Some(Commands::Stop{}) => {
                let mut controller = RoutineController::new();
                let run_mode = RunMode::Explicit { term };
                controller.add_routine(Box::new(StopLocalInfrastructure::new(run_mode.clone())));
                controller.run_routines(run_mode);
            }
            Some(Commands::Clean{}) => {
                let run_mode = RunMode::Explicit { term };
                let igloo_dir = get_igloo_directory().expect("Nothing to clean, no .igloo directory found");
                let mut controller = RoutineController::new();
                controller.add_routine(Box::new(CleanProject::new(igloo_dir,run_mode.clone())));
                controller.run_routines(run_mode);
                

            }
            Some(Commands::Add(add_args)) => {
                add_handler(add_args);   
            }
            None => {}
        }
    } else {
        show_message(term, MessageType::Banner, Message {
            action: "Coming Soon".to_string(),
            details: "Join the IglooKit community to stay up to date on the latest features: https://discord.gg/WX3V3K4QCc".to_string(),
        });
    }
    
}

pub async fn cli_run() {
    let term = Arc::new(RwLock::new(CommandTerminal::new()));
    let config = read_config(term.clone());
    let cli = Cli::parse();
    let debug_status = if cli.debug { DebugStatus::Debug } else { DebugStatus::Silent };

    top_command_handler(term.clone(), config, &cli.command, debug_status).await
}

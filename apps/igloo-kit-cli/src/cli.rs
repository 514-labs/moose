mod commands;
mod display;
mod local_webserver;
mod logger;
mod routines;
mod settings;
mod watcher;

use std::sync::{Arc, RwLock};

use self::settings::setup_igloo_user_directory;
use log::info;

use self::{
    commands::AddArgs,
    display::{show_message, CommandTerminal, Message, MessageType},
    routines::{
        clean::CleanProject, initialize::InitializeProject, start::RunLocalInfratructure,
        stop::StopLocalInfrastructure, validate::ValidateRedPandaCluster, RoutineController,
        RunMode,
    },
};
use crate::{
    framework::{directories::get_igloo_directory, AddableObjects},
    project::Project,
};
use clap::Parser;
use commands::Commands;
use logger::setup_logging;
use settings::{read_settings, Settings};

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

async fn top_command_handler(
    term: Arc<RwLock<CommandTerminal>>,
    settings: Settings,
    commands: &Option<Commands>,
    debug: DebugStatus,
) {
    if !settings.features.coming_soon_wall {
        match commands {
            Some(Commands::Init {
                name,
                language,
                location,
            }) => {
                info!(
                    "Running init command with name: {}, language: {}, location: {}",
                    name, language, location
                );

                let mut controller = RoutineController::new();
                let run_mode = RunMode::Explicit { term };

                let project = Project::new(name.clone(), *language, location.clone());

                controller.add_routine(Box::new(InitializeProject::new(
                    run_mode.clone(),
                    project.clone(),
                )));
                controller.run_routines(run_mode);
                project
                    .write_to_file()
                    .expect("Failed to write project to file");
            }
            Some(Commands::Dev {}) => {
                let mut controller = RoutineController::new();
                let run_mode = RunMode::Explicit { term };
                let project = Project::from_file()
                    .expect("No project found, please run `igloo init` to create a project");
                controller.add_routine(Box::new(RunLocalInfratructure::new(
                    debug,
                    settings.clickhouse.clone(),
                    settings.redpanda.clone(),
                    project.clone(),
                )));
                controller.add_routine(Box::new(ValidateRedPandaCluster::new(debug)));
                controller.run_routines(run_mode);
                let _ = routines::start_development_mode(
                    project.clone(),
                    settings.clickhouse.clone(),
                    settings.redpanda.clone(),
                    settings.local_webserver,
                )
                .await;
            }
            // Some(Commands::Link { name, language, location, project }) => {
            //     let mut controller = RoutineController::new();
            //     let run_mode = RunMode::Explicit { term };

            //     // controller.add_routine(Box::new(InitializeClient::new(
            //     //     run_mode.clone(),
            //     //     project.clone(),
            //     // )));
            //     controller.run_routines(run_mode);
            //     project
            //         .write_to_file()
            //         .expect("Failed to write project to file");
            // }
            Some(Commands::Update {}) => {
                // This command may not be needed if we have incredible automation
                todo!("Will update the project's underlying infrascructure based on any added objects")
            }
            Some(Commands::Stop {}) => {
                let mut controller = RoutineController::new();
                let run_mode = RunMode::Explicit { term };
                controller.add_routine(Box::new(StopLocalInfrastructure::new(run_mode.clone())));
                controller.run_routines(run_mode);
            }
            Some(Commands::Clean {}) => {
                let run_mode = RunMode::Explicit { term };
                let project = Project::from_file()
                    .expect("No project found, please run `igloo init` to create a project");
                let igloo_dir = get_igloo_directory(project)
                    .expect("Nothing to clean, no .igloo directory found");
                let mut controller = RoutineController::new();
                controller.add_routine(Box::new(CleanProject::new(igloo_dir, run_mode.clone())));
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
            details: "Join the IglooKit community to stay up to date on the latest features: https://join.slack.com/t/igloocommunity/shared_invite/zt-25gsnx2x2-9ttVTt4L9LYFrRcM6jimcg".to_string(),
        });
    }
}

pub async fn cli_run() {
    setup_igloo_user_directory().expect("Failed to setup igloo user directory");

    let term = Arc::new(RwLock::new(CommandTerminal::new()));

    let config = read_settings(term.clone()).unwrap();
    setup_logging(config.logger.clone()).expect("Failed to setup logging");

    info!("Configuration loaded and logging setup");

    let cli = Cli::parse();
    let debug_status = if cli.debug {
        DebugStatus::Debug
    } else {
        DebugStatus::Silent
    };

    top_command_handler(term.clone(), config, &cli.command, debug_status).await
}

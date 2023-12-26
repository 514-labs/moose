#[macro_use]
mod display;

mod commands;
pub mod local_webserver;
mod logger;
mod routines;
mod settings;
mod watcher;

use self::settings::setup_user_directory;
use log::{debug, info};

use self::{
    display::{Message, MessageType},
    routines::{
        clean::CleanProject, initialize::InitializeProject, start::RunLocalInfratructure,
        stop::StopLocalInfrastructure, validate::ValidateRedPandaCluster, RoutineController,
        RunMode,
    },
};
use crate::project::Project;
use clap::Parser;
use commands::Commands;
use logger::setup_logging;
use settings::{read_settings, Settings};
use std::path::Path;

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

#[derive(PartialEq, Clone, Copy)]
pub enum DebugStatus {
    Debug,
    Silent,
}

async fn top_command_handler(settings: Settings, commands: &Option<Commands>, debug: DebugStatus) {
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

                let dir_path = Path::new(location);
                let project = Project::from_dir(dir_path, name.clone(), *language);

                debug!("Project: {:?}", project);

                let mut controller = RoutineController::new();
                let run_mode = RunMode::Explicit {};

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
                let project = Project::load_from_current_dir()
                    .expect("No project found, please run `igloo init` to create a project");

                let mut controller = RoutineController::new();
                let run_mode = RunMode::Explicit {};

                controller
                    .add_routine(Box::new(RunLocalInfratructure::new(debug, project.clone())));

                controller.add_routine(Box::new(ValidateRedPandaCluster::new(debug)));

                controller.run_routines(run_mode);
                let _ = routines::start_development_mode(&project).await;
            }
            Some(Commands::Update {}) => {
                // This command may not be needed if we have incredible automation
                todo!("Will update the project's underlying infrascructure based on any added objects")
            }
            Some(Commands::Stop {}) => {
                let mut controller = RoutineController::new();
                let run_mode = RunMode::Explicit {};
                controller.add_routine(Box::new(StopLocalInfrastructure::new(run_mode.clone())));
                controller.run_routines(run_mode);
            }
            Some(Commands::Clean {}) => {
                let run_mode = RunMode::Explicit {};
                let project = Project::load_from_current_dir()
                    .expect("No project found, please run `igloo init` to create a project");

                let mut controller = RoutineController::new();
                controller.add_routine(Box::new(CleanProject::new(project, run_mode.clone())));
                controller.run_routines(run_mode);
            }
            None => {}
        }
    } else {
        show_message!(MessageType::Banner, Message {
            action: "Coming Soon".to_string(),
            details: "Join the IglooKit community to stay up to date on the latest features: https://join.slack.com/t/igloocommunity/shared_invite/zt-25gsnx2x2-9ttVTt4L9LYFrRcM6jimcg".to_string(),
        });
    }
}

pub async fn cli_run() {
    setup_user_directory().expect("Failed to setup igloo user directory");

    let config = read_settings().unwrap();
    setup_logging(config.logger.clone()).expect("Failed to setup logging");

    info!("Configuration loaded and logging setup");

    info!("Feature Configuration: {:?}", config.features);

    let cli = Cli::parse();
    let debug_status = if cli.debug {
        DebugStatus::Debug
    } else {
        DebugStatus::Silent
    };

    top_command_handler(config, &cli.command, debug_status).await
}

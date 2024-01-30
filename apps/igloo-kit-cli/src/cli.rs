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
        clean::CleanProject, initialize::InitializeProject, start::RunLocalInfrastructure,
        stop::StopLocalInfrastructure, validate::ValidateRedPandaCluster, RoutineController,
        RunMode,
    },
};
use crate::cli::settings::init_config_file;
use crate::project::Project;
use clap::Parser;
use commands::Commands;
use logger::setup_logging;
use settings::{read_settings, Settings};
use std::path::Path;

#[derive(Parser)]
#[command(author, version, about, long_about = None, arg_required_else_help(true))]
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
    command: Commands,
}

async fn top_command_handler(settings: Settings, commands: &Commands) {
    if !settings.features.coming_soon_wall {
        match commands {
            Commands::Init {
                name,
                language,
                location,
            } => {
                info!(
                    "Running init command with name: {}, language: {}, location: {}",
                    name, language, location
                );

                let dir_path = Path::new(location);
                let project = Project::from_dir(dir_path, name.clone(), *language);

                debug!("Project: {:?}", project);

                let mut controller = RoutineController::new();
                let run_mode = RunMode::Explicit {};

                controller.add_routine(Box::new(InitializeProject::new(run_mode, project.clone())));

                controller.run_routines(run_mode);

                project
                    .write_to_file()
                    .expect("Failed to write project to file");
            }
            Commands::Dev {} => {
                info!("Running dev command");

                let project = Project::load_from_current_dir()
                    .expect("No project found, please run `igloo init` to create a project");

                let mut controller = RoutineController::new();
                let run_mode = RunMode::Explicit {};

                controller.add_routine(Box::new(RunLocalInfrastructure::new(project.clone())));

                controller.add_routine(Box::new(ValidateRedPandaCluster::new()));

                controller.run_routines(run_mode);

                routines::start_development_mode(&project).await.unwrap();
            }
            Commands::Update {} => {
                // This command may not be needed if we have incredible automation
                todo!("Will update the project's underlying infrastructure based on any added objects")
            }
            Commands::Stop {} => {
                let mut controller = RoutineController::new();
                let run_mode = RunMode::Explicit {};
                controller.add_routine(Box::new(StopLocalInfrastructure::new(run_mode)));
                controller.run_routines(run_mode);
            }
            Commands::Clean {} => {
                let run_mode = RunMode::Explicit {};
                let project = Project::load_from_current_dir()
                    .expect("No project found, please run `igloo init` to create a project");

                let mut controller = RoutineController::new();
                controller.add_routine(Box::new(CleanProject::new(project, run_mode)));
                controller.run_routines(run_mode);
            }
        }
    } else {
        show_message!(MessageType::Banner, Message {
            action: "Coming Soon".to_string(),
            details: "Join the MooseJS community to stay up to date on the latest features: https://join.slack.com/t/igloocommunity/shared_invite/zt-25gsnx2x2-9ttVTt4L9LYFrRcM6jimcg".to_string(),
        });
    }
}

pub async fn cli_run() {
    setup_user_directory().expect("Failed to setup igloo user directory");
    init_config_file().unwrap();

    let config = read_settings().unwrap();
    setup_logging(config.logger.clone()).expect("Failed to setup logging");

    info!("CLI Configuration loaded and logging setup: {:?}", config);

    let cli = Cli::parse();

    top_command_handler(config, &cli.command).await
}

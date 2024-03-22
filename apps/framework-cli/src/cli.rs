#[macro_use]
mod display;

mod commands;
pub mod local_webserver;
mod logger;
mod routines;
mod settings;
mod watcher;

use std::path::Path;
use std::process::exit;
use std::sync::Arc;

use clap::Parser;
use commands::Commands;
use home::home_dir;
use log::{debug, info};
use logger::setup_logging;
use settings::{read_settings, Settings};

use crate::cli::routines::migrate::GenerateMigration;
use crate::cli::routines::start::CopyOldSchema;
use crate::cli::{
    display::{Message, MessageType},
    routines::{
        initialize::InitializeProject, start::RunLocalInfrastructure,
        validate::ValidateRedPandaCluster, RoutineController, RunMode,
    },
    settings::{init_config_file, setup_user_directory},
};
use crate::project::Project;
use crate::utilities::constants::{CONTEXT, CTX_SESSION_ID};
use crate::utilities::git::is_git_repo;

use self::routines::{
    clean::CleanProject, clean::DeleteVersions, docker_packager::BuildDockerfile,
    docker_packager::CreateDockerfile, stop::StopLocalInfrastructure,
};

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

                if dir_path.canonicalize().unwrap() == home_dir().unwrap().canonicalize().unwrap() {
                    show_message!(
                        MessageType::Error,
                        Message {
                            action: "Init".to_string(),
                            details: "You cannot create a project in your home directory"
                                .to_string(),
                        }
                    );
                    exit(1);
                }

                let project = Project::new(dir_path, name.clone(), *language);
                let project_arc = Arc::new(project);

                debug!("Project: {:?}", project_arc);

                crate::utilities::capture::capture!(
                    ActivityType::InitCommand,
                    CONTEXT.get(CTX_SESSION_ID).unwrap().clone(),
                    name.clone()
                );

                let mut controller = RoutineController::new();
                let run_mode = RunMode::Explicit {};

                controller.add_routine(Box::new(InitializeProject::new(
                    run_mode,
                    project_arc.clone(),
                )));

                controller.run_routines(run_mode);

                project_arc
                    .write_to_disk()
                    .expect("Failed to write project to file");

                let is_git_repo =
                    is_git_repo(dir_path).expect("Failed to check if directory is a git repo");
                if !is_git_repo {
                    crate::utilities::git::create_init_commit(project_arc, dir_path);
                    show_message!(
                        MessageType::Success,
                        Message::new("Created".to_string(), "Git Repository".to_string())
                    );
                }
            }
            Commands::Build { docker } => {
                let run_mode = RunMode::Explicit {};
                info!("Running build command");
                let project = Project::load_from_current_dir()
                    .expect("No project found, please run `moose init` to create a project");
                let project_arc = Arc::new(project);

                crate::utilities::capture::capture!(
                    ActivityType::BuildCommand,
                    CONTEXT.get(CTX_SESSION_ID).unwrap().clone(),
                    project_arc.name().clone()
                );

                let mut controller = RoutineController::new();

                // Remove versions directory so only the relevant versions will be populated
                let internal_directory = project_arc.internal_dir().unwrap();
                controller.add_routine(Box::new(DeleteVersions::new(internal_directory)));

                // Copy the old schema
                controller.add_routine(Box::new(CopyOldSchema::new(project_arc.clone())));

                // docker flag is true then build docker images
                if *docker {
                    crate::utilities::capture::capture!(
                        ActivityType::DockerCommand,
                        CONTEXT.get(CTX_SESSION_ID).unwrap().clone(),
                        project_arc.name().clone()
                    );
                    controller.add_routine(Box::new(CreateDockerfile::new(project_arc.clone())));
                    controller.add_routine(Box::new(BuildDockerfile::new(project_arc.clone())));
                }
                controller.run_routines(run_mode);
            }
            Commands::Dev {} => {
                info!("Running dev command");

                let project = Project::load_from_current_dir()
                    .expect("No project found, please run `moose init` to create a project");

                let _ = project.set_enviroment(false);
                let project_arc = Arc::new(project);

                crate::utilities::capture::capture!(
                    ActivityType::DevCommand,
                    CONTEXT.get(CTX_SESSION_ID).unwrap().clone(),
                    project_arc.name().clone()
                );

                let mut controller = RoutineController::new();
                let run_mode = RunMode::Explicit {};

                controller.add_routine(Box::new(RunLocalInfrastructure::new(project_arc.clone())));

                controller.add_routine(Box::new(ValidateRedPandaCluster::new(
                    project_arc.name().clone(),
                )));

                controller.add_routine(Box::new(CopyOldSchema::new(project_arc.clone())));

                controller.run_routines(run_mode);

                routines::start_development_mode(project_arc).await.unwrap();
            }
            Commands::Migrate {} => {
                info!("Running migrate command");
                let project = Project::load_from_current_dir()
                    .expect("No project found, please run `moose init` to create a project");

                let mut controller = RoutineController::new();
                let run_mode = RunMode::Explicit {};

                controller.add_routine(Box::new(GenerateMigration::new(Arc::new(project))));
                controller.run_routines(run_mode);
            }
            Commands::Prod {} => {
                info!("Running prod command");
                let project = Project::load_from_current_dir()
                    .expect("No project found, please run `moose init` to create a project");

                let _ = project.set_enviroment(true);
                let project_arc = Arc::new(project);

                crate::utilities::capture::capture!(
                    ActivityType::ProdCommand,
                    CONTEXT.get(CTX_SESSION_ID).unwrap().clone(),
                    project_arc.name().clone()
                );

                routines::start_production_mode(project_arc).await.unwrap();
            }
            Commands::Update {} => {
                // This command may not be needed if we have incredible automation
                todo!("Will update the project's underlying infrastructure based on any added objects")
            }
            Commands::Stop {} => {
                let mut controller = RoutineController::new();
                let run_mode = RunMode::Explicit {};
                let project = Project::load_from_current_dir()
                    .expect("No project found, please run `moose init` to create a project");
                let project_arc = Arc::new(project);

                crate::utilities::capture::capture!(
                    ActivityType::StopCommand,
                    CONTEXT.get(CTX_SESSION_ID).unwrap().clone(),
                    project_arc.name().clone()
                );

                controller.add_routine(Box::new(StopLocalInfrastructure::new(project_arc)));
                controller.run_routines(run_mode);
            }
            Commands::Clean {} => {
                let run_mode = RunMode::Explicit {};
                let project = Project::load_from_current_dir()
                    .expect("No project found, please run `moose init` to create a project");
                let project_arc = Arc::new(project);

                crate::utilities::capture::capture!(
                    ActivityType::CleanCommand,
                    CONTEXT.get(CTX_SESSION_ID).unwrap().clone(),
                    project_arc.name().clone()
                );

                let mut controller = RoutineController::new();
                controller.add_routine(Box::new(CleanProject::new(project_arc, run_mode)));
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
    setup_user_directory().expect("Failed to setup moose user directory");
    init_config_file().unwrap();

    let config = read_settings().unwrap();
    setup_logging(config.logger.clone()).expect("Failed to setup logging");

    info!("CLI Configuration loaded and logging setup: {:?}", config);

    let cli = Cli::parse();

    top_command_handler(config, &cli.command).await
}

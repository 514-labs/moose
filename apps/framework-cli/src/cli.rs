#[macro_use]
mod display;

mod commands;
pub mod local_webserver;
mod logger;
mod routines;
pub mod settings;
mod watcher;

use std::cmp::Ordering;
use std::path::Path;
use std::process::exit;
use std::sync::Arc;

use clap::Parser;
use commands::{Commands, FlowCommands, GenerateCommand};
use config::ConfigError;
use home::home_dir;
use log::{debug, info};
use logger::setup_logging;
use settings::{read_settings, Settings};

use crate::cli::routines::dev::{copy_old_schema, create_deno_files, create_models_volume};
use crate::cli::routines::flow::{create_flow_directory, create_flow_file};
use crate::cli::routines::templates;
use crate::cli::routines::version::BumpVersion;
use crate::cli::routines::{RoutineFailure, RoutineSuccess};
use crate::cli::{
    display::{Message, MessageType},
    routines::{
        dev::run_local_infrastructure, initialize::InitializeProject, RoutineController, RunMode,
    },
    settings::{init_config_file, setup_user_directory},
};
use crate::infrastructure::olap::clickhouse::version_sync::{parse_version, version_to_string};
use crate::project::Project;
use crate::utilities::constants::CLI_VERSION;
use crate::utilities::git::is_git_repo;

use self::routines::{
    clean::CleanProject, docker_packager::BuildDockerfile, docker_packager::CreateDockerfile,
    migrate::GenerateMigration,
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

fn load_project() -> Result<Project, RoutineFailure> {
    Project::load_from_current_dir().map_err(|e| match e {
        ConfigError::Foreign(_) => RoutineFailure::error(Message {
            action: "Loading".to_string(),
            details: "No project found, please run `moose init` to create a project".to_string(),
        }),
        _ => RoutineFailure::error(Message {
            action: "Loading".to_string(),
            details: format!("Please validate the project's configs: {:?}", e),
        }),
    })
}

async fn top_command_handler(
    settings: Settings,
    commands: &Commands,
) -> Result<RoutineSuccess, RoutineFailure> {
    match commands {
        Commands::Init {
            name,
            language,
            location,
            template,
            no_fail_already_exists,
        } => {
            info!(
                "Running init command with name: {}, language: {}, location: {:?}, template: {:?}",
                name, language, location, template
            );

            crate::utilities::capture::capture!(
                if template.is_some() {
                    ActivityType::InitTemplateCommand
                } else {
                    ActivityType::InitCommand
                },
                name.clone(),
                &settings
            );

            let dir_path = Path::new(location.as_deref().unwrap_or(name));
            if !no_fail_already_exists && dir_path.exists() {
                return Err(RoutineFailure::error(Message {
                    action: "Init".to_string(),
                    details:
                        "Directory already exists, please use the --no-fail-already-exists flag if this is expected."
                            .to_string(),
                }));
            }

            std::fs::create_dir_all(dir_path).expect("Failed to create directory");

            if dir_path.canonicalize().unwrap() == home_dir().unwrap().canonicalize().unwrap() {
                return Err(RoutineFailure::error(Message {
                    action: "Init".to_string(),
                    details: "You cannot create a project in your home directory".to_string(),
                }));
            }

            // TODO: refactor this to be extracted in different functions
            match template {
                Some(template) => {
                    templates::generate_template(template, CLI_VERSION, dir_path).await
                }
                None => {
                    let project = Project::new(dir_path, name.clone(), *language);
                    let project_arc = Arc::new(project);

                    debug!("Project: {:?}", project_arc);

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
                            MessageType::Info,
                            Message {
                                action: "Init".to_string(),
                                details: "Created a new git repository".to_string(),
                            }
                        );
                    }

                    {
                        show_message!(
                            MessageType::Success,
                            Message {
                                action: "Success!".to_string(),
                                details: format!(
                                    "Created project at {} 🚀",
                                    dir_path.to_string_lossy()
                                ),
                            }
                        );
                    }

                    {
                        show_message!(
                            MessageType::Info,
                            Message {
                                action: "".to_string(),
                                details: "\n".to_string(),
                            }
                        );
                    }

                    Ok(RoutineSuccess::highlight(Message::new(
                        "Get Started".to_string(),
                        format!("\n\n📂 Go to your project directory: \n\t$ cd {}\n\n🛠️  Start dev server: \n\t$ npx @514labs/moose-cli dev\n\n", dir_path.to_string_lossy()),
                    )))
                }
            }
        }
        Commands::Build { docker } => {
            let run_mode = RunMode::Explicit {};
            info!("Running build command");
            let project: Project = load_project()?;
            let project_arc = Arc::new(project);

            crate::utilities::capture::capture!(
                ActivityType::BuildCommand,
                project_arc.name().clone(),
                &settings
            );

            let mut controller = RoutineController::new();

            // Remove versions directory so only the relevant versions will be populated
            project_arc.delete_old_versions().map_err(|e| {
                RoutineFailure::error(Message {
                    action: "Build".to_string(),
                    details: format!("Failed to delete old versions: {:?}", e),
                })
            })?;

            copy_old_schema(&project_arc)?;

            // docker flag is true then build docker images
            if *docker {
                crate::utilities::capture::capture!(
                    ActivityType::DockerCommand,
                    project_arc.name().clone(),
                    &settings
                );
                // TODO get rid of the routines and use functions instead
                controller.add_routine(Box::new(CreateDockerfile::new(project_arc.clone())));
                controller.add_routine(Box::new(BuildDockerfile::new(project_arc.clone())));
                controller.run_routines(run_mode);

                Ok(RoutineSuccess::success(Message::new(
                    "Built".to_string(),
                    "Docker images".to_string(),
                )))
            } else {
                Err(RoutineFailure::error(Message {
                    action: "Build".to_string(),
                    details: "Docker flag is not set and is currently mandatory".to_string(),
                }))
            }
        }
        Commands::Dev {} => {
            info!("Running dev command");

            let project = load_project()?;

            let _ = project.set_enviroment(false);
            let project_arc = Arc::new(project);

            crate::utilities::capture::capture!(
                ActivityType::DevCommand,
                project_arc.name().clone(),
                &settings
            );

            run_local_infrastructure(&project_arc)?;

            routines::start_development_mode(project_arc)
                .await
                .map_err(|e| {
                    RoutineFailure::error(Message {
                        action: "Dev".to_string(),
                        details: format!("Failed to start development mode: {:?}", e),
                    })
                })?;

            Ok(RoutineSuccess::success(Message::new(
                "Ran".to_string(),
                "local infrastructure".to_string(),
            )))
        }
        Commands::Generate(generate) => match generate.command {
            Some(GenerateCommand::Migrations {}) => {
                info!("Running generate migration command");
                let project = load_project()?;
                let project_arc = Arc::new(project);

                let mut controller = RoutineController::new();
                let run_mode = RunMode::Explicit {};

                copy_old_schema(&project_arc)?;

                // TODO get rid of the routines and use functions instead
                controller.add_routine(Box::new(GenerateMigration::new(project_arc)));
                controller.run_routines(run_mode);

                Ok(RoutineSuccess::success(Message::new(
                    "Generated".to_string(),
                    "migrations".to_string(),
                )))
            }
            None => Err(RoutineFailure::error(Message {
                action: "Generate".to_string(),
                details: "Please provide a subcommand".to_string(),
            })),
        },
        Commands::Prod {} => {
            info!("Running prod command");
            let project = load_project()?;

            let _ = project.set_enviroment(true);
            let project_arc = Arc::new(project);

            crate::utilities::capture::capture!(
                ActivityType::ProdCommand,
                project_arc.name().clone(),
                &settings
            );

            create_models_volume(&project_arc)?;
            create_deno_files(&project_arc)?;

            routines::start_production_mode(project_arc).await.unwrap();

            Ok(RoutineSuccess::success(Message::new(
                "Ran".to_string(),
                "production infrastructure".to_string(),
            )))
        }
        Commands::BumpVersion { new_version } => {
            let project = load_project()?;
            let project_arc = Arc::new(project);

            crate::utilities::capture::capture!(
                ActivityType::BumpVersionCommand,
                project_arc.name().clone(),
                &settings
            );

            let mut controller = RoutineController::new();
            let run_mode = RunMode::Explicit {};

            let new_version = match new_version {
                None => {
                    let current = parse_version(project_arc.version());
                    let bump_location = if current.len() > 1 { 1 } else { 0 };

                    let new_version = current
                        .into_iter()
                        .enumerate()
                        .map(|(i, v)| match i.cmp(&bump_location) {
                            Ordering::Less => v,
                            Ordering::Equal => v + 1,
                            Ordering::Greater => 0,
                        })
                        .collect::<Vec<i32>>();
                    version_to_string(&new_version)
                }
                Some(new_version) => new_version.clone(),
            };

            // TODO get rid of the routines and use functions instead
            controller.add_routine(Box::new(BumpVersion::new(project_arc.clone(), new_version)));
            controller.run_routines(run_mode);

            Ok(RoutineSuccess::success(Message::new(
                "Bumped".to_string(),
                "Version".to_string(),
            )))
        }
        Commands::Clean {} => {
            let run_mode = RunMode::Explicit {};
            let project = load_project()?;
            let project_arc = Arc::new(project);

            crate::utilities::capture::capture!(
                ActivityType::CleanCommand,
                project_arc.name().clone(),
                &settings
            );

            // TODO get rid of the routines and use functions instead
            let mut controller = RoutineController::new();
            controller.add_routine(Box::new(CleanProject::new(project_arc, run_mode)));
            controller.run_routines(run_mode);

            Ok(RoutineSuccess::success(Message::new(
                "Cleaned".to_string(),
                "Project".to_string(),
            )))
        }
        Commands::Flow(flow) => {
            info!("Running flow command");

            let flow_cmd = flow.command.as_ref().unwrap();
            match flow_cmd {
                FlowCommands::Init(init) => {
                    let project = load_project()?;
                    let project_arc = Arc::new(project);

                    crate::utilities::capture::capture!(
                        ActivityType::FlowInitCommand,
                        project_arc.name().clone(),
                        &settings
                    );

                    create_flow_directory(
                        &project_arc,
                        init.source.clone(),
                        init.destination.clone(),
                    )?;
                    create_flow_file(&project_arc, init.source.clone(), init.destination.clone())?;

                    Ok(RoutineSuccess::success(Message::new(
                        "Created".to_string(),
                        "Flow".to_string(),
                    )))
                }
            }
        }
    }
}

pub async fn cli_run() {
    setup_user_directory().expect("Failed to setup moose user directory");
    init_config_file().unwrap();

    let config = read_settings().unwrap();
    setup_logging(config.logger.clone()).expect("Failed to setup logging");

    info!("CLI Configuration loaded and logging setup: {:?}", config);

    let cli = Cli::parse();

    match top_command_handler(config, &cli.command).await {
        Ok(s) => {
            show_message!(s.message_type, s.message);
            exit(0);
        }
        Err(e) => {
            show_message!(e.message_type, e.message);
            exit(1);
        }
    };
}

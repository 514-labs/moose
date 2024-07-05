#[macro_use]
pub(crate) mod display;

mod commands;
pub mod local_webserver;
mod logger;
mod routines;
pub mod settings;
mod watcher;
use super::metrics::Metrics;
use std::cmp::Ordering;
use std::path::Path;
use std::process::exit;
use std::sync::Arc;

use clap::Parser;
use commands::{AggregationCommands, Commands, ConsumptionCommands, FlowCommands, GenerateCommand};
use config::ConfigError;
use display::with_spinner_async;
use home::home_dir;
use log::{debug, info};
use logger::setup_logging;
use regex::Regex;
use routines::ls::{list_db, list_streaming};
use routines::plan;
use routines::ps::show_processes;
use settings::{read_settings, Settings};

use crate::cli::routines::aggregation::create_aggregation_file;
use crate::cli::routines::consumption::create_consumption_file;

use crate::cli::routines::dev::copy_old_schema;
use crate::cli::routines::flow::create_flow_file;
use crate::cli::routines::initialize::initialize_project;
use crate::cli::routines::logs::{follow_logs, show_logs};
use crate::cli::routines::migrate::generate_migration;
use crate::cli::routines::templates;
use crate::cli::routines::version::bump_version;
use crate::cli::routines::{RoutineFailure, RoutineSuccess};
use crate::cli::{
    display::{Message, MessageType},
    routines::{dev::run_local_infrastructure, RoutineController, RunMode},
    settings::{init_config_file, setup_user_directory},
};
use crate::framework::core::code_loader::load_framework_objects;
use crate::framework::languages::SupportedLanguages;
use crate::framework::sdk::ingest::generate_sdk;
use crate::infrastructure::olap::clickhouse::version_sync::{parse_version, version_to_string};
use crate::project::Project;
use crate::utilities::constants::{CLI_VERSION, PROJECT_NAME_ALLOW_PATTERN};
use crate::utilities::git::is_git_repo;

use self::routines::{
    clean::CleanProject, docker_packager::BuildDockerfile, docker_packager::CreateDockerfile,
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

fn check_project_name(name: &str) -> Result<(), RoutineFailure> {
    let project_name_regex = Regex::new(PROJECT_NAME_ALLOW_PATTERN).unwrap();

    if !project_name_regex.is_match(name) {
        return Err(RoutineFailure::error(Message {
            action: "Init".to_string(),
            details: format!(
                "Project name should match the following: {}",
                PROJECT_NAME_ALLOW_PATTERN
            ),
        }));
    }
    Ok(())
}

async fn top_command_handler(
    settings: Settings,
    commands: &Commands,
    metrics: Arc<Metrics>,
) -> Result<RoutineSuccess, RoutineFailure> {
    match commands {
        Commands::Init {
            name,
            language,
            location,
            template,
            no_fail_already_exists,
            empty,
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

            check_project_name(name)?;

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

                    initialize_project(&project_arc, empty, &settings.features)?.show();

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

                    let install_string = match language {
                        SupportedLanguages::Typescript => "npm install",
                        SupportedLanguages::Python => "pip install .",
                    };

                    Ok(RoutineSuccess::highlight(Message::new(
                        "Get Started".to_string(),
                        format!("\n\n📂 Go to your project directory: \n\t$ cd {}\n\n   Install Dependencies:\n\t$ {} \n\n🛠️ Start dev server: \n\t$ npx @514labs/moose-cli@latest dev\n\n", dir_path.to_string_lossy(), install_string),
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
            check_project_name(&project_arc.name())?;

            let mut controller = RoutineController::new();

            // Remove versions directory so only the relevant versions will be populated
            project_arc.delete_old_versions().map_err(|e| {
                RoutineFailure::error(Message {
                    action: "Build".to_string(),
                    details: format!("Failed to delete old versions: {:?}", e),
                })
            })?;

            copy_old_schema(&project_arc)?.show();

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

            let mut project = load_project()?;
            project.set_is_production_env(false);
            let project_arc = Arc::new(project);

            crate::utilities::capture::capture!(
                ActivityType::DevCommand,
                project_arc.name().clone(),
                &settings
            );

            check_project_name(&project_arc.name())?;
            run_local_infrastructure(&project_arc)?.show();

            routines::start_development_mode(project_arc, &settings.features, metrics)
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
        Commands::Generate(generate) => match &generate.command {
            Some(GenerateCommand::Migrations {}) => {
                info!("Running generate migration command");
                let project = load_project()?;
                let project_arc = Arc::new(project);

                check_project_name(&project_arc.name())?;
                copy_old_schema(&project_arc)?.show();
                generate_migration(&project_arc).await?.show();

                Ok(RoutineSuccess::success(Message::new(
                    "Generated".to_string(),
                    "migrations".to_string(),
                )))
            }
            Some(GenerateCommand::Sdk {
                language,
                destination,
                project_location,
                full_package: packaged,
            }) => {
                let canonical_location = project_location.canonicalize().map_err(|e| {
                    RoutineFailure::error(Message {
                        action: "Generate".to_string(),
                        details: format!("Failed to canonicalize path: {:?}", e),
                    })
                })?;

                let project = Project::load(&canonical_location).map_err(|e| {
                    RoutineFailure::error(Message {
                        action: "Generate".to_string(),
                        details: format!("Failed to load project: {:?}", e),
                    })
                })?;

                with_spinner_async(
                    "Generating SDK",
                    async {
                        let framework_object_versions =
                            load_framework_objects(&project).await.map_err(|e| {
                                RoutineFailure::error(Message {
                                    action: "Generate".to_string(),
                                    details: format!(
                                        "Failed to load initial project state: {:?}",
                                        e
                                    ),
                                })
                            })?;

                        generate_sdk(
                            language,
                            &project,
                            &framework_object_versions,
                            destination,
                            packaged,
                        )
                        .map_err(|e| {
                            RoutineFailure::error(Message {
                                action: "Generate".to_string(),
                                details: format!("Failed to generate SDK: {:?}", e),
                            })
                        })
                    },
                    true,
                )
                .await?;

                Ok(RoutineSuccess::success(Message::new(
                    "Generated".to_string(),
                    "SDK".to_string(),
                )))
            }
            None => Err(RoutineFailure::error(Message {
                action: "Generate".to_string(),
                details: "Please provide a subcommand".to_string(),
            })),
        },
        Commands::Prod {} => {
            info!("Running prod command");
            let mut project = load_project()?;

            project.set_is_production_env(true);
            let project_arc = Arc::new(project);

            crate::utilities::capture::capture!(
                ActivityType::ProdCommand,
                project_arc.name().clone(),
                &settings
            );

            check_project_name(&project_arc.name())?;

            routines::start_production_mode(project_arc, settings.features, metrics)
                .await
                .unwrap();

            Ok(RoutineSuccess::success(Message::new(
                "Ran".to_string(),
                "production infrastructure".to_string(),
            )))
        }
        Commands::Plan {} => {
            info!("Running plan command");
            let project = load_project()?;

            crate::utilities::capture::capture!(
                ActivityType::PlanCommand,
                project.name().clone(),
                &settings
            );

            check_project_name(&project.name())?;
            plan(&project).await.map_err(|e| {
                RoutineFailure::error(Message {
                    action: "Plan".to_string(),
                    details: format!("Failed to plan changes: {:?}", e),
                })
            })?;

            Ok(RoutineSuccess::success(Message::new(
                "Plan".to_string(),
                "Successfuly planned changes to the infrastructure".to_string(),
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

            check_project_name(&project_arc.name())?;

            let new_version = match new_version {
                None => {
                    let current = parse_version(project_arc.cur_version());
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

            bump_version(&project_arc, new_version)
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

            check_project_name(&project_arc.name())?;

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

                    check_project_name(&project_arc.name())?;
                    create_flow_file(&project_arc, init.source.clone(), init.destination.clone())
                        .await
                }
            }
        }
        Commands::Aggregation(aggregation) => {
            info!("Running aggregation command");

            let aggregation_cmd = aggregation.command.as_ref().unwrap();
            match aggregation_cmd {
                AggregationCommands::Init { name } => {
                    let project = load_project()?;
                    let project_arc = Arc::new(project);

                    crate::utilities::capture::capture!(
                        ActivityType::AggregationInitCommand,
                        project_arc.name().clone(),
                        &settings
                    );

                    check_project_name(&project_arc.name())?;
                    create_aggregation_file(&project_arc, name.to_string()).await
                }
            }
        }
        Commands::Consumption(consumption) => {
            info!("Running consumption command");

            let consumption_cmd = consumption.command.as_ref().unwrap();
            match consumption_cmd {
                ConsumptionCommands::Init { name } => {
                    let project = load_project()?;
                    let project_arc = Arc::new(project);

                    crate::utilities::capture::capture!(
                        ActivityType::ConsumptionInitCommand,
                        project_arc.name().clone(),
                        &settings
                    );

                    check_project_name(&project_arc.name())?;
                    create_consumption_file(&project_arc, name.to_string())?.show();

                    Ok(RoutineSuccess::success(Message::new(
                        "Created".to_string(),
                        "Api".to_string(),
                    )))
                }
            }
        }
        Commands::Logs { tail, filter } => {
            info!("Running logs command");

            let project = load_project()?;
            let project_arc = Arc::new(project);

            crate::utilities::capture::capture!(
                ActivityType::LogsCommand,
                project_arc.name().clone(),
                &settings
            );

            check_project_name(&project_arc.name())?;
            let log_file_path = settings.logger.log_file.clone();
            let filter_value = filter.clone().unwrap_or_else(|| "".to_string());

            if *tail {
                follow_logs(log_file_path, filter_value)
            } else {
                show_logs(log_file_path, filter_value)
            }
        }
        Commands::Ps {} => {
            info!("Running ps command");

            let project = load_project()?;
            let project_arc = Arc::new(project);

            crate::utilities::capture::capture!(
                ActivityType::PsCommand,
                project_arc.name().clone(),
                &settings
            );

            show_processes(project_arc)
        }
        Commands::Ls {
            version,
            limit,
            streaming,
        } => {
            info!("Running ls command");

            let project = load_project()?;
            let project_arc = Arc::new(project);

            crate::utilities::capture::capture!(
                ActivityType::LsCommand,
                project_arc.name().clone(),
                &settings
            );

            if *streaming {
                list_streaming(project_arc, limit).await
            } else {
                list_db(project_arc, version, limit).await
            }
        }
    }
}

pub async fn cli_run() {
    let user_directory = setup_user_directory();
    if let Err(e) = user_directory {
        show_message!(
            MessageType::Error,
            Message {
                action: "Init".to_string(),
                details: format!(
                    "Failed to initialize ~/.moose, please check your permissions: {:?}",
                    e
                ),
            }
        );
        exit(1);
    }
    init_config_file().unwrap();

    let config = read_settings().unwrap();
    setup_logging(&config.logger, &config.telemetry.machine_id).expect("Failed to setup logging");

    info!("CLI Configuration loaded and logging setup: {:?}", config);

    let cli = Cli::parse();

    let (metrics, rx) = Metrics::new();
    let arc_metrics = Arc::new(metrics);
    arc_metrics.start_listening_to_metrics(rx).await;

    match top_command_handler(config, &cli.command, arc_metrics).await {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn set_test_temp_dir() {
        let test_dir = "tests/tmp";
        // check that the directory isn't already set to test_dir
        let current_dir = std::env::current_dir().unwrap();
        if current_dir.ends_with(test_dir) {
            return;
        }
        std::env::set_current_dir(test_dir).unwrap();
    }

    fn get_test_project_dir() -> std::path::PathBuf {
        set_test_temp_dir();
        let current_dir = std::env::current_dir().unwrap();
        current_dir.join("test_project")
    }

    fn set_test_project_dir() {
        let test_project_dir = get_test_project_dir();
        std::env::set_current_dir(test_project_dir).unwrap();
    }

    async fn run_project_init(project_type: &str) -> Result<RoutineSuccess, RoutineFailure> {
        let cli = Cli::parse_from([
            "moose",
            "init",
            "test_project",
            project_type,
            "--no-fail-already-exists",
        ]);

        let config = read_settings().unwrap();

        let (metrics, _rx) = Metrics::new();
        let arc_metrics = Arc::new(metrics);

        top_command_handler(config, &cli.command, arc_metrics).await
    }

    #[tokio::test]
    #[ignore] // Ignoring this test until we have a better way of creating temp directories
    async fn cli_python_init() {
        let og_directory = std::env::current_dir().unwrap();
        // Set current working directory to the tmp test directory
        set_test_temp_dir();
        let result = run_project_init("python").await;
        std::env::set_current_dir(og_directory).unwrap();
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore] // Ignoring this test until we have a better way of creating temp directories
    async fn test_project_has_py_data_model() {
        let og_directory = std::env::current_dir().unwrap();

        set_test_temp_dir();
        let _ = run_project_init("python").await.unwrap();
        set_test_project_dir();

        let project = Project::load_from_current_dir().unwrap();

        let data_model_path = project.app_dir().join("datamodels");

        // Make sure all the data models are .py files
        let data_model_files = std::fs::read_dir(data_model_path).unwrap();

        std::env::set_current_dir(og_directory).unwrap();
        for file in data_model_files {
            let file = file.unwrap();
            let file_name = file.file_name();
            let file_name = file_name.to_str().unwrap();
            assert!(file_name.ends_with(".py"));
        }
    }
}

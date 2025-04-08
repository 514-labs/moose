#[macro_use]
pub(crate) mod display;

mod commands;
pub mod local_webserver;
mod logger;
mod routines;
pub mod settings;
mod watcher;
use super::metrics::Metrics;
use crate::cli::routines::block::create_block_file;
use crate::cli::routines::consumption::create_consumption_file;
use crate::utilities::docker::DockerClient;
use crate::utilities::machine_id::get_or_create_machine_id;
use clap::Parser;
use commands::{
    BlockCommands, Commands, ConsumptionCommands, DataModelCommands, FunctionCommands,
    GenerateCommand, WorkflowCommands,
};
use config::ConfigError;
use display::{with_spinner, with_spinner_async};
use home::home_dir;
use log::{debug, info};
use logger::setup_logging;
use regex::Regex;
use routines::auth::generate_hash_token;
use routines::build::build_package;
use routines::clean::clean_project;
use routines::datamodel::parse_and_generate;
use routines::docker_packager::{build_dockerfile, create_dockerfile};
use routines::ls::{list_db, list_streaming};
use routines::metrics_console::run_console;
use routines::ps::show_processes;
use routines::scripts::{
    get_workflow_status, init_workflow, list_workflows, pause_workflow, run_workflow,
    terminate_workflow, unpause_workflow,
};

use settings::{read_settings, Settings};
use std::path::Path;
use std::process::exit;
use std::sync::Arc;

use crate::cli::routines::logs::{follow_logs, show_logs};
use crate::cli::routines::peek::peek;
use crate::cli::routines::setup_redis_client;
use crate::cli::routines::streaming::create_streaming_function_file;
use crate::cli::routines::templates;
use crate::cli::routines::{RoutineFailure, RoutineSuccess};
use crate::cli::settings::user_directory;
use crate::cli::{
    display::{Message, MessageType},
    routines::dev::run_local_infrastructure,
    settings::{init_config_file, setup_user_directory},
};
use crate::framework::bulk_import::import_csv_file;
use crate::framework::core::check::check_system_reqs;
use crate::framework::core::infrastructure::api_endpoint::APIType;
use crate::framework::core::infrastructure_map::InfrastructureMap;
use crate::framework::core::primitive_map::PrimitiveMap;
use crate::framework::languages::SupportedLanguages;
use crate::framework::sdk::ingest::generate_sdk;
use crate::metrics::TelemetryMetadata;
use crate::project::Project;
use crate::utilities::capture::{wait_for_usage_capture, ActivityType};
use crate::utilities::constants::{CLI_VERSION, PROJECT_NAME_ALLOW_PATTERN};
use crate::utilities::git::is_git_repo;

use anyhow::Result;

#[derive(Parser)]
#[command(author, version, about, long_about = None, arg_required_else_help(true), next_display_order = None)]
struct Cli {
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

fn maybe_create_git_repo(dir_path: &Path, project_arc: Arc<Project>) {
    let is_git_repo = is_git_repo(dir_path).expect("Failed to check if directory is a git repo");

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
                details: format!("Created project at {} 🚀", dir_path.to_string_lossy()),
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
}

async fn top_command_handler(
    settings: Settings,
    commands: &Commands,
    machine_id: String,
) -> Result<RoutineSuccess, RoutineFailure> {
    match commands {
        Commands::Init {
            name,
            location,
            template,
            no_fail_already_exists,
        } => {
            info!(
                "Running init command with name: {}, location: {:?}, template: {:?}",
                name, location, template
            );

            let capture_handle = crate::utilities::capture::capture_usage(
                ActivityType::InitTemplateCommand,
                Some(name.to_string()),
                &settings,
                machine_id.clone(),
            );

            check_project_name(name)?;

            let template_config =
                templates::get_template_config(&template.to_lowercase(), CLI_VERSION).await?;

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

            let language = match template_config.language.as_str() {
                "typescript" => SupportedLanguages::Typescript,
                "python" => SupportedLanguages::Python,
                _ => SupportedLanguages::Typescript,
            };

            templates::generate_template(&template.to_lowercase(), CLI_VERSION, dir_path).await?;
            let project = Project::new(dir_path, name.clone(), language);
            let project_arc = Arc::new(project);

            // Update project configuration based on language
            match language {
                SupportedLanguages::Typescript => {
                    let package_json_path = dir_path.join("package.json");
                    if package_json_path.exists() {
                        let mut package_json: serde_json::Value = serde_json::from_str(
                            &std::fs::read_to_string(&package_json_path).map_err(|e| {
                                RoutineFailure::error(Message {
                                    action: "Init".to_string(),
                                    details: format!("Failed to read package.json: {}", e),
                                })
                            })?,
                        )
                        .map_err(|e| {
                            RoutineFailure::error(Message {
                                action: "Init".to_string(),
                                details: format!("Failed to parse package.json: {}", e),
                            })
                        })?;

                        if let Some(obj) = package_json.as_object_mut() {
                            obj.insert("name".to_string(), serde_json::Value::String(name.clone()));
                            std::fs::write(
                                &package_json_path,
                                serde_json::to_string_pretty(&package_json).map_err(|e| {
                                    RoutineFailure::error(Message {
                                        action: "Init".to_string(),
                                        details: format!("Failed to serialize package.json: {}", e),
                                    })
                                })?,
                            )
                            .map_err(|e| {
                                RoutineFailure::error(Message {
                                    action: "Init".to_string(),
                                    details: format!("Failed to write package.json: {}", e),
                                })
                            })?;
                        }
                    }
                }
                SupportedLanguages::Python => {
                    let setup_py_path = dir_path.join("setup.py");
                    if setup_py_path.exists() {
                        let setup_py_content =
                            std::fs::read_to_string(&setup_py_path).map_err(|e| {
                                RoutineFailure::error(Message {
                                    action: "Init".to_string(),
                                    details: format!("Failed to read setup.py: {}", e),
                                })
                            })?;

                        // Replace the name in setup.py
                        let name_pattern = Regex::new(r"name='[^']*'").map_err(|e| {
                            RoutineFailure::error(Message {
                                action: "Init".to_string(),
                                details: format!("Failed to create regex pattern: {}", e),
                            })
                        })?;
                        let new_setup_py =
                            name_pattern.replace(&setup_py_content, &format!("name='{}'", name));

                        std::fs::write(&setup_py_path, new_setup_py.as_bytes()).map_err(|e| {
                            RoutineFailure::error(Message {
                                action: "Init".to_string(),
                                details: format!("Failed to write setup.py: {}", e),
                            })
                        })?;
                    }
                }
            }

            maybe_create_git_repo(dir_path, project_arc);
            wait_for_usage_capture(capture_handle).await;

            let post_install_message = template_config
                .post_install_print
                .replace("{project_dir}", &dir_path.to_string_lossy());

            Ok(RoutineSuccess::highlight(Message::new(
                "Get Started".to_string(),
                format!("\n\n{}", post_install_message),
            )))
        }
        // This command is used to check the project for errors that are not related to runtime
        // For example, it checks that the project is valid and that all the primitives are loaded
        // It is used in the build process to ensure that the project is valid while building docker images
        Commands::Check { write_infra_map } => {
            info!(
                "Running check command with write_infra_map: {}",
                *write_infra_map
            );
            let project_arc = Arc::new(load_project()?);

            let capture_handle = crate::utilities::capture::capture_usage(
                ActivityType::CheckCommand,
                Some(project_arc.name()),
                &settings,
                machine_id.clone(),
            );

            check_project_name(&project_arc.name())?;

            check_system_reqs(&project_arc.language_project_config)
                .await
                .map_err(|e| {
                    RoutineFailure::error(Message {
                        action: "System".to_string(),
                        details: format!("Failed to validate system requirements: {:?}", e),
                    })
                })?;

            let infra_map = if project_arc.features.data_model_v2 {
                debug!("Loading InfrastructureMap from user code (DMV2)");
                InfrastructureMap::load_from_user_code(&project_arc)
                    .await
                    .map_err(|e| {
                        RoutineFailure::error(Message {
                            action: "Build".to_string(),
                            details: format!("Failed to load InfrastructureMap: {:?}", e),
                        })
                    })?
            } else {
                debug!("Loading InfrastructureMap from primitives");
                let primitive_map = PrimitiveMap::load(&project_arc).await.map_err(|e| {
                    RoutineFailure::error(Message {
                        action: "Build".to_string(),
                        details: format!("Failed to load Primitives: {:?}", e),
                    })
                })?;

                InfrastructureMap::new(&project_arc, primitive_map)
            };

            if *write_infra_map {
                let json_path = project_arc
                    .internal_dir()
                    .map_err(|e| {
                        RoutineFailure::new(
                            Message::new(
                                "Failed".to_string(),
                                "to create .moose directory".to_string(),
                            ),
                            e,
                        )
                    })?
                    .join("infrastructure_map.json");

                infra_map.save_to_json(&json_path).map_err(|e| {
                    RoutineFailure::new(
                        Message::new(
                            "Failed".to_string(),
                            "to save InfrastructureMap as JSON".to_string(),
                        ),
                        e,
                    )
                })?;
            }

            wait_for_usage_capture(capture_handle).await;

            Ok(RoutineSuccess::success(Message::new(
                "Checked".to_string(),
                "No Errors found".to_string(),
            )))
        }
        Commands::Build {
            docker,
            amd64,
            arm64,
        } => {
            info!("Running build command");
            let project_arc = Arc::new(load_project()?);

            check_project_name(&project_arc.name())?;

            // docker flag is true then build docker images
            if *docker {
                let capture_handle = crate::utilities::capture::capture_usage(
                    ActivityType::DockerCommand,
                    Some(project_arc.name()),
                    &settings,
                    machine_id.clone(),
                );

                let docker_client = DockerClient::new(&settings);
                create_dockerfile(&project_arc, &docker_client)?.show();
                let _: RoutineSuccess =
                    build_dockerfile(&project_arc, &docker_client, *amd64, *arm64)?;

                wait_for_usage_capture(capture_handle).await;

                Ok(RoutineSuccess::success(Message::new(
                    "Built".to_string(),
                    "Docker image(s)".to_string(),
                )))
            } else {
                let capture_handle = crate::utilities::capture::capture_usage(
                    ActivityType::BuildCommand,
                    Some(project_arc.name()),
                    &settings,
                    machine_id.clone(),
                );

                // Use the new build_package function instead of Docker build
                let package_path = with_spinner(
                    "Bundling deployment package",
                    || {
                        build_package(&project_arc).map_err(|e| {
                            RoutineFailure::error(Message {
                                action: "Build".to_string(),
                                details: format!("Failed to build package: {:?}", e),
                            })
                        })
                    },
                    !project_arc.is_production,
                )?;

                wait_for_usage_capture(capture_handle).await;

                Ok(RoutineSuccess::success(Message::new(
                    "Built".to_string(),
                    format!("Package available at {}", package_path.display()),
                )))
            }
        }
        Commands::Dev {} => {
            info!("Running dev command");

            let mut project = load_project()?;
            project.set_is_production_env(false);
            let project_arc = Arc::new(project);

            let capture_handle = crate::utilities::capture::capture_usage(
                ActivityType::DevCommand,
                Some(project_arc.name()),
                &settings,
                machine_id.clone(),
            );

            let docker_client = DockerClient::new(&settings);

            check_project_name(&project_arc.name())?;
            run_local_infrastructure(&project_arc, &settings, &docker_client)?.show();

            let redis_client = setup_redis_client(project_arc.clone()).await.map_err(|e| {
                RoutineFailure::error(Message {
                    action: "Dev".to_string(),
                    details: format!("Failed to setup redis client: {:?}", e),
                })
            })?;

            let (metrics, rx_events) = Metrics::new(
                TelemetryMetadata {
                    anonymous_telemetry_enabled: settings.telemetry.enabled,
                    machine_id: machine_id.clone(),
                    metric_labels: settings.metric.labels.clone(),
                    is_moose_developer: settings.telemetry.is_moose_developer,
                    is_production: project_arc.is_production,
                    project_name: project_arc.name().to_string(),
                    export_metrics: settings.telemetry.export_metrics,
                    metric_endpoints: settings.metric.endpoints.clone(),
                },
                if settings.features.metrics_v2 {
                    Some(redis_client.clone())
                } else {
                    None
                },
            );

            let arc_metrics = Arc::new(metrics);
            arc_metrics.start_listening_to_metrics(rx_events).await;

            routines::start_development_mode(project_arc, arc_metrics, redis_client, &settings)
                .await
                .map_err(|e| {
                    RoutineFailure::error(Message {
                        action: "Dev".to_string(),
                        details: format!("Failed to start development mode: {:?}", e),
                    })
                })?;

            wait_for_usage_capture(capture_handle).await;

            Ok(RoutineSuccess::success(Message::new(
                "Ran".to_string(),
                "local infrastructure".to_string(),
            )))
        }
        Commands::Generate(generate) => match &generate.command {
            Some(GenerateCommand::HashToken {}) => {
                info!("Running generate hash token command");
                let project = load_project()?;
                let project_arc = Arc::new(project);

                let capture_handle = crate::utilities::capture::capture_usage(
                    ActivityType::GenerateHashCommand,
                    Some(project_arc.name()),
                    &settings,
                    machine_id.clone(),
                );

                check_project_name(&project_arc.name())?;
                generate_hash_token();

                wait_for_usage_capture(capture_handle).await;

                Ok(RoutineSuccess::success(Message::new(
                    "Token".to_string(),
                    "Success".to_string(),
                )))
            }
            Some(GenerateCommand::Sdk {
                language,
                destination,
                project_location,
                full_package: packaged,
                overwrite,
            }) => {
                let canonical_location = project_location.canonicalize().map_err(|e| {
                    RoutineFailure::error(Message {
                        action: "Generate".to_string(),
                        details: format!("Failed to canonicalize path: {:?}", e),
                    })
                })?;

                if destination.exists() && destination.is_dir() {
                    // Check if the directory contains any files or subdirectories
                    let is_empty = std::fs::read_dir(destination)
                        .map(|mut dir| dir.next().is_none())
                        .unwrap_or(false);

                    if !is_empty && !overwrite {
                        return Err(RoutineFailure::error(Message {
                            action: "Generate".to_string(),
                            details: format!(
                                "Directory '{}' is not empty, and --overwrite flag is not set.",
                                destination.display()
                            ),
                        }));
                    }
                }

                let project = Project::load(&canonical_location).map_err(|e| {
                    RoutineFailure::error(Message {
                        action: "Generate".to_string(),
                        details: format!("Failed to load project: {:?}", e),
                    })
                })?;

                let capture_handle = crate::utilities::capture::capture_usage(
                    ActivityType::GenerateSDKCommand,
                    Some(project.name()),
                    &settings,
                    machine_id.clone(),
                );

                with_spinner_async(
                    "Generating SDK",
                    async {
                        let primitive_map = PrimitiveMap::load(&project).await.map_err(|e| {
                            RoutineFailure::error(Message {
                                action: "Generate".to_string(),
                                details: format!("Failed to load initial project state: {:?}", e),
                            })
                        })?;

                        generate_sdk(language, &project, &primitive_map, destination, packaged)
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

                wait_for_usage_capture(capture_handle).await;

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
            info!("Moose Version: {}", CLI_VERSION);

            let mut project = load_project()?;

            project.set_is_production_env(true);
            let project_arc = Arc::new(project);

            check_project_name(&project_arc.name())?;

            let redis_client = setup_redis_client(project_arc.clone()).await.map_err(|e| {
                RoutineFailure::error(Message {
                    action: "Prod".to_string(),
                    details: format!("Failed to setup redis client: {:?}", e),
                })
            })?;

            let (metrics, rx_events) = Metrics::new(
                TelemetryMetadata {
                    anonymous_telemetry_enabled: settings.telemetry.enabled,
                    machine_id: machine_id.clone(),
                    metric_labels: settings.metric.labels.clone(),
                    is_moose_developer: settings.telemetry.is_moose_developer,
                    is_production: project_arc.is_production,
                    project_name: project_arc.name().to_string(),
                    export_metrics: settings.telemetry.export_metrics,
                    metric_endpoints: settings.metric.endpoints.clone(),
                },
                if settings.features.metrics_v2 {
                    Some(redis_client.clone())
                } else {
                    None
                },
            );

            let arc_metrics = Arc::new(metrics);
            arc_metrics.start_listening_to_metrics(rx_events).await;

            let capture_handle = crate::utilities::capture::capture_usage(
                ActivityType::ProdCommand,
                Some(project_arc.name()),
                &settings,
                machine_id.clone(),
            );

            routines::start_production_mode(&settings, project_arc, arc_metrics, redis_client)
                .await
                .map_err(|e| {
                    RoutineFailure::error(Message {
                        action: "Prod".to_string(),
                        details: format!("Failed to start production mode: {:?}", e),
                    })
                })?;

            wait_for_usage_capture(capture_handle).await;

            Ok(RoutineSuccess::success(Message::new(
                "Ran".to_string(),
                "production infrastructure".to_string(),
            )))
        }
        Commands::Plan { url, token } => {
            info!("Running plan command");
            let project = load_project()?;

            let capture_handle = crate::utilities::capture::capture_usage(
                ActivityType::PlanCommand,
                Some(project.name()),
                &settings,
                machine_id.clone(),
            );

            check_project_name(&project.name())?;

            let result = routines::remote_plan(&project, url, token).await;

            result.map_err(|e| {
                RoutineFailure::error(Message {
                    action: "Plan".to_string(),
                    details: format!("Failed to plan changes: {:?}", e),
                })
            })?;

            wait_for_usage_capture(capture_handle).await;

            Ok(RoutineSuccess::success(Message::new(
                "Plan".to_string(),
                "Successfully planned changes to the infrastructure".to_string(),
            )))
        }
        Commands::Clean {} => {
            let project = load_project()?;
            let project_arc = Arc::new(project);

            let capture_handle = crate::utilities::capture::capture_usage(
                ActivityType::CleanCommand,
                Some(project_arc.name()),
                &settings,
                machine_id.clone(),
            );

            check_project_name(&project_arc.name())?;

            let docker_client = DockerClient::new(&settings);
            let _ = clean_project(&project_arc, &docker_client)?;

            wait_for_usage_capture(capture_handle).await;

            Ok(RoutineSuccess::success(Message::new(
                "Cleaned".to_string(),
                "Project".to_string(),
            )))
        }
        Commands::Function(function_args) => {
            info!("Running function command");

            let func_cmd = function_args.command.as_ref().unwrap();
            match func_cmd {
                FunctionCommands::Init(init) => {
                    let project = load_project()?;
                    let project_arc = Arc::new(project);

                    let capture_handle = crate::utilities::capture::capture_usage(
                        ActivityType::FuncInitCommand,
                        Some(project_arc.name()),
                        &settings,
                        machine_id.clone(),
                    );

                    check_project_name(&project_arc.name())?;
                    let result = create_streaming_function_file(
                        &project_arc,
                        init.source.clone(),
                        init.destination.clone(),
                    )
                    .await;

                    wait_for_usage_capture(capture_handle).await;

                    result
                }
            }
        }
        Commands::DataModel(data_model_args) => {
            let dm_cmd = data_model_args.command.as_ref().unwrap();
            let project = load_project()?;
            match dm_cmd {
                DataModelCommands::Init(args) => {
                    debug!("Running datamodel init command");

                    let capture_handle = crate::utilities::capture::capture_usage(
                        ActivityType::DataModelInitCommand,
                        Some(project.name()),
                        &settings,
                        machine_id.clone(),
                    );

                    let file = std::fs::read_to_string(&args.sample).map_err(|e| {
                        RoutineFailure::new(
                            Message {
                                action: "Failed".to_string(),
                                details: "to read samples".to_string(),
                            },
                            e,
                        )
                    })?;
                    let interface = parse_and_generate(&args.name, file, project.language);
                    std::fs::write(
                        project.data_models_dir().join(format!(
                            "{}.{}",
                            args.name,
                            project.language.extension()
                        )),
                        interface.as_bytes(),
                    )
                    .map_err(|e| {
                        RoutineFailure::new(
                            Message {
                                action: "Failed".to_string(),
                                details: "to write data model".to_string(),
                            },
                            e,
                        )
                    })?;

                    wait_for_usage_capture(capture_handle).await;

                    Ok(RoutineSuccess::success(Message::new(
                        "DataModel".to_string(),
                        "Initialized".to_string(),
                    )))
                }
            }
        }
        Commands::Block(block) => {
            info!("Running block command");

            let block_cmd = block.command.as_ref().unwrap();
            match block_cmd {
                BlockCommands::Init { name } => {
                    let project = load_project()?;
                    let project_arc = Arc::new(project);

                    let capture_handle = crate::utilities::capture::capture_usage(
                        ActivityType::BlockInitCommand,
                        Some(project_arc.name()),
                        &settings,
                        machine_id.clone(),
                    );

                    check_project_name(&project_arc.name())?;
                    let result = create_block_file(&project_arc, name.to_string()).await;

                    wait_for_usage_capture(capture_handle).await;

                    result
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

                    let capture_handle = crate::utilities::capture::capture_usage(
                        ActivityType::ConsumptionInitCommand,
                        Some(project_arc.name()),
                        &settings,
                        machine_id.clone(),
                    );

                    check_project_name(&project_arc.name())?;
                    create_consumption_file(&project_arc, name.to_string())?.show();

                    wait_for_usage_capture(capture_handle).await;

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

            let capture_handle = crate::utilities::capture::capture_usage(
                ActivityType::LogsCommand,
                Some(project.name()),
                &settings,
                machine_id.clone(),
            );

            check_project_name(&project.name())?;

            let log_file_path = chrono::Local::now()
                .format(&settings.logger.log_file_date_format)
                .to_string();

            let log_file_path = user_directory()
                .join(log_file_path)
                .to_str()
                .unwrap()
                .to_string();

            let filter_value = filter.clone().unwrap_or_else(|| "".to_string());

            let result = if *tail {
                follow_logs(log_file_path, filter_value)
            } else {
                show_logs(log_file_path, filter_value)
            };

            wait_for_usage_capture(capture_handle).await;

            result
        }
        Commands::Ps {} => {
            info!("Running ps command");

            let project = load_project()?;
            let project_arc = Arc::new(project);

            let capture_handle = crate::utilities::capture::capture_usage(
                ActivityType::PsCommand,
                Some(project_arc.name()),
                &settings,
                machine_id.clone(),
            );

            let result = show_processes(project_arc);

            wait_for_usage_capture(capture_handle).await;

            result
        }
        Commands::Ls {
            version,
            limit,
            streaming,
        } => {
            info!("Running ls command");

            let project = load_project()?;
            let project_arc = Arc::new(project);

            let capture_handle = crate::utilities::capture::capture_usage(
                ActivityType::LsCommand,
                Some(project_arc.name()),
                &settings,
                machine_id.clone(),
            );

            let res = if *streaming {
                list_streaming(project_arc, limit).await
            } else {
                list_db(project_arc, version, limit).await
            };

            wait_for_usage_capture(capture_handle).await;

            res
        }

        Commands::Metrics {} => {
            let capture_handle = crate::utilities::capture::capture_usage(
                ActivityType::MetricsCommand,
                None,
                &settings,
                machine_id.clone(),
            );

            let result = run_console().await;

            wait_for_usage_capture(capture_handle).await;

            result
        }

        Commands::Import {
            data_model_name,
            file,
            format,
            destination,
            version,
        } => {
            let project = load_project()?;

            let capture_handle = crate::utilities::capture::capture_usage(
                ActivityType::ImportCommand,
                Some(project.name()),
                &settings,
                machine_id.clone(),
            );

            let primitive_map = crate::framework::core::primitive_map::PrimitiveMap::load(&project)
                .await
                .map_err(|e| {
                    RoutineFailure::error(Message {
                        action: "Import".to_string(),
                        details: format!("Failed to load primitive map: {:?}", e),
                    })
                })?;

            let infrastructure_map =
                crate::framework::core::infrastructure_map::InfrastructureMap::new(
                    &project,
                    primitive_map,
                );

            let api_endpoint = infrastructure_map
                .api_endpoints
                .values()
                .find(|endpoint| {
                    endpoint.name == *data_model_name
                        && matches!(endpoint.api_type, APIType::INGRESS { .. })
                        && match version {
                            None => true,
                            Some(v) => endpoint.version.as_str() == v,
                        }
                })
                .ok_or_else(|| {
                    RoutineFailure::error(Message {
                        action: "Import".to_string(),
                        details: format!(
                            "Could not find ingress API endpoint for data model {}",
                            data_model_name
                        ),
                    })
                })?;

            let format = format
                .as_deref()
                .or_else(|| file.extension().and_then(|ext| ext.to_str()))
                .ok_or(RoutineFailure::error(Message::new(
                    "Format".to_string(),
                    "missing".to_string(),
                )))?;

            if format == "csv" {
                import_csv_file(&api_endpoint.api_type, file, destination)
                    .await
                    .map_err(|e| {
                        RoutineFailure::new(
                            Message::new("Import".to_string(), "failed".to_string()),
                            e,
                        )
                    })?;
            } else {
                return Err(RoutineFailure::error(Message::new(
                    "Unknown Format".to_string(),
                    "only 'csv' is supported currently".to_string(),
                )));
            }

            wait_for_usage_capture(capture_handle).await;

            Ok(RoutineSuccess::success(Message::new(
                "Imported".to_string(),
                "".to_string(),
            )))
        }
        Commands::Peek {
            data_model_name,
            limit,
            file,
            topic,
        } => {
            info!("Running peek command");

            let project = load_project()?;
            let project_arc = Arc::new(project);

            let capture_handle = crate::utilities::capture::capture_usage(
                ActivityType::PeekCommand,
                Some(project_arc.name()),
                &settings,
                machine_id.clone(),
            );

            let result = peek(project_arc, data_model_name, *limit, file.clone(), *topic).await;

            wait_for_usage_capture(capture_handle).await;

            result
        }
        Commands::Workflow(workflow_args) => {
            let project = load_project()?;

            if !(settings.features.scripts || project.features.workflows) {
                return Err(RoutineFailure::error(Message {
                    action: "Workflow".to_string(),
                    details: "Feature not enabled, to turn on go to moose.config.toml and set 'workflows' to true under the 'features' section".to_string(),
                }));
            }

            let activity_type = match &workflow_args.command {
                Some(WorkflowCommands::Init { .. }) => ActivityType::WorkflowInitCommand,
                Some(WorkflowCommands::Run { .. }) => ActivityType::WorkflowRunCommand,
                Some(WorkflowCommands::List { .. }) => ActivityType::WorkflowListCommand,
                Some(WorkflowCommands::Resume { .. }) => ActivityType::WorkflowResumeCommand,
                Some(WorkflowCommands::Terminate { .. }) => ActivityType::WorkflowTerminateCommand,
                Some(WorkflowCommands::Pause { .. }) => ActivityType::WorkflowPauseCommand,
                Some(WorkflowCommands::Unpause { .. }) => ActivityType::WorkflowUnpauseCommand,
                Some(WorkflowCommands::Status { .. }) => ActivityType::WorkflowStatusCommand,
                None => ActivityType::WorkflowCommand,
            };

            let capture_handle = crate::utilities::capture::capture_usage(
                activity_type,
                Some(project.name()),
                &settings,
                machine_id.clone(),
            );

            let result = match &workflow_args.command {
                Some(WorkflowCommands::Init { name, tasks, task }) => {
                    init_workflow(&project, name, tasks.clone(), task.clone()).await
                }
                Some(WorkflowCommands::Run { name, input }) => {
                    run_workflow(&project, name, input.clone()).await
                }
                Some(WorkflowCommands::List { status, limit }) => {
                    list_workflows(&project, status.clone(), *limit).await
                }
                Some(WorkflowCommands::Resume { .. }) => Err(RoutineFailure::error(Message {
                    action: "Workflow Resume".to_string(),
                    details: "Not implemented yet".to_string(),
                })),
                Some(WorkflowCommands::Terminate { name }) => {
                    terminate_workflow(&project, name).await
                }
                Some(WorkflowCommands::Pause { name }) => pause_workflow(&project, name).await,
                Some(WorkflowCommands::Unpause { name }) => unpause_workflow(&project, name).await,
                Some(WorkflowCommands::Status {
                    name,
                    id,
                    verbose,
                    json,
                }) => get_workflow_status(&project, name, id.clone(), *verbose, *json).await,
                None => Err(RoutineFailure::error(Message {
                    action: "Workflow".to_string(),
                    details: "No subcommand provided".to_string(),
                })),
            };

            wait_for_usage_capture(capture_handle).await;

            result
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
    let machine_id = get_or_create_machine_id();
    setup_logging(&config.logger, &machine_id).expect("Failed to setup logging");

    info!("CLI Configuration loaded and logging setup: {:?}", config);

    let cli = Cli::parse();
    let cli_result = top_command_handler(config, &cli.command, machine_id).await;
    match cli_result {
        Ok(s) => {
            show_message!(s.message_type, s.message);
            exit(0);
        }
        Err(e) => {
            show_message!(e.message_type, e.message);
            if let Some(err) = e.error {
                eprintln!("{:?}", err)
            }
            exit(1);
        }
    }
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
        let machine_id = get_or_create_machine_id();

        top_command_handler(config, &cli.command, machine_id).await
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

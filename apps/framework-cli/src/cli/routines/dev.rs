use super::{
    validate::{
        validate_clickhouse_run, validate_redpanda_cluster, validate_redpanda_run,
        validate_temporal_run,
    },
    RoutineFailure, RoutineSuccess,
};
use crate::cli::{display::with_spinner, settings::Settings};
use crate::utilities::constants::CLI_PROJECT_INTERNAL_DIR;
use crate::{cli::display::Message, project::Project};
use crate::{cli::routines::util::ensure_docker_running, utilities::docker::DockerClient};
use lazy_static::lazy_static;

pub fn run_local_infrastructure(
    project: &Project,
    settings: &Settings,
    docker_client: &DockerClient,
) -> Result<RoutineSuccess, RoutineFailure> {
    create_docker_compose_file(project, settings, docker_client)?.show();

    ensure_docker_running(docker_client)?;
    run_containers(project, docker_client)?.show();

    validate_clickhouse_run(project, docker_client)?.show();
    if project.features.streaming_engine {
        validate_redpanda_run(project, docker_client)?.show();
        validate_redpanda_cluster(project.name(), docker_client)?.show();
    }
    if settings.features.scripts || project.features.workflows {
        validate_temporal_run(project, docker_client)?.show();
    }

    Ok(RoutineSuccess::success(Message::new(
        "Successfully".to_string(),
        "ran local infrastructure".to_string(),
    )))
}

lazy_static! {
    static ref FAILED_TO_CREATE_INTERNAL_DIR: Message = Message::new(
        "Failed".to_string(),
        format!(
            "to create {} directory. Check permissions or contact us`",
            CLI_PROJECT_INTERNAL_DIR
        ),
    );
}

pub fn run_containers(
    project: &Project,
    docker_client: &DockerClient,
) -> Result<RoutineSuccess, RoutineFailure> {
    let docker_compose_res = with_spinner(
        "Starting local infrastructure",
        || docker_client.start_containers(project),
        !project.is_production,
    );

    match docker_compose_res {
        Ok(_) => Ok(RoutineSuccess::success(Message::new(
            "Successfully".to_string(),
            "started containers".to_string(),
        ))),
        Err(err) => Err(RoutineFailure::new(
            Message::new("Failed".to_string(), "to start containers".to_string()),
            err,
        )),
    }
}

pub fn create_docker_compose_file(
    project: &Project,
    settings: &Settings,
    docker_client: &DockerClient,
) -> Result<RoutineSuccess, RoutineFailure> {
    let output = docker_client.create_compose_file(project, settings);

    match output {
        Ok(_) => Ok(RoutineSuccess::success(Message::new(
            "Created".to_string(),
            "docker compose file".to_string(),
        ))),
        Err(err) => Err(RoutineFailure::new(
            Message::new(
                "Failed".to_string(),
                "to create docker compose file".to_string(),
            ),
            err,
        )),
    }
}

use lazy_static::lazy_static;
use std::fs;

use crate::cli::display::with_spinner;
use crate::cli::routines::util::ensure_docker_running;
use crate::framework::languages::create_models_dir;
use crate::framework::typescript::create_typescript_models_dir;
use crate::utilities::constants::CLI_PROJECT_INTERNAL_DIR;
use crate::utilities::docker;
use crate::utilities::git::dump_old_version_schema;
use crate::{cli::display::Message, project::Project};
use log::debug;

use super::{
    validate::{
        validate_clickhouse_run, validate_console_run, validate_redpanda_cluster,
        validate_redpanda_run,
    },
    RoutineFailure, RoutineSuccess,
};

pub fn run_local_infrastructure(project: &Project) -> Result<RoutineSuccess, RoutineFailure> {
    create_deno_files(project)?;
    create_models_volume(project)?;
    create_docker_compose_file(project)?;

    copy_old_schema(project)?;

    ensure_docker_running()?;
    run_containers(project)?;

    validate_clickhouse_run()?;
    validate_redpanda_run()?;
    validate_console_run()?;
    validate_redpanda_cluster(project.name())?;

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

pub fn run_containers(project: &Project) -> Result<RoutineSuccess, RoutineFailure> {
    let docker_compose_res = with_spinner("Starting local infrastructure", || {
        docker::start_containers(project)
    });

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

pub fn copy_old_schema(project: &Project) -> Result<RoutineSuccess, RoutineFailure> {
    for (version, commit_hash) in project.supported_old_versions.iter() {
        let dest = project.old_version_location(version).unwrap();

        fs::create_dir_all(dest.clone()).map_err(|err| {
            RoutineFailure::new(
                Message::new("Failed".to_string(), "to create directory".to_string()),
                err,
            )
        })?;

        dump_old_version_schema(project, commit_hash.clone(), &dest).map_err(|git_err| {
            debug!("<DCM> Failed to retrieve old schema: {}", git_err);
            RoutineFailure::new(
                Message::new("Failed".to_string(), "to retrieve old schema".to_string()),
                git_err,
            )
        })?;
    }

    Ok(RoutineSuccess::success(Message::new(
        "Loaded".to_string(),
        "old schemas".to_string(),
    )))
}

pub fn create_deno_files(project: &Project) -> Result<RoutineSuccess, RoutineFailure> {
    project.create_deno_files().map_err(|err| {
        RoutineFailure::new(Message::new("Failed".to_string(), "".to_string()), err)
    })?;

    Ok(RoutineSuccess::success(Message::new(
        "Created".to_string(),
        "deno files".to_string(),
    )))
}

pub fn create_models_volume(project: &Project) -> Result<RoutineSuccess, RoutineFailure> {
    create_models_dir(project).map_err(|err| {
        RoutineFailure::new(
            Message::new(
                "Failed".to_string(),
                format!("to create models volume in {}", err),
            ),
            err,
        )
    })?;

    create_typescript_models_dir(project).map_err(|err| {
        RoutineFailure::new(
            Message::new(
                "Failed".to_string(),
                format!("to create models volume in {}", err),
            ),
            err,
        )
    })?;

    Ok(RoutineSuccess::success(Message::new(
        "Created".to_string(),
        "Models volume".to_string(),
    )))
}

pub fn create_docker_compose_file(project: &Project) -> Result<RoutineSuccess, RoutineFailure> {
    let output = docker::create_compose_file(project);

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

use super::{
    validate::{validate_clickhouse_run, validate_redpanda_cluster, validate_redpanda_run},
    RoutineFailure, RoutineSuccess,
};
use crate::cli::display::with_spinner;
use crate::cli::routines::util::ensure_docker_running;
use crate::framework::languages::SupportedLanguages;
use crate::framework::python;
use crate::utilities::constants::CLI_PROJECT_INTERNAL_DIR;
use crate::utilities::docker;
use crate::utilities::git::dump_old_version_schema;
use crate::{cli::display::Message, project::Project};
use lazy_static::lazy_static;
use log::debug;
use std::fs;
use std::io::ErrorKind;

pub fn run_local_infrastructure(project: &Project) -> Result<RoutineSuccess, RoutineFailure> {
    create_docker_compose_file(project)?.show();

    copy_old_schema(project)?.show();

    ensure_docker_running()?;
    run_containers(project)?.show();

    validate_clickhouse_run(project)?.show();
    validate_redpanda_run(project)?.show();
    validate_redpanda_cluster(project.name())?.show();

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
    let docker_compose_res = with_spinner(
        "Starting local infrastructure",
        || docker::start_containers(project),
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

        if project.language == SupportedLanguages::Python {
            std::os::unix::fs::symlink(
                dest,
                project
                    .old_version_location(&python::version_to_identifier(version))
                    .unwrap(),
            )
            .or_else(|err| {
                if err.kind() == ErrorKind::AlreadyExists {
                    Ok(())
                } else {
                    Err(RoutineFailure::new(
                        Message::new(
                            "Failed".to_string(),
                            format!("to create symlink for version {}", version),
                        ),
                        err,
                    ))
                }
            })?;
        }
    }

    Ok(RoutineSuccess::success(Message::new(
        "Loaded".to_string(),
        "old schemas".to_string(),
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

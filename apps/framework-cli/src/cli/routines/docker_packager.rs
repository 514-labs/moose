use super::{RoutineFailure, RoutineSuccess};
use crate::cli::display::with_spinner;
use crate::cli::routines::util::ensure_docker_running;
use crate::framework::languages::SupportedLanguages;
use crate::utilities::constants::{
    APP_DIR, CLI_INTERNAL_VERSIONS_DIR, OLD_PROJECT_CONFIG_FILE, PACKAGE_JSON, PROJECT_CONFIG_FILE,
    SETUP_PY,
};
use crate::utilities::{constants, docker, system};
use crate::{cli::display::Message, project::Project};
use log::{error, info};
use std::fs;

// Adapted from https://github.com/nodejs/docker-node/blob/2928850549388a57a33365302fc5ebac91f78ffe/20/bookworm-slim/Dockerfile
// and removed yarn
static TS_BASE_DOCKER_FILE: &str = r#"
FROM node:20-bookworm-slim

# This is to remove the notice to update NPM that will break the output from STDOUT
RUN npm config set update-notifier false
"#;

// Python and node 'slim' term is flipped
static PY_BASE_DOCKER_FILE: &str = r#"
FROM python:3.12-slim-bookworm
"#;

static DOCKER_FILE_COMMON: &str = r#"
ARG DEBIAN_FRONTEND=noninteractive

# Update the package lists for upgrades for security purposes
RUN apt-get update && apt-get upgrade -y

# Install tail and locales package
RUN apt-get install -y locales coreutils curl

# Generate locale files
RUN locale-gen en_US.UTF-8
ENV LANG en_US.UTF-8
ENV LANGUAGE en_US:en
ENV LC_ALL en_US.UTF-8
ENV TZ UTC

# Install Moose
ARG FRAMEWORK_VERSION="0.0.0"
ARG DOWNLOAD_URL
RUN curl -Lo /usr/local/bin/moose ${DOWNLOAD_URL}
RUN chmod +x /usr/local/bin/moose

RUN moose --version

# Setup healthcheck
HEALTHCHECK --interval=30s --timeout=3s \
  CMD curl -f http://localhost:4000/health || exit 1

# Sets up non-root user using 1001 because node creates a user with 1000
RUN groupadd --gid 1001 moose \
  && useradd --uid 1001 --gid moose --shell /bin/bash --create-home moose

# all commands from here on will be run as the moose user
USER moose:moose

# Set the working directory inside the container
WORKDIR /application

# Copy the application files to the container
COPY --chown=moose:moose ./app ./app
# Placeholder for the language specific copy package file copy
COPY_PACKAGE_FILE

# https://stackoverflow.com/questions/70096208/dockerfile-copy-folder-if-it-exists-conditional-copy/70096420#70096420
COPY --chown=moose:moose ./project.tom[l] ./project.toml
COPY --chown=moose:moose ./moose.config.tom[l] ./moose.config.toml
COPY --chown=moose:moose ./versions .moose/versions

# Placeholder for the language specific install command
INSTALL_COMMAND

# Checks that the project is valid
RUN moose check

# Expose the ports on which the application will listen
EXPOSE 4000

# Set the command to run the application
CMD ["moose", "prod"]
"#;

pub fn create_dockerfile(project: &Project) -> Result<RoutineSuccess, RoutineFailure> {
    let internal_dir = project.internal_dir().map_err(|err| {
        error!("Failed to get internal directory for project: {}", err);
        RoutineFailure::new(
            Message::new(
                "Failed".to_string(),
                "to get internal directory for project".to_string(),
            ),
            err,
        )
    })?;

    ensure_docker_running()?;

    let versions_file_path = internal_dir.join("packager/versions/.gitkeep");

    info!("Creating versions at: {:?}", versions_file_path);
    fs::create_dir_all(versions_file_path.parent().unwrap()).map_err(|err| {
        error!("Failed to create directory for app versions: {}", err);
        RoutineFailure::new(
            Message::new(
                "Failed".to_string(),
                "to create directory for app versions".to_string(),
            ),
            err,
        )
    })?;

    let file_path = internal_dir.join("packager/Dockerfile");

    info!("Creating Dockerfile at: {:?}", file_path);
    fs::create_dir_all(file_path.parent().unwrap()).map_err(|err| {
        error!("Failed to create directory for project packaging: {}", err);
        RoutineFailure::new(
            Message::new(
                "Failed".to_string(),
                "to create directory for project packaging".to_string(),
            ),
            err,
        )
    })?;

    let docker_file = match project.language {
        SupportedLanguages::Typescript => {
            let install = DOCKER_FILE_COMMON
                .replace(
                    "COPY_PACKAGE_FILE",
                    "COPY --chown=moose:moose ./package.json ./package.json",
                )
                // We should get compatible with other package managers
                // and respect log files
                .replace("INSTALL_COMMAND", "RUN npm install");

            format!("{}{}", TS_BASE_DOCKER_FILE, install)
        }
        SupportedLanguages::Python => {
            let install = DOCKER_FILE_COMMON
                .replace(
                    "COPY_PACKAGE_FILE",
                    "COPY --chown=moose:moose ./setup.py ./setup.py",
                )
                .replace("INSTALL_COMMAND", "RUN pip install .");

            format!("{}{}", PY_BASE_DOCKER_FILE, install)
        }
    };

    fs::write(&file_path, docker_file).map_err(|err| {
        error!("Failed to write Docker file for project: {}", err);
        RoutineFailure::new(
            Message::new(
                "Failed".to_string(),
                "to write Docker file for project".to_string(),
            ),
            err,
        )
    })?;

    info!("Dockerfile created at: {:?}", file_path);
    Ok(RoutineSuccess::success(Message::new(
        "Successfully".to_string(),
        "created dockerfile".to_string(),
    )))
}

pub fn build_dockerfile(
    project: &Project,
    is_amd64: bool,
    is_arm64: bool,
) -> Result<RoutineSuccess, RoutineFailure> {
    let internal_dir = project.internal_dir().map_err(|err| {
        error!("Failed to get internal directory for project: {}", err);
        RoutineFailure::new(
            Message::new(
                "Failed".to_string(),
                "to get internal directory for project".to_string(),
            ),
            err,
        )
    })?;

    ensure_docker_running()?;
    let file_path = internal_dir.join("packager/Dockerfile");
    let file_path_display = file_path.clone();
    info!("Building Dockerfile at: {:?}", file_path_display);

    fs::create_dir_all(file_path.parent().unwrap()).map_err(|err| {
        error!("Failed to create directory for project packaging: {}", err);
        RoutineFailure::new(
            Message::new(
                "Failed".to_string(),
                "to create directory for project packaging".to_string(),
            ),
            err,
        )
    })?;

    // Copy versions folder to packager directory
    let copy_result = system::copy_directory(
        &internal_dir.join(CLI_INTERNAL_VERSIONS_DIR),
        &internal_dir.join("packager"),
    );
    match copy_result {
        Ok(_) => {
            info!("Copied versions directory to packager directory");
        }
        Err(err) => {
            error!(
                "Failed to copy versions directory to packager directory: {}",
                err
            );
            return Err(RoutineFailure::new(
                Message::new(
                    "Failed".to_string(),
                    "to copy versions directory to packager directory".to_string(),
                ),
                err,
            ));
        }
    }

    // Copy app & etc to packager directory
    let project_root_path = project.project_location.clone();
    let items_to_copy = vec![
        APP_DIR,
        PACKAGE_JSON,
        SETUP_PY,
        PROJECT_CONFIG_FILE,
        OLD_PROJECT_CONFIG_FILE,
    ];

    for item in items_to_copy {
        if !project_root_path.join(item).exists() {
            continue;
        }

        let copy_result = system::copy_directory(
            &project_root_path.join(item),
            &internal_dir.join("packager"),
        );
        match copy_result {
            Ok(_) => {
                info!("Copied {} to packager directory", item);
            }
            Err(err) => {
                error!("Failed to copy {} to packager directory: {}", item, err);
                return Err(RoutineFailure::new(
                    Message::new(
                        "Failed".to_string(),
                        format!("to copy {} to packager directory", item),
                    ),
                    err,
                ));
            }
        }
    }

    // consts::CLI_VERSION is set from an environment variable during the CI/CD process
    // however, it's set to 0.0.1 in development,
    // so we set it to a recent version for the purpose of local dev testing.
    let mut cli_version = constants::CLI_VERSION;
    if cli_version == "0.0.1" {
        cli_version = "0.3.522";
    }

    let build_all = is_amd64 == is_arm64;

    if build_all || is_amd64 {
        info!("Creating docker linux/amd64 image");
        let buildx_result = with_spinner(
            "Creating docker linux/amd64 image",
            || {
                docker::buildx(
                    &internal_dir.join("packager"),
                    cli_version,
                    "linux/amd64",
                    "x86_64-unknown-linux-gnu",
                )
            },
            !project.is_production,
        );
        match buildx_result {
            Ok(_) => {
                info!("Docker image created");
            }
            Err(err) => {
                error!("Failed to create docker image: {}", err);
                return Err(RoutineFailure::new(
                    Message::new("Failed".to_string(), "to create docker image".to_string()),
                    err,
                ));
            }
        }
    }

    if build_all || is_arm64 {
        info!("Creating docker linux/arm64 image");
        let buildx_result = with_spinner(
            "Creating docker linux/arm64 image",
            || {
                docker::buildx(
                    &internal_dir.join("packager"),
                    cli_version,
                    "linux/arm64",
                    "aarch64-unknown-linux-gnu",
                )
            },
            !project.is_production,
        );
        match buildx_result {
            Ok(_) => {
                info!("Docker image created");
            }
            Err(err) => {
                error!("Failed to create docker image: {}", err);
                return Err(RoutineFailure::new(
                    Message::new("Failed".to_string(), "to create docker image".to_string()),
                    err,
                ));
            }
        }
    }

    Ok(RoutineSuccess::success(Message::new(
        "Successfully".to_string(),
        "created docker image for deployment".to_string(),
    )))
}

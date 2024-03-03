use super::{Routine, RoutineFailure, RoutineSuccess};
use crate::cli::display::with_spinner;
use crate::cli::display::MessageType;
use crate::cli::routines::util::ensure_docker_running;
use crate::utilities::{constants, docker, system};
use crate::{cli::display::Message, project::Project};
use log::{error, info};
use std::fs;
use std::sync::Arc;

static DOCKER_FILE: &str = r#"
FROM debian:buster-slim
# Created from docker_packager routine

# Update the package lists for upgrades for security purposes
RUN apt-get update && apt-get upgrade -y

# Install tail and locales package
RUN apt-get install -y locales coreutils curl

# Generate locale files
RUN locale-gen en_US.UTF-8

# Set the working directory inside the container
WORKDIR /application

# Copy the application files to the container
COPY ./app ./app

# Expose the ports on which the application will listen
EXPOSE 4000

# post install
RUN locale-gen en_US.UTF-8
ENV LANG en_US.UTF-8
ENV LANGUAGE en_US:en
ENV LC_ALL en_US.UTF-8
ENV TZ UTC

ARG FRAMEWORK_VERSION="0.0.0"
ARG DOWNLOAD_URL
RUN curl -Lo /usr/local/bin/moose ${DOWNLOAD_URL}
RUN chmod +x /usr/local/bin/moose
RUN moose init mymooseapp ts .

# Set the command to run the application
CMD ["moose", "prod"]
"#;

pub struct CreateDockerfile {
    project: Arc<Project>,
}
impl CreateDockerfile {
    pub fn new(project: Arc<Project>) -> Self {
        Self { project }
    }
}

impl Routine for CreateDockerfile {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        let internal_dir = self.project.internal_dir().map_err(|err| {
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

        info!("Creating Dockerfile at: {:?}", file_path_display);

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

        fs::write(file_path, DOCKER_FILE).map_err(|err| {
            error!("Failed to write Docker file for project: {}", err);
            RoutineFailure::new(
                Message::new(
                    "Failed".to_string(),
                    "to write Docker file for project".to_string(),
                ),
                err,
            )
        })?;

        info!("Dockerfile created at: {:?}", file_path_display);
        Ok(RoutineSuccess::success(Message::new(
            "Successfully".to_string(),
            "created dockerfile".to_string(),
        )))
    }
}

pub struct BuildDockerfile {
    project: Arc<Project>,
}

impl BuildDockerfile {
    pub fn new(project: Arc<Project>) -> Self {
        Self { project }
    }
}

impl Routine for BuildDockerfile {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        let internal_dir = self.project.internal_dir().map_err(|err| {
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

        let project_root_path = self.project.project_location.clone();
        let copy_result = system::copy_directory(
            &project_root_path.join("app"),
            &internal_dir.join("packager"),
        );
        match copy_result {
            Ok(_) => {
                info!("Copied app directory to packager directory");
            }
            Err(err) => {
                error!(
                    "Failed to copy app directory to packager directory: {}",
                    err
                );
                return Err(RoutineFailure::new(
                    Message::new(
                        "Failed".to_string(),
                        "to copy app directory to packager directory".to_string(),
                    ),
                    err,
                ));
            }
        }

        let container_names = docker::list_container_names().unwrap();
        let substring = "buildx_buildkit";

        let contains_substring = container_names.iter().any(|s| s.contains(substring));
        if !contains_substring {
            show_message!(
                MessageType::Error,
                Message {
                    action: "Docker".to_string(),
                    details: "Buildx not found, please load it using `docker buildx create --use` and try again.".to_string(),
                }
            );
        }

        // consts::CLI_VERSION is set from an environment variable during the CI/CD process
        // however, its set to 0.0.1 in development so we set it to 0.3.93 for the purpose of local dev testing.
        let mut cli_version = constants::CLI_VERSION;
        if cli_version == "0.0.1" {
            cli_version = "0.3.101";
        }

        info!("Creating docker linux/amd64 image");
        let buildx_result = with_spinner("Creating docker linux/amd64 image", || {
            docker::buildx(
                &internal_dir.join("packager"),
                cli_version,
                "linux/amd64",
                "x86_64-unknown-linux-gnu",
            )
        });
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

        info!("Creating docker linux/arm64 image");
        let buildx_result = with_spinner("Creating docker linux/arm64 image", || {
            docker::buildx(
                &internal_dir.join("packager"),
                cli_version,
                "linux/arm64",
                "aarch64-unknown-linux-gnu",
            )
        });
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

        Ok(RoutineSuccess::success(Message::new(
            "Successfully".to_string(),
            "created docker image for deployment".to_string(),
        )))
    }
}

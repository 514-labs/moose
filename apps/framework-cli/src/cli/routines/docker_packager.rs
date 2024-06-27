use super::{Routine, RoutineFailure, RoutineSuccess};
use crate::cli::display::with_spinner;
use crate::cli::routines::util::ensure_docker_running;
use crate::utilities::constants::{
    APP_DIR, CLI_INTERNAL_VERSIONS_DIR, OLD_PROJECT_CONFIG_FILE, PACKAGE_JSON, PROJECT_CONFIG_FILE,
};
use crate::utilities::{constants, docker, system};
use crate::{cli::display::Message, project::Project};
use log::{error, info};
use std::fs;
use std::sync::Arc;

static DOCKER_FILE: &str = r#"
# Created from docker_packager routine

ARG DENO_VERSION=1.41.2
FROM denoland/deno:bin-$DENO_VERSION AS deno

FROM debian:bookworm-slim

ARG DEBIAN_FRONTEND=noninteractive

COPY --from=deno /deno /usr/local/bin/deno

# Update the package lists for upgrades for security purposes
RUN apt-get update && apt-get upgrade -y

# Install tail and locales package
RUN apt-get install -y locales coreutils curl

# Install Node
ENV NODE_VERSION 20.13.1

RUN ARCH= OPENSSL_ARCH= && dpkgArch="$(dpkg --print-architecture)" \
    && case "${dpkgArch##*-}" in \
      amd64) ARCH='x64' OPENSSL_ARCH='linux-x86_64';; \
      ppc64el) ARCH='ppc64le' OPENSSL_ARCH='linux-ppc64le';; \
      s390x) ARCH='s390x' OPENSSL_ARCH='linux*-s390x';; \
      arm64) ARCH='arm64' OPENSSL_ARCH='linux-aarch64';; \
      armhf) ARCH='armv7l' OPENSSL_ARCH='linux-armv4';; \
      i386) ARCH='x86' OPENSSL_ARCH='linux-elf';; \
      *) echo "unsupported architecture"; exit 1 ;; \
    esac \
    && set -ex \
    # libatomic1 for arm
    && apt-get update && apt-get install -y ca-certificates curl wget gnupg dirmngr xz-utils libatomic1 --no-install-recommends \
    && rm -rf /var/lib/apt/lists/* \
    # use pre-existing gpg directory, see https://github.com/nodejs/docker-node/pull/1895#issuecomment-1550389150
    && export GNUPGHOME="$(mktemp -d)" \
    # gpg keys listed at https://github.com/nodejs/node#release-keys
    && for key in \
      4ED778F539E3634C779C87C6D7062848A1AB005C \
      141F07595B7B3FFE74309A937405533BE57C7D57 \
      74F12602B6F1C4E913FAA37AD3A89613643B6201 \
      DD792F5973C6DE52C432CBDAC77ABFA00DDBF2B7 \
      61FC681DFB92A079F1685E77973F295594EC4689 \
      8FCCA13FEF1D0C2E91008E09770F7A9A5AE15600 \
      C4F0DFFF4E8C1A8236409D08E73BC641CC11F4C8 \
      890C08DB8579162FEE0DF9DB8BEAB4DFCF555EF4 \
      C82FA3AE1CBEDC6BE46B9360C43CEC45C17AB93C \
      108F52B48DB57BB0CC439B2997B01419BD92F80A \
      A363A499291CBBC940DD62E41F10027AF002F8B0 \
      CC68F5A3106FF448322E48ED27F5E38D5B0A215F \
    ; do \
      gpg --batch --keyserver hkps://keys.openpgp.org --recv-keys "$key" || \
      gpg --batch --keyserver keyserver.ubuntu.com --recv-keys "$key" ; \
    done \
    && curl -fsSLO --compressed "https://nodejs.org/dist/v$NODE_VERSION/node-v$NODE_VERSION-linux-$ARCH.tar.xz" \
    && curl -fsSLO --compressed "https://nodejs.org/dist/v$NODE_VERSION/SHASUMS256.txt.asc" \
    && gpg --batch --decrypt --output SHASUMS256.txt SHASUMS256.txt.asc \
    && gpgconf --kill all \
    && rm -rf "$GNUPGHOME" \
    && grep " node-v$NODE_VERSION-linux-$ARCH.tar.xz\$" SHASUMS256.txt | sha256sum -c - \
    && tar -xJf "node-v$NODE_VERSION-linux-$ARCH.tar.xz" -C /usr/local --strip-components=1 --no-same-owner \
    && rm "node-v$NODE_VERSION-linux-$ARCH.tar.xz" SHASUMS256.txt.asc SHASUMS256.txt \
    # smoke tests
    && node --version \
    && npm --version

# This is to remove the notice to update NPM that will break the output from STDOUT
RUN npm config set update-notifier false

# Generate locale files
RUN locale-gen en_US.UTF-8

# Set the working directory inside the container
WORKDIR /application

# Copy the application files to the container
COPY ./app ./app
COPY ./package.json ./package.json

# https://stackoverflow.com/questions/70096208/dockerfile-copy-folder-if-it-exists-conditional-copy/70096420#70096420
COPY ./project.tom[l] ./project.toml
COPY ./moose.config.tom[l] ./moose.config.toml
COPY ./versions .moose/versions

# We should get compatible with other package managers 
# and respect log files
RUN npm install

# Expose the ports on which the application will listen
EXPOSE 4000

# Setup healthcheck
HEALTHCHECK --interval=30s --timeout=3s \
  CMD curl -f http://localhost:4000/health || exit 1

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

RUN moose --version

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

        fs::write(&file_path, DOCKER_FILE).map_err(|err| {
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
        let project_root_path = self.project.project_location.clone();
        let items_to_copy = vec![
            APP_DIR,
            PACKAGE_JSON,
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
        // however, its set to 0.0.1 in development so we set it to 0.3.93 for the purpose of local dev testing.
        let mut cli_version = constants::CLI_VERSION;
        if cli_version == "0.0.1" {
            cli_version = "0.3.343";
        }

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
            !self.project.is_production,
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
            !self.project.is_production,
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

        Ok(RoutineSuccess::success(Message::new(
            "Successfully".to_string(),
            "created docker image for deployment".to_string(),
        )))
    }
}

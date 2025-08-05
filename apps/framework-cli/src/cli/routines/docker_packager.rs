use super::{RoutineFailure, RoutineSuccess};
use crate::cli::display::with_spinner_completion;
use crate::cli::routines::util::ensure_docker_running;
use crate::framework::languages::SupportedLanguages;
use crate::utilities::constants::{
    APP_DIR, OLD_PROJECT_CONFIG_FILE, PACKAGE_JSON, PROJECT_CONFIG_FILE, REQUIREMENTS_TXT,
    SETUP_PY, TSCONFIG_JSON,
};
use crate::utilities::docker::DockerClient;
use crate::utilities::package_managers::get_lock_file_path;
use crate::utilities::{constants, system};
use crate::{cli::display::Message, project::Project};

use log::{debug, error, info};
use std::fs;
use std::path::{Path, PathBuf};

// Adapted from https://github.com/nodejs/docker-node/blob/2928850549388a57a33365302fc5ebac91f78ffe/20/bookworm-slim/Dockerfile
// and removed yarn
static TS_BASE_DOCKER_FILE: &str = r#"
FROM node:20-bookworm-slim

# This is to remove the notice to update NPM that will break the output from STDOUT
RUN npm config set update-notifier false

# Install alternative package managers globally
RUN npm install -g pnpm@latest
"#;

// Python and node 'slim' term is flipped
static PY_BASE_DOCKER_FILE: &str = r#"
FROM python:3.12-slim-bookworm
"#;

// Monorepo-aware TypeScript Dockerfile template
static TS_MONOREPO_DOCKER_FILE: &str = r#"
# Stage 1: Full monorepo context for dependency resolution
FROM node:20-bookworm-slim AS monorepo-base

# This is to remove the notice to update NPM that will break the output from STDOUT
RUN npm config set update-notifier false

# Install alternative package managers globally
RUN npm install -g pnpm@latest

# Set working directory to monorepo root
WORKDIR /monorepo

# Copy workspace configuration files
COPY pnpm-workspace.yaml ./
COPY pnpm-lock.yaml ./

# Copy workspace package directories (will be replaced with actual patterns)
MONOREPO_PACKAGE_COPIES

# Install all dependencies from monorepo root
RUN pnpm install --frozen-lockfile

# Stage 2: Production image  
FROM node:20-bookworm-slim

# This is to remove the notice to update NPM that will break the output from STDOUT
RUN npm config set update-notifier false

# Install alternative package managers globally
RUN npm install -g pnpm@latest
"#;

static DOCKER_FILE_COMMON: &str = r#"
ARG DEBIAN_FRONTEND=noninteractive

# Update the package lists for upgrades for security purposes
RUN apt-get update && apt-get upgrade -y

# Install tail and locales package
RUN apt-get install -y locales coreutils curl

# moose depends on libc 2.40+, not available in stable
# This uses unstable, but unpinned because they delete older versions
RUN echo "deb http://deb.debian.org/debian/ unstable main" >> /etc/apt/sources.list \
    && apt-get update \
    && apt-get install -y libc6 \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# Generate locale files
RUN locale-gen en_US.UTF-8
ENV LANG=en_US.UTF-8
ENV LANGUAGE=en_US:en
ENV LC_ALL=en_US.UTF-8
ENV TZ=UTC
ENV DOCKER_IMAGE=true

# Install Moose
ARG FRAMEWORK_VERSION="0.0.0"
ARG DOWNLOAD_URL
RUN echo "DOWNLOAD_URL: ${DOWNLOAD_URL}"
RUN ldd --version
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

# Placeholder for the language specific copy package file copy
COPY_PACKAGE_FILE

# https://stackoverflow.com/questions/70096208/dockerfile-copy-folder-if-it-exists-conditional-copy/70096420#70096420
COPY --chown=moose:moose ./project.tom[l] ./project.toml
COPY --chown=moose:moose ./moose.config.tom[l] ./moose.config.toml
COPY --chown=moose:moose ./versions .moose/versions

# Placeholder for the language specific install command
INSTALL_COMMAND

# Checks that the project is valid
RUN which moose
RUN moose check --write-infra-map || (echo "Error running moose check" && exit 1)

# Expose the ports on which the application will listen
EXPOSE 4000

# Set the command to run the application
CMD ["moose", "prod"]
"#;

pub fn create_dockerfile(
    project: &Project,
    docker_client: &DockerClient,
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

    ensure_docker_running(docker_client).map_err(|e| {
        RoutineFailure::new(
            Message::new(
                "Failed".to_string(),
                "to ensure docker is running".to_string(),
            ),
            e,
        )
    })?;

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
            let project_root = project.project_location.clone();

            // Check if we're in a pnpm workspace (monorepo)
            if let Some(workspace_root) = find_pnpm_workspace_root(&project_root) {
                info!("Detected pnpm workspace at: {:?}", workspace_root);

                // Calculate relative path from workspace root to project
                let relative_project_path = match project_root.strip_prefix(&workspace_root) {
                    Ok(path) => path,
                    Err(e) => {
                        error!("Failed to calculate relative project path: {}", e);
                        // Fall back to standard build if we can't determine the relative path
                        return create_standard_typescript_dockerfile(project, &internal_dir);
                    }
                };

                // Read pnpm-workspace.yaml to understand structure
                let workspace_config = match read_workspace_config(&workspace_root) {
                    Ok(config) => config,
                    Err(e) => {
                        error!("Failed to read workspace config: {:?}", e);
                        // Fall back to standard build if we can't read the workspace config
                        return create_standard_typescript_dockerfile(project, &internal_dir);
                    }
                };

                // Generate COPY commands for all workspace packages
                let package_copies =
                    generate_workspace_copy_commands(&workspace_root, &workspace_config);

                // Build the monorepo Dockerfile
                let mut dockerfile = TS_MONOREPO_DOCKER_FILE.to_string();
                dockerfile = dockerfile.replace("MONOREPO_PACKAGE_COPIES", &package_copies);

                // Add the common Docker sections
                dockerfile.push_str(DOCKER_FILE_COMMON);

                // Generate COPY commands for all workspace directories
                let workspace_copies = workspace_config
                    .iter()
                    .filter_map(|pattern| {
                        let base_path = pattern.replace("/*", "");
                        let full_path = workspace_root.join(&base_path);
                        if full_path.exists() && full_path.is_dir() {
                            Some(format!(
                                "COPY --from=monorepo-base /monorepo/{base_path} ./{base_path}"
                            ))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                // Modify the copy commands to copy from the build stage
                let copy_from_build = format!(
                    r#"
# Copy application files from monorepo stage
COPY --from=monorepo-base --chown=moose:moose /monorepo/{}/app ./app
COPY --from=monorepo-base --chown=moose:moose /monorepo/{}/package.json ./package.json
COPY --from=monorepo-base --chown=moose:moose /monorepo/{}/tsconfig.json ./tsconfig.json

# Copy config files
COPY --chown=moose:moose ./project.tom[l] ./project.toml
COPY --chown=moose:moose ./moose.config.tom[l] ./moose.config.toml

# Create node_modules by installing dependencies for this specific workspace
# We need to copy necessary monorepo files and run pnpm install with filter
USER root:root
WORKDIR /temp-monorepo
COPY --from=monorepo-base /monorepo/pnpm-workspace.yaml ./
COPY --from=monorepo-base /monorepo/pnpm-lock.yaml ./
COPY --from=monorepo-base /monorepo/{} ./{}
# Copy all workspace directories that exist
{}
RUN pnpm install --frozen-lockfile --filter "./{}" --shamefully-hoist
# Copy the generated node_modules to the application directory
RUN cp -r /temp-monorepo/{}/node_modules /application/node_modules && \
    chown -R moose:moose /application/node_modules

RUN if [ -d "/application/node_modules/@514labs/moose-lib/dist/" ]; then ls -la /application/node_modules/@514labs/moose-lib/dist/; fi
# Clean up
RUN rm -rf /temp-monorepo
USER moose:moose
WORKDIR /application"#,
                    relative_project_path.to_string_lossy(),
                    relative_project_path.to_string_lossy(),
                    relative_project_path.to_string_lossy(),
                    relative_project_path.to_string_lossy(),
                    relative_project_path.to_string_lossy(),
                    workspace_copies,
                    relative_project_path.to_string_lossy(),
                    relative_project_path.to_string_lossy(),
                );

                dockerfile = dockerfile.replace("COPY_PACKAGE_FILE", &copy_from_build);
                dockerfile = dockerfile.replace(
                    "INSTALL_COMMAND",
                    "# Dependencies already installed in build stage",
                );

                // Store monorepo info for build phase
                let monorepo_info_path = internal_dir.join("packager/.monorepo-info");
                fs::create_dir_all(monorepo_info_path.parent().unwrap()).ok();
                fs::write(
                    &monorepo_info_path,
                    format!(
                        "{}\n{}",
                        workspace_root.to_string_lossy(),
                        relative_project_path.to_string_lossy()
                    ),
                )
                .ok();

                dockerfile
            } else {
                // Not a monorepo, use standard Dockerfile generation
                create_standard_typescript_dockerfile_content(project)?
            }
        }
        SupportedLanguages::Python => {
            let install = DOCKER_FILE_COMMON
                .replace(
                    "COPY_PACKAGE_FILE",
                    r#"COPY --chown=moose:moose ./setup.py ./setup.py
COPY --chown=moose:moose ./requirements.txt ./requirements.txt
COPY --chown=moose:moose ./app ./app"#,
                )
                .replace("INSTALL_COMMAND", "RUN pip install -r requirements.txt");

            format!("{PY_BASE_DOCKER_FILE}{install}")
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
    docker_client: &DockerClient,
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

    ensure_docker_running(docker_client).map_err(|e| {
        RoutineFailure::new(
            Message::new(
                "Failed".to_string(),
                "to ensure docker is running".to_string(),
            ),
            e,
        )
    })?;

    let file_path = internal_dir.join("packager/Dockerfile");
    info!("Building Dockerfile at: {:?}", file_path);

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

    // Copy app & etc to packager directory
    let project_root_path = project.project_location.clone();
    let items_to_copy = vec![
        APP_DIR,
        PACKAGE_JSON,
        SETUP_PY,
        REQUIREMENTS_TXT,
        TSCONFIG_JSON,
        PROJECT_CONFIG_FILE,
        OLD_PROJECT_CONFIG_FILE,
    ];

    // Handle lock file copying for TypeScript projects only (may be from parent directories for monorepos)
    if project.language == SupportedLanguages::Typescript {
        if let Some(lock_file_path) = get_lock_file_path(&project_root_path) {
            // Safely extract filename with proper error handling
            let lock_file_name = match lock_file_path.file_name().and_then(|name| name.to_str()) {
                Some(name) => name,
                None => {
                    error!("Invalid lock file path: {:?}", lock_file_path);
                    return Err(RoutineFailure::new(
                        Message::new(
                            "Failed".to_string(),
                            "to extract lock file name from path".to_string(),
                        ),
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "Invalid lock file path",
                        ),
                    ));
                }
            };

            let destination_path = internal_dir.join("packager").join(lock_file_name);

            match fs::copy(&lock_file_path, &destination_path) {
                Ok(_) => {
                    info!(
                        "Copied lock file from {:?} to packager directory",
                        lock_file_path
                    );
                }
                Err(err) => {
                    error!("Failed to copy lock file {:?}: {}", lock_file_path, err);
                    return Err(RoutineFailure::new(
                        Message::new(
                            "Failed".to_string(),
                            format!("to copy lock file {lock_file_name}"),
                        ),
                        err,
                    ));
                }
            }
        }
    }

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
                        format!("to copy {item} to packager directory"),
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
        cli_version = "0.3.810";
    }

    // Check if this is a monorepo build
    // .monorepo-info is a temporary file generated by our docker_packager process.
    let monorepo_info_path = internal_dir.join("packager/.monorepo-info");
    let (build_context, dockerfile_path) = if monorepo_info_path.exists() {
        // Read monorepo info
        match fs::read_to_string(&monorepo_info_path) {
            Ok(content) => {
                let lines: Vec<&str> = content.lines().collect();
                if lines.len() >= 2 {
                    let workspace_root = PathBuf::from(lines[0]);
                    let relative_project_path = lines[1];
                    info!(
                        "Building monorepo Docker image from workspace root: {:?}",
                        workspace_root
                    );

                    // Create a temporary Dockerfile at the workspace root
                    let temp_dockerfile = workspace_root.join("Dockerfile");

                    // Read the generated Dockerfile and adjust paths for workspace root context
                    let dockerfile_content = fs::read_to_string(&file_path).map_err(|err| {
                        error!("Failed to read generated Dockerfile: {}", err);
                        RoutineFailure::new(
                            Message::new(
                                "Failed".to_string(),
                                "to read generated Dockerfile".to_string(),
                            ),
                            err,
                        )
                    })?;

                    // Replace relative paths in COPY commands for config files
                    let adjusted_dockerfile = dockerfile_content
                        .replace(
                            "COPY --chown=moose:moose ./project.tom[l]",
                                                &format!(
                        "COPY --chown=moose:moose {relative_project_path}/project.tom[l]"
                    ),
                        )
                        .replace(
                            "COPY --chown=moose:moose ./moose.config.tom[l]",
                            &format!(
                                "COPY --chown=moose:moose {relative_project_path}/moose.config.tom[l]"
                            ),
                        )
                        .replace(
                            "COPY --chown=moose:moose ./versions",
                            &format!(
                                "COPY --chown=moose:moose {relative_project_path}/.moose/packager/versions"
                            ),
                        );

                    // Write the adjusted Dockerfile to the workspace root
                    fs::write(&temp_dockerfile, adjusted_dockerfile).map_err(|err| {
                        error!("Failed to write temporary Dockerfile: {}", err);
                        RoutineFailure::new(
                            Message::new(
                                "Failed".to_string(),
                                "to write temporary Dockerfile".to_string(),
                            ),
                            err,
                        )
                    })?;

                    (workspace_root, temp_dockerfile)
                } else {
                    info!("Invalid monorepo info, falling back to standard build");
                    (internal_dir.join("packager"), file_path)
                }
            }
            Err(e) => {
                info!(
                    "Failed to read monorepo info: {}, falling back to standard build",
                    e
                );
                (internal_dir.join("packager"), file_path)
            }
        }
    } else {
        // Standard build
        (internal_dir.join("packager"), file_path)
    };

    let build_all = is_amd64 == is_arm64;

    if build_all || is_amd64 {
        info!("Creating docker linux/amd64 image");
        let buildx_result = with_spinner_completion(
            "Creating docker linux/amd64 image",
            "Docker linux/amd64 image created successfully",
            || {
                // For monorepo builds, we need to copy config files to workspace root
                if build_context != internal_dir.join("packager") {
                    let project_root = project.project_location.clone();
                    let temp_project_toml = build_context.join("project.toml");
                    let temp_moose_config = build_context.join("moose.config.toml");

                    // Copy project files
                    if project_root.join(PROJECT_CONFIG_FILE).exists() {
                        fs::copy(project_root.join(PROJECT_CONFIG_FILE), &temp_project_toml).ok();
                    } else if project_root.join(OLD_PROJECT_CONFIG_FILE).exists() {
                        fs::copy(
                            project_root.join(OLD_PROJECT_CONFIG_FILE),
                            &temp_project_toml,
                        )
                        .ok();
                    }

                    if let Some(moose_config_src) = [
                        internal_dir.join("packager/moose.config.toml"),
                        project_root.join("moose.config.toml"),
                    ]
                    .iter()
                    .find(|p| p.exists())
                    {
                        fs::copy(moose_config_src, &temp_moose_config).ok();
                    }
                }

                docker_client.buildx(
                    &build_context,
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
        let buildx_result = with_spinner_completion(
            "Creating docker linux/arm64 image",
            "Docker linux/arm64 image created successfully",
            || {
                // For monorepo builds, we need to copy config files to workspace root
                if build_context != internal_dir.join("packager") {
                    let project_root = project.project_location.clone();
                    let temp_project_toml = build_context.join("project.toml");
                    let temp_moose_config = build_context.join("moose.config.toml");

                    // Copy project files
                    if project_root.join(PROJECT_CONFIG_FILE).exists() {
                        fs::copy(project_root.join(PROJECT_CONFIG_FILE), &temp_project_toml).ok();
                    } else if project_root.join(OLD_PROJECT_CONFIG_FILE).exists() {
                        fs::copy(
                            project_root.join(OLD_PROJECT_CONFIG_FILE),
                            &temp_project_toml,
                        )
                        .ok();
                    }

                    if let Some(moose_config_src) = [
                        internal_dir.join("packager/moose.config.toml"),
                        project_root.join("moose.config.toml"),
                    ]
                    .iter()
                    .find(|p| p.exists())
                    {
                        fs::copy(moose_config_src, &temp_moose_config).ok();
                    }
                }

                docker_client.buildx(
                    &build_context,
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

    // Clean up temporary files for monorepo builds
    if build_context != internal_dir.join("packager") {
        // Remove temporary Dockerfile
        if dockerfile_path.exists()
            && dockerfile_path.file_name() == Some(std::ffi::OsStr::new("Dockerfile"))
        {
            fs::remove_file(&dockerfile_path).ok();
        }

        // Remove temporary config files
        let temp_project_toml = build_context.join("project.toml");
        let temp_moose_config = build_context.join("moose.config.toml");
        fs::remove_file(temp_project_toml).ok();
        fs::remove_file(temp_moose_config).ok();

        // Remove monorepo info file
        fs::remove_file(monorepo_info_path).ok();
    }

    Ok(RoutineSuccess::success(Message::new(
        "Successfully".to_string(),
        "created docker image for deployment".to_string(),
    )))
}

/// Detects if the project is part of a pnpm workspace by looking for pnpm-workspace.yaml
fn find_pnpm_workspace_root(start_dir: &Path) -> Option<PathBuf> {
    let mut current_dir = start_dir.to_path_buf();

    loop {
        let workspace_file = current_dir.join("pnpm-workspace.yaml");
        if workspace_file.exists() {
            debug!("Found pnpm-workspace.yaml at: {:?}", current_dir);
            return Some(current_dir);
        }

        match current_dir.parent() {
            Some(parent) => current_dir = parent.to_path_buf(),
            None => break,
        }
    }

    None
}

/// Reads and parses pnpm-workspace.yaml to get package patterns
fn read_workspace_config(workspace_root: &Path) -> Result<Vec<String>, RoutineFailure> {
    let workspace_yaml_path = workspace_root.join("pnpm-workspace.yaml");
    let content = fs::read_to_string(&workspace_yaml_path).map_err(|e| {
        RoutineFailure::new(
            Message::new(
                "Failed".to_string(),
                "to read pnpm-workspace.yaml".to_string(),
            ),
            e,
        )
    })?;

    // Parse YAML to extract package patterns
    let mut patterns = Vec::new();
    let mut in_packages = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Check if we're entering the packages section
        if trimmed == "packages:" {
            in_packages = true;
            continue;
        }

        // If we're in packages section
        if in_packages {
            // Check if this is a list item (handles various indentations)
            if let Some(item) = trimmed.strip_prefix("- ") {
                // Remove quotes if present
                let pattern = item.trim_matches('"').trim_matches('\'');
                patterns.push(pattern.to_string());
            } else if !trimmed.starts_with(' ') && !trimmed.starts_with('\t') {
                // We've hit a new top-level key, exit packages section
                in_packages = false;
            }
        }
    }

    Ok(patterns)
}

/// Generates COPY commands for workspace packages that exist
fn generate_workspace_copy_commands(workspace_root: &Path, patterns: &[String]) -> String {
    let mut copy_commands = Vec::new();

    for pattern in patterns {
        // Remove wildcards for COPY command
        let base_path = pattern.replace("/*", "");

        // Check if the directory exists
        let full_path = workspace_root.join(&base_path);
        if full_path.exists() && full_path.is_dir() {
            copy_commands.push(format!("COPY {base_path} ./{base_path}"));
        } else {
            debug!("Skipping non-existent workspace directory: {base_path}");
        }
    }

    copy_commands.join("\n")
}

/// Creates standard TypeScript Dockerfile content (non-monorepo)
fn create_standard_typescript_dockerfile_content(
    project: &Project,
) -> Result<String, RoutineFailure> {
    let project_root = project.project_location.clone();
    let (install_command, lock_file_copy) =
        if let Some(lock_file_path) = get_lock_file_path(&project_root) {
            // Determine package manager based on detected lock file
            let lock_file_name = lock_file_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");

            match lock_file_name {
                "pnpm-lock.yaml" => (
                    "RUN pnpm install --frozen-lockfile".to_string(),
                    "COPY --chown=moose:moose ./pnpm-lock.yaml ./pnpm-lock.yaml",
                ),
                "package-lock.json" => (
                    "RUN npm ci".to_string(),
                    "COPY --chown=moose:moose ./package-lock.json ./package-lock.json",
                ),
                "yarn.lock" => (
                    "RUN yarn install --frozen-lockfile".to_string(),
                    "COPY --chown=moose:moose ./yarn.lock ./yarn.lock",
                ),
                _ => {
                    // Fallback to configured package manager if lock file is unrecognized
                    let pm = &project.typescript_config.package_manager;
                    (format!("RUN {pm} install"), "")
                }
            }
        } else {
            // No lock file found, use configured package manager
            let pm = &project.typescript_config.package_manager;
            (format!("RUN {pm} install"), "")
        };

    // Build copy commands for package files
    let mut copy_commands = vec![
        "COPY --chown=moose:moose ./package.json ./package.json",
        "COPY --chown=moose:moose ./tsconfig.json ./tsconfig.json",
        "COPY --chown=moose:moose ./app ./app",
    ];

    // Add lock file copy command if detected
    if !lock_file_copy.is_empty() {
        copy_commands.push(lock_file_copy);
    }

    let copy_section = copy_commands.join("\n                    ");

    let install = DOCKER_FILE_COMMON
        .replace(
            "COPY_PACKAGE_FILE",
            &format!("\n                    {copy_section}"),
        )
        .replace("INSTALL_COMMAND", &install_command);

    Ok(format!("{TS_BASE_DOCKER_FILE}{install}"))
}

/// Creates standard TypeScript Dockerfile and writes it to disk
fn create_standard_typescript_dockerfile(
    project: &Project,
    internal_dir: &Path,
) -> Result<RoutineSuccess, RoutineFailure> {
    let dockerfile_content = create_standard_typescript_dockerfile_content(project)?;
    let file_path = internal_dir.join("packager/Dockerfile");

    fs::write(&file_path, dockerfile_content).map_err(|err| {
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

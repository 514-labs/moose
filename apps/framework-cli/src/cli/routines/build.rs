/// # Build Module
///
/// This module provides functionality for creating deployment packages without Docker.
/// It builds a standalone ZIP archive that can be deployed to any server running Moose.
///
/// ## Architecture
///
/// The build process follows these steps:
/// 1. Create a staging directory in the project's `.moose/packager` folder
/// 2. Copy essential project files (app directory, config files)
/// 3. Handle language-specific dependencies based on the project type (Node.js or Python)
/// 4. Run validation with `moose check` to ensure the package is valid
/// 5. Create a README with deployment instructions
/// 6. Package everything into a ZIP archive with the project name and current date
///
/// ## Example
///
/// ```rust,no_run
/// use crate::project::Project;
/// use crate::cli::routines::build::build_package;
///
/// fn deploy(project: &Project) -> Result<(), Box<dyn std::error::Error>> {
///     let package_path = build_package(project)?;
///     println!("Package created at: {}", package_path.display());
///     Ok(())
/// }
/// ```
use chrono::Local;
use log::{debug, error, info};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::cli::display::with_spinner;
use crate::framework::languages::SupportedLanguages;
use crate::project::Project;
use crate::project::ProjectFileError;
use crate::utilities::constants::{APP_DIR, OLD_PROJECT_CONFIG_FILE, PROJECT_CONFIG_FILE};
use crate::utilities::system;

/// Represents errors that can occur during the build process.
///
/// This enum provides specific error variants for different failure scenarios
/// during the package creation process, allowing for clear error reporting and
/// targeted error handling.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum BuildError {
    /// Standard IO errors that may occur during file operations.
    ///
    /// This includes file not found, permission denied, etc.
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    /// Project-specific file errors.
    ///
    /// Occurs when there are issues with project configuration files or structure.
    #[error("Project file error: {0}")]
    ProjectFile(#[from] ProjectFileError),

    /// Error when cleaning up an existing package directory.
    ///
    /// This happens when trying to remove a previous build directory fails.
    #[error("Failed to clean up existing package directory: {0}")]
    CleanupFailed(String),

    /// Error when creating the package directory.
    ///
    /// This happens when the system cannot create the directory for staging files.
    #[error("Failed to create package directory: {0}")]
    CreateDirFailed(String),

    /// Error when copying a file to the package directory.
    ///
    /// The first parameter identifies the file being copied,
    /// and the second parameter provides the error details.
    #[error("Failed to copy {0} to package directory: {1}")]
    FileCopyFailed(String, String),

    /// Error with Node.js dependencies.
    ///
    /// This includes failures in npm installation or copying node_modules.
    #[error("Failed to copy Node dependencies: {0}")]
    NodeDependenciesFailed(String),

    /// Error with Python dependencies.
    ///
    /// This includes failures in generating requirements.txt or other Python-specific steps.
    #[error("Failed to copy Python dependencies: {0}")]
    PythonDependenciesFailed(String),

    /// Error running the moose check command.
    ///
    /// This happens when validation of the package contents fails.
    #[error("Failed to run moose check: {0}")]
    MooseCheckFailed(String),

    /// Error creating the README file.
    ///
    /// This happens when the system cannot write the README.md file to the package.
    #[error("Failed to create README file: {0}")]
    ReadmeFailed(String),

    /// Error creating the ZIP archive.
    ///
    /// This happens when the system cannot create or move the final ZIP archive.
    #[error("Failed to create archive: {0}")]
    ArchiveFailed(String),
}

/// Directory name for the package staging area.
///
/// This is the directory where files are collected before being archived.
pub const BUILD_PACKAGE_DIR: &str = "packager";

/// Builds a deployment package for a Moose project without using Docker.
///
/// This function creates a self-contained ZIP archive that includes all the necessary
/// files to run a Moose application on a server. The package includes:
///  - The app directory with application code
///  - Configuration files
///  - Language-specific dependencies
///  - A generated README with deployment instructions
///
/// # Arguments
///
/// * `project` - A reference to the Project being packaged
///
/// # Returns
///
/// * `Result<PathBuf, BuildError>` - On success, returns the path to the created archive
///
/// # Errors
///
/// Returns a `BuildError` if any step of the build process fails, including:
/// - IO operations
/// - Dependency collection
/// - Validation
/// - Archive creation
///
/// # Side Effects
///
/// - Creates a `.moose/packager` directory
/// - Runs `moose check --write-infra-map` in the package directory
/// - Creates a ZIP archive in the `.moose` directory
pub fn build_package(project: &Project) -> Result<PathBuf, BuildError> {
    info!("Starting build process for deployment package");

    // Get internal directory for staging
    let internal_dir = project.internal_dir()?;

    // Create the package directory (equivalent to packager in docker_packager)
    let package_dir = internal_dir.join(BUILD_PACKAGE_DIR);
    info!("Setting up package directory at: {:?}", package_dir);

    // Clean up any existing packager directory
    if package_dir.exists() {
        fs::remove_dir_all(&package_dir).map_err(|err| {
            error!("Failed to clean up existing package directory: {}", err);
            BuildError::CleanupFailed(err.to_string())
        })?;
    }

    // Create the package directory
    fs::create_dir_all(&package_dir).map_err(|err| {
        error!("Failed to create package directory: {}", err);
        BuildError::CreateDirFailed(err.to_string())
    })?;

    // Copy project files to packager directory
    let project_root_path = project.project_location.clone();

    // Files to include in the package
    let files_to_copy = [APP_DIR, PROJECT_CONFIG_FILE, OLD_PROJECT_CONFIG_FILE];

    for item in &files_to_copy {
        let source_path = project_root_path.join(item);
        let dest_path = package_dir.join(item);

        if !source_path.exists() {
            debug!("Skipping {}, does not exist", item);
            continue;
        }

        match copy_recursive(&source_path, &dest_path) {
            Ok(_) => {
                debug!("Copied {} to package directory", item);
            }
            Err(err) => {
                error!("Failed to copy {} to package directory: {}", item, err);
                return Err(BuildError::FileCopyFailed(
                    item.to_string(),
                    err.to_string(),
                ));
            }
        }
    }

    // Copy language-specific dependencies
    match project.language {
        SupportedLanguages::Typescript => {
            copy_node_dependencies(project, &package_dir)
                .map_err(|e| BuildError::NodeDependenciesFailed(e.to_string()))?;
        }
        SupportedLanguages::Python => {
            copy_python_dependencies(project, &package_dir)
                .map_err(|e| BuildError::PythonDependenciesFailed(e.to_string()))?;
        }
    }

    // Run moose check with --write-infra-map
    run_moose_check(&package_dir).map_err(|e| BuildError::MooseCheckFailed(e.to_string()))?;

    // Create README with deployment instructions
    create_readme(&package_dir).map_err(|e| BuildError::ReadmeFailed(e.to_string()))?;

    // Create the archive
    let archive_path = create_archive(project, &package_dir)
        .map_err(|e| BuildError::ArchiveFailed(e.to_string()))?;

    info!(
        "Successfully created deployment package at {}",
        archive_path.display()
    );

    Ok(archive_path)
}

/// Recursively copies files and directories from source to destination.
///
/// This is a utility function that handles both individual files and directory structures,
/// preserving the file hierarchy in the destination.
///
/// # Arguments
///
/// * `source` - Path to the source file or directory
/// * `destination` - Path where the file or directory should be copied
///
/// # Returns
///
/// * `Result<(), std::io::Error>` - Returns Ok(()) on success
///
/// # Errors
///
/// Returns an IO error if any file operation fails, such as:
/// - Reading directory contents
/// - Creating directories
/// - Copying files
///
/// # Notes
///
/// This function will create parent directories if they don't exist.
fn copy_recursive(source: &PathBuf, destination: &PathBuf) -> Result<(), std::io::Error> {
    if source.is_dir() {
        fs::create_dir_all(destination)?;

        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let entry_path = entry.path();
            let file_name = entry.file_name();
            let dest_path = destination.join(file_name);

            if entry_path.is_dir() {
                copy_recursive(&entry_path, &dest_path)?;
            } else {
                fs::copy(&entry_path, &dest_path)?;
            }
        }
        Ok(())
    } else {
        // Ensure the parent directory exists
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(source, destination)?;
        Ok(())
    }
}

/// Copies Node.js dependencies to the package directory.
///
/// This function handles copying the node_modules directory to the package.
/// If node_modules doesn't exist, it attempts to install dependencies first.
///
/// # Arguments
///
/// * `project` - A reference to the Project containing Node.js dependencies
/// * `package_dir` - Path to the package directory where dependencies should be copied
///
/// # Returns
///
/// * `Result<(), BuildError>` - Returns Ok(()) on success
///
/// # Errors
///
/// Returns a `BuildError` if:
/// - npm install fails
/// - Copying node_modules fails
/// - node_modules doesn't exist even after attempting installation
///
/// # Side Effects
///
/// May run `npm install` in the project directory if node_modules doesn't exist
fn copy_node_dependencies(project: &Project, package_dir: &PathBuf) -> Result<(), BuildError> {
    info!("Copying Node.js dependencies");

    let node_modules_path = project.project_location.join("node_modules");
    if !node_modules_path.exists() {
        // Try to install dependencies if node_modules doesn't exist
        info!("node_modules not found, installing dependencies...");
        let result = with_spinner(
            "Installing npm dependencies",
            || {
                let mut cmd = Command::new("npm");
                cmd.current_dir(&project.project_location);
                cmd.arg("install");
                let output = cmd.output()?;
                if !output.status.success() {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!(
                            "npm install failed: {}",
                            String::from_utf8_lossy(&output.stderr)
                        ),
                    ));
                }
                Ok(())
            },
            false,
        );

        if let Err(err) = result {
            return Err(BuildError::NodeDependenciesFailed(err.to_string()));
        }
    }

    // Now copy the node_modules
    if node_modules_path.exists() {
        info!("Copying node_modules to package directory");

        let copy_result = system::copy_directory(&node_modules_path, package_dir);
        if let Err(err) = copy_result {
            error!("Failed to copy node_modules to package directory: {}", err);
            return Err(BuildError::NodeDependenciesFailed(err.to_string()));
        }
    } else {
        error!("node_modules directory not found even after npm install");
        return Err(BuildError::NodeDependenciesFailed(
            "node_modules directory not found".to_string(),
        ));
    }

    Ok(())
}

/// Copies Python dependencies to the package directory.
///
/// Instead of copying the virtual environment, this function generates
/// a requirements.txt file by freezing the current environment.
///
/// # Arguments
///
/// * `project` - A reference to the Project containing Python dependencies
/// * `package_dir` - Path to the package directory where the requirements.txt will be created
///
/// # Returns
///
/// * `Result<(), BuildError>` - Returns Ok(()) on success
///
/// # Errors
///
/// Returns a `BuildError` if:
/// - Running pip freeze fails
/// - Writing the requirements.txt file fails
///
/// # Side Effects
///
/// Runs `pip freeze --exclude-editable` in the project directory
fn copy_python_dependencies(project: &Project, package_dir: &PathBuf) -> Result<(), BuildError> {
    info!("Preparing Python dependencies");

    // Create requirements.txt file
    let requirements_path = package_dir.join("requirements.txt");
    let result = with_spinner(
        "Generating requirements.txt",
        || {
            let mut cmd = Command::new("pip");
            cmd.current_dir(&project.project_location);
            cmd.args(["freeze", "--exclude-editable"]);
            let output = cmd.output()?;
            if !output.status.success() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "pip freeze failed: {}",
                        String::from_utf8_lossy(&output.stderr)
                    ),
                ));
            }

            let requirements = String::from_utf8_lossy(&output.stdout).to_string();
            fs::write(&requirements_path, requirements)?;
            Ok(())
        },
        false,
    );

    if let Err(err) = result {
        return Err(BuildError::PythonDependenciesFailed(err.to_string()));
    }

    info!("Requirements.txt generated successfully");
    Ok(())
}

/// Runs moose check with --write-infra-map in the package directory.
///
/// This validates the package and generates necessary infrastructure maps.
///
/// # Arguments
///
/// * `package_dir` - Path to the package directory where moose check should run
///
/// # Returns
///
/// * `Result<(), BuildError>` - Returns Ok(()) on success
///
/// # Errors
///
/// Returns a `BuildError` if the moose check command fails
///
/// # Side Effects
///
/// Runs `moose check --write-infra-map` in the package directory,
/// which updates the infrastructure map files
fn run_moose_check(package_dir: &PathBuf) -> Result<(), BuildError> {
    info!("Running moose check with --write-infra-map");

    let result = with_spinner(
        "Running moose check",
        || {
            let mut cmd = Command::new("moose");
            cmd.current_dir(package_dir);
            cmd.args(["check", "--write-infra-map"]);
            let output = cmd.output()?;
            if !output.status.success() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "moose check failed: {}",
                        String::from_utf8_lossy(&output.stderr)
                    ),
                ));
            }
            Ok(())
        },
        false,
    );

    if let Err(err) = result {
        return Err(BuildError::MooseCheckFailed(err.to_string()));
    }

    info!("Moose check completed successfully");
    Ok(())
}

/// Creates a README file with deployment instructions in the package directory.
///
/// This provides users with guidance on how to deploy the package on a server.
///
/// # Arguments
///
/// * `package_dir` - Path to the package directory where the README should be created
///
/// # Returns
///
/// * `Result<(), BuildError>` - Returns Ok(()) on success
///
/// # Errors
///
/// Returns a `BuildError` if writing the README file fails
///
/// # Side Effects
///
/// Creates a README.md file in the package directory
fn create_readme(package_dir: &PathBuf) -> Result<(), BuildError> {
    info!("Creating README with deployment instructions");

    let readme_content = r#"# Moose Application Deployment Package

This package contains a Moose application ready for deployment.

## Deployment Instructions

1. Extract this package to your deployment server
2. Set up any required environment variables
3. Run `moose prod` to start the application in production mode

For more information about Moose, visit https://www.moosejs.com/
"#;

    fs::write(package_dir.join("README.md"), readme_content).map_err(|err| {
        error!("Failed to create README file: {}", err);
        BuildError::ReadmeFailed(err.to_string())
    })?;

    info!("README created successfully");
    Ok(())
}

/// Creates a zip archive of the package directory.
///
/// The archive name includes the project name and current date.
///
/// # Arguments
///
/// * `project` - A reference to the Project being packaged
/// * `package_dir` - Path to the package directory that should be archived
///
/// # Returns
///
/// * `Result<PathBuf, BuildError>` - On success, returns the path to the created archive
///
/// # Errors
///
/// Returns a `BuildError` if:
/// - Creating the zip archive fails
/// - Moving the archive to the .moose directory fails
///
/// # Side Effects
///
/// - Creates a ZIP file in the parent directory of the package directory
/// - Moves the ZIP file to the .moose directory
fn create_archive(project: &Project, package_dir: &PathBuf) -> Result<PathBuf, BuildError> {
    info!("Creating zip archive of package");

    let project_name = project.name();
    let date = Local::now().format("%Y-%m-%d");
    let archive_name = format!("{}-{}.zip", project_name, date);
    let internal_dir = project.internal_dir().map_err(|err| {
        error!("Failed to get internal directory for project: {}", err);
        BuildError::ProjectFile(err)
    })?;

    let archive_path = internal_dir.join(&archive_name);

    let result = with_spinner(
        &format!("Creating archive {}", archive_name),
        || {
            // Use zip command to create archive
            let status = Command::new("zip")
                .current_dir(package_dir.parent().unwrap())
                .args(["-r", &archive_name, "packager"])
                .status()?;

            if !status.success() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "zip command failed",
                ));
            }

            // Move the archive to the .moose directory
            let temp_archive = package_dir.parent().unwrap().join(&archive_name);
            if temp_archive.exists() {
                fs::rename(&temp_archive, &archive_path)?;
            } else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Archive was not created correctly",
                ));
            }

            Ok(())
        },
        false,
    );

    if let Err(err) = result {
        return Err(BuildError::ArchiveFailed(err.to_string()));
    }

    info!("Archive created successfully at: {:?}", archive_path);
    Ok(archive_path)
}

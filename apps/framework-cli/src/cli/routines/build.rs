use chrono::Local;
use log::{debug, error, info};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::framework::languages::SupportedLanguages;
use crate::project::Project;
use crate::project::ProjectFileError;
use crate::utilities::constants::{
    APP_DIR, PACKAGE_JSON, PROJECT_CONFIG_FILE, SETUP_PY, TSCONFIG_JSON,
};
use crate::utilities::system;

/// Represents errors that can occur during the build process
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum BuildError {
    /// IO error during file operations
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    /// Project file error
    #[error("Project file error: {0}")]
    ProjectFile(#[from] ProjectFileError),

    /// Error cleaning up package directory
    #[error("Failed to clean up existing package directory: {0}")]
    CleanupFailed(String),

    /// Error creating package directory
    #[error("Failed to create package directory: {0}")]
    CreateDirFailed(String),

    /// Error copying file to package directory
    #[error("Failed to copy {0} to package directory: {1}")]
    FileCopyFailed(String, String),

    /// Error with Node dependencies
    #[error("Failed to copy Node dependencies: {0}")]
    NodeDependenciesFailed(String),

    /// Error with Python dependencies
    #[error("Failed to copy Python dependencies: {0}")]
    PythonDependenciesFailed(String),

    /// Error running moose check
    #[error("Failed to run moose check: {0}")]
    MooseCheckFailed(String),

    /// Error creating README
    #[error("Failed to create README file: {0}")]
    ReadmeFailed(String),

    /// Error creating archive
    #[error("Failed to create archive: {0}")]
    ArchiveFailed(String),
}

pub const BUILD_PACKAGE_DIR: &str = "packager";

/// Builds a deployment package without using Docker
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
    let files_to_copy = [
        APP_DIR,
        PROJECT_CONFIG_FILE,
        PACKAGE_JSON,
        SETUP_PY,
        TSCONFIG_JSON,
    ];

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
            return Err(BuildError::PythonDependenciesFailed(
                "Python project not supported, please contact support@fiveonefour.com to get it supported".to_string(),
            ));
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

/// Recursively copies files and directories from source to destination
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

/// Copies Node.js dependencies to the package directory
fn copy_node_dependencies(project: &Project, package_dir: &PathBuf) -> Result<(), BuildError> {
    info!("Copying Node.js dependencies");

    let node_modules_path = project.project_location.join("node_modules");
    if !node_modules_path.exists() {
        // Try to install dependencies if node_modules doesn't exist
        return Err(BuildError::NodeDependenciesFailed(
            "node_modules directory not found, please run `npm install` or your package manager of choice to install dependencies".to_string(),
        ));
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

/// Runs moose check with --write-infra-map in the package directory
fn run_moose_check(package_dir: &PathBuf) -> Result<(), BuildError> {
    info!("Running moose check with --write-infra-map");

    let mut cmd = Command::new("moose");
    cmd.current_dir(package_dir);
    cmd.args(["check", "--write-infra-map"]);
    let output = cmd.output()?;
    if !output.status.success() {
        return Err(BuildError::MooseCheckFailed(
            "moose check failed".to_string(),
        ));
    }

    info!("Moose check completed successfully");
    Ok(())
}

/// Creates a README file with deployment instructions
fn create_readme(package_dir: &PathBuf) -> Result<(), BuildError> {
    info!("Creating README with deployment instructions");

    let readme_content = r#"# Moose Application Deployment Package

This package contains a Moose application ready for deployment. The builds needs to run 
on a machine that has the same os and architecture as the deployment target.

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

/// Creates a zip archive of the package directory
fn create_archive(project: &Project, package_dir: &PathBuf) -> Result<PathBuf, BuildError> {
    info!("Creating zip archive of package");

    let project_name = project.name();
    let date = Local::now().format("%Y-%m-%d");
    let archive_name = format!("{}-{}.zip", project_name, date);
    let internal_dir = project.internal_dir()?;

    let archive_path = internal_dir.join(&archive_name);

    // Use zip command to create archive silently
    // Change directory into the packager directory first so files are at top level in the zip
    let status = Command::new("zip")
        .current_dir(package_dir) // Changed to package_dir instead of parent
        .args(["-q", "-r", &archive_name, "."]) // Changed "packager" to "." to zip current directory contents
        .status()?;

    if !status.success() {
        return Err(BuildError::ArchiveFailed("zip command failed".to_string()));
    }

    // Move the archive to the .moose directory
    let temp_archive = package_dir.join(&archive_name);
    if temp_archive.exists() {
        fs::rename(&temp_archive, &archive_path)?;
    } else {
        return Err(BuildError::ArchiveFailed(
            "Archive was not created correctly".to_string(),
        ));
    }

    info!("Archive created successfully at: {:?}", archive_path);
    Ok(archive_path)
}

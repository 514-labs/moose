//! Utilities for interacting with npm.

use std::{fmt, path::PathBuf, process::Command};

use home::home_dir;
use log::{debug, error};

pub fn get_root() -> Result<PathBuf, std::io::Error> {
    let result = Command::new("npm").arg("root").arg("-g").output()?;

    let stdout =
        String::from_utf8(result.stdout).expect("Failed to get npm root. Is npm installed?");

    Ok(PathBuf::from(stdout.trim()))
}

pub fn get_or_create_global_folder() -> Result<PathBuf, std::io::Error> {
    //! Get the global folder for npm.
    //!
    //! # Returns
    //! - `Result<PathBuf, std::io::Error>` - A result containing the path to the global folder.
    //!
    let home_dir = home_dir().expect("Failed to get home directory.");

    let global_mod_folder = home_dir.join(".node_modules");

    if global_mod_folder.exists() {
        return Ok(global_mod_folder);
    } else {
        std::fs::create_dir(&global_mod_folder)?;
    }

    Ok(global_mod_folder)
}

pub enum PackageManager {
    Npm,
    Pnpm,
}

impl fmt::Display for PackageManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PackageManager::Npm => write!(f, "npm"),
            PackageManager::Pnpm => write!(f, "pnpm"),
        }
    }
}

pub fn install_packages(
    directory: &PathBuf,
    package_manager: &PackageManager,
) -> Result<(), std::io::Error> {
    //! Install packages in a directory.
    //! This is useful for installing packages in a directory other than the current directory.

    debug!("Installing packages in directory: {:?}", directory);

    let mut command = Command::new(package_manager.to_string());
    command.current_dir(directory);
    command.arg("install");

    let output = command.output()?; // We should explore not using output here and instead using spawn.
    let stdout = output.stdout.clone();
    match String::from_utf8(output.stdout) {
        Ok(val) => {
            debug!("{}", val);
        }
        Err(e) => {
            error!(
                "Failed to decode stdout as UTF-8 for command: `{}` with args {:?} in directory {:?}. Raw output: {:?}. Error: {:?}",
                package_manager.to_string(),
                command.get_args().collect::<Vec<_>>(),
                command.get_current_dir(),
                &stdout,
                e
            );
        }
    }

    Ok(())
}

pub fn run_build(
    directory: &PathBuf,
    package_manager: &PackageManager,
) -> Result<(), std::io::Error> {
    let mut command = Command::new(package_manager.to_string());
    command.current_dir(directory);
    command.arg("run");
    command.arg("build");

    let output = command.output()?; // We should explore not using output here and instead using spawn.
    let stdout = output.stdout.clone();
    match String::from_utf8(output.stdout) {
        Ok(val) => {
            debug!("{}", val);
        }
        Err(e) => {
            error!(
                "Failed to decode stdout as UTF-8 for command: `{}` with args {:?} in directory {:?}. Raw output: {:?}. Error: {:?}",
                package_manager.to_string(),
                command.get_args().collect::<Vec<_>>(),
                command.get_current_dir(),
                &stdout,
                e
            );
        }
    }

    Ok(())
}

pub fn link_sdk(
    directory: &PathBuf,
    package_name: Option<String>,
    package_manager: &PackageManager,
) -> Result<(), std::io::Error> {
    //! Links a package to the global pnpm folder if package_name is None. If the package_name is Some, then it will link the package to
    //! the global pnpm folder with the given name.
    let mut command = Command::new(package_manager.to_string());
    command.current_dir(directory);
    command.arg("link");
    command.arg("--global");

    if let Some(name) = package_name {
        command.arg(name);
    }

    let output = command.output()?; // We should explore not using output here and instead using spawn.
    let stdout = output.stdout.clone();
    match String::from_utf8(output.stdout) {
        Ok(val) => {
            debug!("{}", val);
        }
        Err(e) => {
            error!(
                "Failed to decode stdout as UTF-8 for command: `{}` with args {:?} in directory {:?}. Raw output: {:?}. Error: {:?}",
                package_manager.to_string(),
                command.get_args().collect::<Vec<_>>(),
                command.get_current_dir(),
                &stdout,
                e
            );
        }
    }

    Ok(())
}

/// Detects the package manager to use based on lock files present in the project directory.
///
/// The detection follows this priority order:
/// 1. pnpm-lock.yaml -> pnpm
/// 2. package-lock.json -> npm
/// 3. Default to npm if no lock files found
///
/// # Arguments
///
/// * `project_dir` - Path to the project directory to scan for lock files
///
/// # Returns
///
/// * `PackageManager` - The detected package manager
pub fn detect_package_manager(project_dir: &PathBuf) -> PackageManager {
    use crate::utilities::constants::{PACKAGE_LOCK_JSON, PNPM_LOCK};

    debug!("Detecting package manager in directory: {:?}", project_dir);

    // Check for pnpm-lock.yaml first (most specific)
    if project_dir.join(PNPM_LOCK).exists() {
        debug!("Found pnpm-lock.yaml, using pnpm");
        return PackageManager::Pnpm;
    }

    // Check for package-lock.json
    if project_dir.join(PACKAGE_LOCK_JSON).exists() {
        debug!("Found package-lock.json, using npm");
        return PackageManager::Npm;
    }

    // Default to npm
    debug!("No lock files found, defaulting to npm");
    PackageManager::Npm
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_output_of_command() -> Result<(), std::io::Error> {
        //! Test to demonstrate the use of command and output handling
        //! Note: this test will fail if npm isn't installed.
        use super::*;
        let mut command = Command::new("npm");
        command.arg("version");
        let output = command.output()?;

        assert!(output.status.success());
        match String::from_utf8(output.stdout) {
            Ok(val) => {
                assert!(!val.is_empty());
            }
            Err(_e) => {
                panic!();
            }
        }

        Ok(())
    }
}

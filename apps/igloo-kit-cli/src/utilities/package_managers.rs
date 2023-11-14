//! Utilities for interacting with npm.

use std::{path::PathBuf, process::Command, fmt};

use home::home_dir;


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

pub fn install_packages(directory: &PathBuf, package_manager: &PackageManager) -> Result<(), std::io::Error> {
    //! Install packages in a directory.
    //! This is useful for installing packages in a directory other than the current directory.
    let mut command = Command::new(package_manager.to_string());
    command.current_dir(&directory);
    command.arg("install");

    let output = command.output()?; // We should explore not using output here and instead using spawn.
    println!("{}", String::from_utf8(output.stdout).unwrap());

    Ok(())
}

pub fn run_build(directory: &PathBuf, package_manager: &PackageManager) -> Result<(), std::io::Error> {
    let mut command = Command::new(package_manager.to_string());
    command.current_dir(&directory);
    command.arg("run");
    command.arg("build");

    let output = command.output()?; // We should explore not using output here and instead using spawn.
    println!("{}", String::from_utf8(output.stdout).unwrap());

    Ok(())
}

pub fn link_sdk(directory: &PathBuf, package_name: Option<String>, package_manager: &PackageManager) -> Result<(), std::io::Error> {
    //! Links a package to the global pnpm folder if package_name is None. If the package_name is Some, then it will link the package to 
    //! the global pnpm folder with the given name.
    let mut command = Command::new(package_manager.to_string());
    command.current_dir(&directory);
    command.arg("link");
    command.arg("--global");

    if package_name.is_some() {
        command.arg(package_name.unwrap());
    }

    let output = command.output()?; // We should explore not using output here and instead using spawn.
    println!("{}", String::from_utf8(output.stdout).unwrap());

    Ok(())
}


use std::{path::PathBuf, io::{Error, ErrorKind}};

use crate::project::{self, Project};

const APP_DIR: [&str; 8] = [
    "app",
    "app/ingestion_points",
    "app/dataframes",
    "app/flows",
    "app/insights",
    "app/insights/dashboards",
    "app/insights/models",
    "app/insights/metrics",
];

pub fn create_igloo_directory(project: Project) -> Result<PathBuf, Error> {
    let igloo_dir = project.location.join(".igloo");

    std::fs::create_dir_all(igloo_dir.clone())?;

    Ok(igloo_dir)
}

pub fn get_igloo_directory(project: Project) -> Result<PathBuf, Error> {
    let igloo_dir = project.location.join(".igloo");

    if igloo_dir.exists() {
        Ok(igloo_dir)
    } else {
        Err(Error::new(ErrorKind::NotFound, "Igloo directory not found"))
    }
}

pub fn get_igloo_directory_from_current() -> Result<PathBuf, Error> {
    let current_dir = std::env::current_dir()?;
    let igloo_dir = current_dir.join(".igloo");

    if igloo_dir.exists() {
        Ok(igloo_dir)
    } else {
        Err(Error::new(ErrorKind::NotFound, "Igloo directory not found"))
    }
}

// Create the app directory and subdirectories in the current directory
pub fn create_app_directories(project: Project) -> Result<(), std::io::Error> {

    for dir in APP_DIR.iter() {
        std::fs::create_dir_all(project.location.join(dir))?;
    }

    Ok(())
}

// Retrieved the app directory in the directory the current directory
pub fn get_app_directory(project: Project) -> Result<PathBuf, std::io::Error> {
    
    let app_dir = project.location.join("app");

    if app_dir.exists() {
        Ok(app_dir)
    } else {
        Err(Error::new(ErrorKind::NotFound, "App directory not found"))
    }
}
use std::{path::PathBuf, io::{Error, ErrorKind}};

use crate::{infrastructure::setup::scaffold::{validate_mount_volumes, create_temp_data_volumes}, cli::{user_messages::{show_message, MessageType, Message}, CommandTerminal}};


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

fn create_igloo_directory() -> Result<PathBuf, Error> {
    let current_dir = std::env::current_dir()?;
    let igloo_dir = current_dir.join(".igloo");

    std::fs::create_dir_all(igloo_dir.clone())?;

    Ok(igloo_dir)
}

pub fn get_igloo_directory() -> Result<PathBuf, Error> {
    let current_dir = std::env::current_dir()?;
    let igloo_dir = current_dir.join(".igloo");

    if igloo_dir.exists() {
        Ok(igloo_dir)
    } else {
        Err(Error::new(ErrorKind::NotFound, "Igloo directory not found"))
    }
}

// Creates the .igloo directory and the Red Panda and Clickhouse mount volumes
pub fn create_top_level_temp_dir(term: &mut CommandTerminal) -> Result<PathBuf, std::io::Error> {
    match create_igloo_directory() {
        Ok(igloo_dir) => {
            match validate_mount_volumes(&igloo_dir) {
                Ok(_) => {
                    show_message( term, MessageType::Info, Message {
                        action: "Found",
                        details: "Red Panda and Clickhouse mount volumes in .igloo directory",
                    });
                    return Ok(igloo_dir)
                },
                Err(_) => {
                    show_message( term, MessageType::Info, Message {
                        action: "Creating",
                        details: "Red Panda and Clickhouse mount volumes in .igloo directory",
                    });
                    {
                        create_temp_data_volumes(term)?;
                    };
                }
            }
            Ok(igloo_dir)
        },
        Err(err) => {
            show_message( term, MessageType::Error, Message {
                action: "Failed",
                details: "to create .igloo directory in current working directory",
            });
            Err(err)
        }
    }
}

// Create the app directory and subdirectories in the current directory
pub fn create_app_directories(term: &mut CommandTerminal) -> Result<(), std::io::Error> {
    let current_dir = std::env::current_dir()?;

    show_message( term, MessageType::Info, Message {
        action: "Creating",
        details: "app directory in current working directory",
    });

    for dir in APP_DIR.iter() {
        std::fs::create_dir_all(current_dir.join(dir))?;
    }

    Ok(())
}

// Retrieved the app directory in the directory the current directory
pub fn get_app_directory(term: &mut CommandTerminal) -> Result<PathBuf, std::io::Error> {
    let current_dir = std::env::current_dir()?;
    let app_dir = current_dir.join("app");

    if app_dir.exists() {
        Ok(app_dir)
    } else {
        show_message( term, MessageType::Error, Message {
            action: "Failed",
            details: "to find app directory in current working directory",
        });
        Err(Error::new(ErrorKind::NotFound, "App directory not found"))
    }
}
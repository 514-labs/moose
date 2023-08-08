use std::{path::PathBuf, io::{Error, ErrorKind}};

use crate::{infrastructure::setup::scaffold::{create_red_panda_mount_volume, create_clickhouse_mount_volume, validate_mount_volumes}, cli::{user_messages::{show_message, MessageType, Message}, CommandTerminal}};


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
                        let igloo_dir = &igloo_dir;
                        match create_red_panda_mount_volume(&igloo_dir) {
                            Ok(dir) => {
                                let dir_display = dir.display();
                                show_message( term, MessageType::Success, Message {
                                    action: "Created",
                                    details: &format!("Red Panda mount volume in {dir_display}"),
                                });
                            },
                            Err(err) => {
                                let dir_display = igloo_dir.display();
                                show_message( term, MessageType::Error, Message {
                                    action: "Failed",
                                    details: &format!("to create Red Panda mount volume in {dir_display}"),
                                });
                                return Err(err)
                            }
                        };
                        match create_clickhouse_mount_volume(&igloo_dir) {
                            Ok(_) => {
                                show_message( term, MessageType::Success, Message {
                                    action: "Created",
                                    details: &format!("Clickhouse mount volumes in .clickhouse directory"),
                                });
                            },
                            Err(err) => {
                                let dir_display = igloo_dir.display();
                                show_message( term, MessageType::Error, Message {
                                    action: "Failed",
                                    details: &format!("to create Red Panda mount volume in {dir_display}"),
                                });
                                return Err(err)
                            }
                        };
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




pub fn create_project_directories(term: &mut CommandTerminal) -> Result<(), std::io::Error> {
    let current_dir = std::env::current_dir().unwrap();

    show_message( term, MessageType::Info, Message {
        action: "Creating",
        details: "app directory in current working directory",
    });

    for dir in APP_DIR.iter() {
        std::fs::create_dir_all(current_dir.join(dir))?;
    }

    Ok(())
}
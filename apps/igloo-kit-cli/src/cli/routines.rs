use std::{path::PathBuf, io::{Error, ErrorKind}};

use super::{CommandTerminal, user_messages::show_message, MessageType, Message};
use crate::{infrastructure::setup::scaffold::{create_red_panda_mount_volume, create_clickhouse_mount_volume, validate_mount_volumes}, framework::{TopLevelObjects, self}};



fn create_igloo_directory() -> Result<PathBuf, Error> {
    let current_dir = std::env::current_dir()?;
    let igloo_dir = current_dir.join(".igloo");

    std::fs::create_dir_all(igloo_dir.clone())?;

    Ok(igloo_dir)

}

fn check_for_igloo_directory() -> Result<PathBuf, Error> {
    let current_dir = std::env::current_dir()?;
    let igloo_dir = current_dir.join(".igloo");

    if igloo_dir.exists() {
        Ok(igloo_dir)
    } else {
        Err(Error::new(ErrorKind::NotFound, "Igloo directory not found"))
    }
}


fn create_temp_mount_volumes(term: &mut CommandTerminal,igloo_dir: PathBuf) -> Result<(), std::io::Error> {
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
    Ok(())
}

fn create_top_level_temp_dir(term: &mut CommandTerminal) -> Result<(), std::io::Error> {
    match create_igloo_directory() {
        Ok(igloo_dir) => {
            match validate_mount_volumes(&igloo_dir) {
                Ok(_) => {
                    show_message( term, MessageType::Info, Message {
                        action: "Found",
                        details: "Red Panda and Clickhouse mount volumes in .igloo directory",
                    });
                    return Ok(())
                },
                Err(_) => {
                    show_message( term, MessageType::Info, Message {
                        action: "Creating",
                        details: "Red Panda and Clickhouse mount volumes in .igloo directory",
                    });
                    create_temp_mount_volumes(term, igloo_dir)?
                }
            }
            Ok(())
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

fn create_temp_data_volumes(term: &mut CommandTerminal) -> Result<(), std::io::Error> {
    match check_for_igloo_directory() {
        Ok(igloo_dir) => {
            create_red_panda_mount_volume(&igloo_dir)?;
            create_clickhouse_mount_volume(&igloo_dir)?;
            Ok(())
        },
        Err(_) => {
            show_message( term, MessageType::Warning, Message {
                action: "Not found",
                details: ".igloo directory in current working directory",
            });
            show_message( term, MessageType::Info, Message {
                action: "Creating",
                details: ".igloo directory in current working directory",
            });
            create_top_level_temp_dir(term)
            }
    }
}


fn create_project_directories(term: &mut CommandTerminal) {
    let current_dir = std::env::current_dir().unwrap();

    show_message( term, MessageType::Info, Message {
        action: "Creating",
        details: "app directory in current working directory",
    });

    let app_dir: PathBuf = current_dir.join("app");

    let ingestion_dir = app_dir.join("ingestion_points");
    let dataframes_dir = app_dir.join("dataframes");
    let flows_dir = app_dir.join("flows");
    let insights_dir = app_dir.join("insights");


    let models_dir = insights_dir.join("models");
    let dashboards_dir = insights_dir.join("dashboards");
    let metrics_dir = insights_dir.join("metrics");

    let dirs = vec![app_dir, ingestion_dir, dataframes_dir, flows_dir, insights_dir, models_dir, dashboards_dir, metrics_dir];

    for dir in dirs.iter() {
        std::fs::create_dir_all(dir).unwrap();
    }

    show_message( term, MessageType::Success, Message {
        action: "Finished",
        details: "project initialization",
    });
}


pub fn initialize_project(term: &mut CommandTerminal) -> Result<(), Error> {
    create_temp_data_volumes(term)?;
    create_project_directories(term);
    Ok(())
}
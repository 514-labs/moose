use std::{fs, path::PathBuf, io::{Error, ErrorKind}};

use crate::{cli::{CommandTerminal, user_messages::{MessageType, Message, show_message}}, framework::directories::{get_igloo_directory, create_top_level_temp_dir}};

use super::database;

// Validate that the clickhouse and redpanda volume paths exist
pub fn validate_mount_volumes(igloo_dir: &PathBuf) -> Result<(), Error> {
    let panda_house = igloo_dir.join(".panda_house").exists();
    let clickhouse = igloo_dir.join(".clickhouse").exists();

    if panda_house && clickhouse {
        Ok(())
    } else {
        
        Err(Error::new(ErrorKind::Other, format!("Mount volume status: redpanda: {panda_house}, clickhouse: {clickhouse}")))
    }
}


pub fn delete_red_panda_mount_volume(igloo_dir: &PathBuf) -> Result<(), Error> {
    let mount_dir = igloo_dir.join(".panda_house");
    let output = fs::remove_dir_all(mount_dir.clone());

    match output {
        Ok(_) =>{ 
            println!("Removed mount directory at {}", mount_dir.display());
            Ok(())
        },
        Err(err) => {
            println!("Failed to remove mount directory at {}", mount_dir.display());
            println!("error: {}", err);
            Err(err)
        },
    }
}

pub fn delete_clickhouse_mount_volume(igloo_dir: &PathBuf) -> Result<(), Error> {
    let mount_dir = igloo_dir.join(".clickhouse");
    let output = fs::remove_dir_all(mount_dir.clone());

    match output {
        Ok(_) => {
            println!("Removed mount directory at {}", mount_dir.display());
            Ok(())
        },
        Err(err) => {
            println!("Failed to remove mount directory at {}", mount_dir.display());
            println!("error: {}", err);
            Err(err)
        },
    }
}

pub fn create_volumes(term: &mut CommandTerminal, igloo_dir: &PathBuf) -> Result<(), Error> {
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
                details: &format!("to create Clickhouse mount volume in {dir_display}"),
            });
            println!("error: {}", err);
            return Err(err)
        }
    };
    Ok(())
}

pub fn create_temp_data_volumes(term: &mut CommandTerminal) -> Result<(), std::io::Error> {
    match get_igloo_directory() {
        Ok(igloo_dir) => {
            create_volumes(term, &igloo_dir)?;
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
            match create_top_level_temp_dir(term) {
                Ok(path) => {
                    create_volumes(term, &path)?;
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
    }}
}

fn create_red_panda_mount_volume(igloo_dir: &PathBuf) -> Result<PathBuf, Error> {
    let mount_dir = igloo_dir.join(".panda_house");
    fs::create_dir_all(mount_dir.clone()).map(|_| mount_dir)
}

fn create_clickhouse_mount_volume(igloo_dir: &PathBuf) -> Result<(), Error> {
    let mount_dir = igloo_dir.join(".clickhouse");

    fs::create_dir_all(mount_dir.clone())?;
    fs::create_dir_all(mount_dir.clone().join("data"))?;
    fs::create_dir_all(mount_dir.clone().join("logs"))?;

    let config_path = mount_dir.clone().join("configs");

    let server_config_path = config_path.clone().join("server");
    let user_config_path = config_path.clone().join("users");
    let scripts_path = config_path.clone().join("scripts");

    // fs::create_dir_all(&server_config_path)?;
    fs::create_dir_all(&user_config_path)?;
    fs::create_dir_all(&scripts_path)?;
    // database::create_server_config_file(&server_config_path)?;
    // database::create_user_config_file(&user_config_path)?;
    // database::create_init_script(&scripts_path)?;
    Ok(())
}
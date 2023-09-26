use std::{path::PathBuf, io::Error, fs};

use crate::{cli::{CommandTerminal, display::show_message}, infrastructure::PANDA_NETWORK, utilities::docker};

use super::stop::{stop_red_panda_container, stop_clickhouse_container};

pub fn clean_project(term: &mut CommandTerminal, igloo_dir: &PathBuf) -> Result<(), Error> {
    stop_red_panda_container(term)?;
    stop_clickhouse_container(term)?;
    remove_docker_network(term, PANDA_NETWORK)?;
    delete_clickhouse_mount_volume(igloo_dir)?;
    delete_red_panda_mount_volume(igloo_dir)?;
    Ok(())
}


pub fn remove_docker_network(term: &mut crate::cli::CommandTerminal, network_name: &str) -> Result<(), std::io::Error> {
    let output = docker::remove_network(network_name);

    match output {
        Ok(_) => {
            show_message(
                term,
                crate::cli::display::MessageType::Success,
                crate::cli::display::Message {
                    action: "Successfully",
                    details: "removed docker network",
                },
            );
            Ok(())
        },
        Err(_) => {
            show_message(
                term,
                crate::cli::display::MessageType::Error,
                crate::cli::display::Message {
                    action: "Failed",
                    details: "to remove docker network",
                },
            );
            Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to remove docker network"))
        },
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
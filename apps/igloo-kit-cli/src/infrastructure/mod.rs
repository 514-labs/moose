use std::{path::PathBuf, io::Error};

use crate::{cli::{CommandTerminal, user_messages::{show_message, MessageType, Message}}, framework::directories};

use self::setup::{
    scaffold::{ delete_clickhouse_mount_volume, delete_red_panda_mount_volume, validate_mount_volumes, create_volumes}, 
    container::{stop_red_panda_container, stop_clickhouse_container, run_red_panda_docker_container, run_ch_docker_container}, 
    network::{create_docker_network, remove_docker_network}, 
    validate::{validate_red_panda_run, validate_clickhouse_run}
};
pub mod setup;
mod docker;


pub fn init(term: &mut CommandTerminal, igloo_dir: &PathBuf) -> Result<(), Error> {
    create_docker_network(term)?;
    create_volumes(term, igloo_dir)?;
    Ok(())
}


// Stop the redpanda and clickhouse containers, removes the docker network, and deletes the temporary data volumes
pub fn clean(term: &mut CommandTerminal, igloo_dir: &PathBuf) -> Result<(), Error> {
    stop_red_panda_container(term)?;
    stop_clickhouse_container(term)?;
    remove_docker_network(term)?;
    delete_clickhouse_mount_volume(igloo_dir)?;
    delete_red_panda_mount_volume(igloo_dir)?;
    Ok(())
}

pub fn spin_up(term: &mut CommandTerminal) -> Result<(), Error> {
    let igloo_dir = match directories::get_igloo_directory() {
        Ok(dir) => dir,
        Err(err) => {
            show_message( term, MessageType::Error, Message {
                action: "Failed",
                details: "Please run `igloo init` to create the necessary mount volumes",
            });
            return Err(err);
        }
    };

    println!("runnign spin up");
     match validate_mount_volumes(&igloo_dir) {
        Ok(_) => {
            create_docker_network(term)?;
            run_red_panda_docker_container(term,true)?;
            validate_red_panda_run(term,true)?;
            run_ch_docker_container(term, true)?;
            validate_clickhouse_run(term, true)?;
            Ok(())
        },
        Err(err) => {
            show_message( term, MessageType::Error, Message {
                action: "Failed",
                details: "Please run `igloo init` to create the necessary mount volumes",
            });
            return Err(err);
        }
    }
}
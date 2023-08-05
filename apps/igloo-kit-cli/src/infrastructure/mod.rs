use std::{path::PathBuf, io::Error};

use crate::cli::CommandTerminal;

use self::setup::{
    scaffold::{ delete_clickhouse_mount_volume, delete_red_panda_mount_volume, validate_mount_volumes, create_volumes}, 
    container::{stop_red_panda_container, stop_ch_container, run_red_panda_docker_container, run_ch_docker_container}, 
    network::{create_docker_network, remove_docker_network}, 
    validate::validate_docker_run
};


pub mod setup;
mod docker;


pub fn init(term: &mut CommandTerminal, igloo_dir: &PathBuf) -> Result<(), Error> {
    create_docker_network();
    create_volumes(term, igloo_dir)?;
    Ok(())
}


// Stop the redpanda and clickhouse containers, removes the docker network, and deletes the temporary data volumes
pub fn clean(igloo_dir: &PathBuf) {
    stop_red_panda_container();
    stop_ch_container();
    remove_docker_network();
    delete_clickhouse_mount_volume(igloo_dir);
    delete_red_panda_mount_volume(igloo_dir);
}

pub fn spin_up(term: &mut CommandTerminal,igloo_dir: &PathBuf) -> Result<(), Error> {
     match validate_mount_volumes(igloo_dir) {
        Ok(_) => {
            create_docker_network();
            run_red_panda_docker_container(term,true)?;
            validate_docker_run(true);
            run_ch_docker_container(term, true)?;
        },
        Err(err) => {
            println!("{}", err);
            println!("Please run `igloo init` to create the necessary mount volumes");
        }
    };
    create_docker_network();
    run_red_panda_docker_container(term, true)?;
    validate_docker_run(true);
    run_ch_docker_container(term, true)?;
    Ok(())
}
use std::{path::PathBuf, io::Error};

use self::setup::{
    scaffold::{create_red_panda_mount_volume, create_clickhouse_mount_volume, delete_clickhouse_mount_volume, delete_red_panda_mount_volume, validate_mount_volumes}, 
    container::{stop_red_panda_container, stop_ch_container, run_red_panda_docker_container, run_ch_docker_container}, 
    network::{create_docker_network, remove_docker_network}, 
    validate::validate_docker_run
};


mod setup;
mod docker;

pub fn init_volumes(igloo_dir: &PathBuf) -> Result<(), Error> {
    create_red_panda_mount_volume(igloo_dir)?;
    create_clickhouse_mount_volume(igloo_dir)?;
    Ok(())
}

pub fn init(igloo_dir: &PathBuf) -> Result<(), Error> {
    create_docker_network();
    init_volumes(igloo_dir)?;
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

pub fn spin_up(igloo_dir: &PathBuf) {
     match validate_mount_volumes(igloo_dir) {
        Ok(_) => {
            create_docker_network();
            run_red_panda_docker_container(true);
            validate_docker_run(true);
            run_ch_docker_container(true)
        },
        Err(err) => {
            println!("{}", err);
            println!("Please run `igloo init` to create the necessary mount volumes");
        }
    };
    create_docker_network();
    run_red_panda_docker_container(true);
    validate_docker_run(true);
    run_ch_docker_container(true)
}
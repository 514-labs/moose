use self::setup::{
    scaffold::{create_red_panda_mount_volume, create_ch_mount_volume, delete_mount_volume}, 
    container::{stop_red_panda_container, stop_ch_container, run_red_panda_docker_container, run_ch_docker_container}, 
    network::{create_docker_network, remove_docker_network}, 
    validate::validate_docker_run
};


mod setup;
mod docker;

pub fn init() {
    create_docker_network();
    create_red_panda_mount_volume();
    create_ch_mount_volume();
}

pub fn teardown() {
    stop_red_panda_container();
    stop_ch_container();
    remove_docker_network();
    delete_mount_volume();
}

pub fn spin_up() {
    create_docker_network();
    run_red_panda_docker_container(true);
    validate_docker_run(true);
    run_ch_docker_container(true)
}
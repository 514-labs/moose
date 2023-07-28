use self::setup::{create_docker_network, create_rp_mount_volume, run_rp_docker_container, validate_docker_run, remove_docker_network, delete_mount_volume, stop_rp_container, stop_ch_container, run_ch_docker_container, create_ch_mount_volume};

mod setup;
mod docker;

pub fn init() {
    create_docker_network();
    create_rp_mount_volume();
    create_ch_mount_volume();
}

pub fn teardown() {
    stop_rp_container();
    stop_ch_container();
    remove_docker_network();
    delete_mount_volume();
}

pub fn spin_up() {
    create_docker_network();
    run_rp_docker_container(true);
    validate_docker_run(true);
    run_ch_docker_container(true)
}
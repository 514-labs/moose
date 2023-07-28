use crate::infrastructure::docker::{self};

pub fn create_docker_network() {
    let output = docker::create_network();

    match output {
        Ok(_) => println!("Created docker network"),
        Err(_) => println!("Failed to create docker network"),
    }
}

pub fn remove_docker_network() {
    let output = docker::remove_network();

    match output {
        Ok(_) => println!("Removed docker network"),
        Err(_) => println!("Failed to remove docker network"),
    }
}
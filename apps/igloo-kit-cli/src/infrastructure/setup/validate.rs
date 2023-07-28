use std::{fs, env, io::{self, Write}};
use crate::infrastructure::docker::{self, run_clickhouse};

pub fn validate_docker_run(debug: bool) {
    let output = docker::filter_list_containers("redpanda-1");

    match output {
        Ok(o) => {
            if debug {
                io::stdout().write_all(&o.stdout).unwrap();
            }
            let output = String::from_utf8(o.stdout).unwrap();
            if output.contains("redpanda-1") {
                println!("Successfully validated docker container");
            } else {
                println!("Failed to validate docker container");
            }
        },
        Err(_) => println!("Failed to validate docker container"),
    }
}

pub fn validate_red_panda_cluster(debug: bool) {
    let output = docker::run_rpk_list();

    match output {
        Ok(o) => {
            if debug {
                io::stdout().write_all(&o.stdout).unwrap();
            }
            let output = String::from_utf8(o.stdout).unwrap();
            if output.contains("redpanda-1") {
                println!("Successfully validated docker container");
            } else {
                println!("Failed to validate docker container");
            }
        },
        Err(_) => println!("Failed to validate redpanda cluster"),
    }
}
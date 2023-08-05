use std::{env, io::{self, Write}};
use crate::infrastructure::docker::{self, run_clickhouse};

pub fn stop_red_panda_container() {
    let output = docker::stop_container("redpanda-1");

    match output {
        Ok(_) => println!("Stopped docker container"),
        Err(_) => println!("Failed to stop docker container"),
    }
}

pub fn run_red_panda_docker_container(debug: bool) {
    let current_dir = env::current_dir().unwrap();

    let output = docker::run_red_panda(current_dir);

    match output {
        Ok(o) => {
            if debug {
                println!("Debugging docker container run");
                io::stdout().write_all(&o.stdout).unwrap();
            }
            println!("Successfully ran docker container")
        },
        Err(_) => println!("Failed to run docker container"),
    }
        
}

pub fn run_ch_docker_container(debug: bool) {
    let current_dir = env::current_dir().unwrap();
    let output = run_clickhouse(current_dir);

    match  output {
        Ok(o) => {
            if debug {
                println!("Debugging docker container run");
                io::stdout().write_all(&o.stdout).unwrap();
            }
            println!("Successfully ran clickhouse container")
        },
        Err(_) => println!("Failed to run clickhouse container"),
    }
}

pub fn stop_ch_container() {
    let output = docker::stop_container("clickhousedb-1");

    match output {
        Ok(_) => println!("Stopped docker container"),
        Err(_) => println!("Failed to stop docker container"),
    }
}
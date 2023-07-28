use std::{fs, env, io::{self, Write}};
use crate::infrastructure::docker::{self, run_clickhouse};


pub fn remove_docker_network() {
    let output = docker::remove_network();

    match output {
        Ok(_) => println!("Removed docker network"),
        Err(_) => println!("Failed to remove docker network"),
    }
}

pub fn delete_mount_volume() {
    let current_dir = env::current_dir().unwrap();
    let mount_dir = current_dir.join(".panda_house");
    
    let output = fs::remove_dir_all(mount_dir.clone());

    match output {
        Ok(_) => println!("Removed mount directory at {}", mount_dir.display()),
        Err(err) => {
            println!("Failed to remove mount directory at {}", mount_dir.display());
            println!("error: {}", err)
        },
    }
}


pub fn stop_red_panda_container() {
    let output = docker::stop_container("redpanda-1");

    match output {
        Ok(_) => println!("Stopped docker container"),
        Err(_) => println!("Failed to stop docker container"),
    }
}



pub fn create_red_panda_mount_volume() {
    let current_dir = env::current_dir().unwrap();
    let mount_dir = current_dir.join(".panda_house");
    
    // This function will fail silently if the directory already exists
    let output = fs::create_dir_all(mount_dir.clone());

    match output {
        Ok(_) => println!("Created mount directory at {}", mount_dir.display()),
        Err(err) => {
            println!("Failed to create mount directory at {}", mount_dir.display());
            println!("error: {}", err)
        },
    }
}

pub fn create_ch_mount_volume() {
    let current_dir = env::current_dir().unwrap();
    let mount_dir = current_dir.join(".clickhouse");

    // This function will fail silently if the directory already exists
    let main_dir_result = fs::create_dir_all(mount_dir.clone());
    let data_dir_result = fs::create_dir_all(mount_dir.clone().join("data"));
    let logs_dir_result = fs::create_dir_all(mount_dir.clone().join("logs"));

    match main_dir_result {
        Ok(_) => {
            println!("Created main mount directory at {}", mount_dir.display());

            match data_dir_result {
                Ok(_) => println!("Created data mount directory at {}", mount_dir.clone().join("data").display()),
                Err(err) => {
                    println!("Failed to create data mount directory at {}", mount_dir.clone().join("data").display());
                    println!("error: {}", err)
                },
            }

            match logs_dir_result {
                Ok(_) => println!("Created logs mount directory at {}", mount_dir.clone().join("logs").display()),
                Err(err) => {
                    println!("Failed to create logs mount directory at {}", mount_dir.clone().join("logs").display());
                    println!("error: {}", err)
                },
            }
        },
        Err(err) => {
            println!("Failed to create mount directory at {}", mount_dir.display());
            println!("error: {}", err)
        },
    }
}

pub fn create_docker_network() {
    let output = docker::create_network();

    match output {
        Ok(_) => println!("Created docker network"),
        Err(_) => println!("Failed to create docker network"),
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

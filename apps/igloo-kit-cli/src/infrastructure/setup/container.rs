use std::{env, io::{self, Write}};
use crate::{infrastructure::docker::{self, run_clickhouse}, cli::{CommandTerminal, user_messages::{show_message, MessageType, Message}}};

// TODO: Print output to terminal with proper messages
pub fn stop_red_panda_container() {
    let output = docker::stop_container("redpanda-1");

    match output {
        Ok(_) => println!("Stopped docker container"),
        Err(_) => println!("Failed to stop docker container"),
    }
}

pub fn run_red_panda_docker_container(term: &mut CommandTerminal, debug: bool) -> Result<(), io::Error> {
    let current_dir = env::current_dir().unwrap();

    let output = docker::run_red_panda(current_dir);

    match output {
        Ok(o) => {
            if debug {
                println!("Debugging docker container run");
                io::stdout().write_all(&o.stdout).unwrap();
            }
            show_message( term, MessageType::Success, Message {
                action: "Successfully",
                details: "ran redpanda container",
            });
            Ok(())
        },
        Err(err) => {
            show_message( term, MessageType::Error, Message {
                action: "Failed",
                details: "to run redpanda container",
            });
            Err(err)
        },
    }
        
}

pub fn run_ch_docker_container(term: &mut CommandTerminal, debug: bool) -> Result<(), io::Error> {
    let current_dir = env::current_dir().unwrap();
    let output = run_clickhouse(current_dir);

    match  output {
        Ok(o) => {
            if debug {
                println!("Debugging docker container run");
                io::stdout().write_all(&o.stdout).unwrap();
            }
            show_message( term, MessageType::Success, Message {
                action: "Successfully",
                details: "ran clickhouse container",
            });
            Ok(())
        },
        Err(err) => {
            show_message( term, MessageType::Error, Message {
                action: "Failed",
                details: "to run clickhouse container",
            });
            Err(err)
        },
    }
}

// TODO: Print output to terminal with proper messages
pub fn stop_ch_container() {
    let output = docker::stop_container("clickhousedb-1");

    match output {
        Ok(_) => println!("Stopped docker container"),
        Err(_) => println!("Failed to stop docker container"),
    }
}
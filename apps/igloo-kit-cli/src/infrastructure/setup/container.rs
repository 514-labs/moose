use std::io::{self, Write};
use crate::{infrastructure::docker::{self, run_clickhouse}, cli::{CommandTerminal, user_messages::{show_message, MessageType, Message}}, framework::directories};


pub fn run_red_panda_docker_container(term: &mut CommandTerminal, debug: bool) -> Result<(), io::Error> {
    let igloo_dir = directories::get_igloo_directory()?;
    let output = docker::run_red_panda(igloo_dir);

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
    let igloo_dir = directories::get_igloo_directory()?;
    let output = run_clickhouse(igloo_dir);

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

pub fn stop_clickhouse_container(term: &mut CommandTerminal) -> Result<(), io::Error> {
    let output = docker::stop_container("clickhousedb-1");

    match output {
        Ok(_) => {
            show_message(
                term,
                MessageType::Success,
                Message {
                    action: "Successfully",
                    details: "stopped clickhouse container",
                },
            );
            Ok(())
        },
        Err(_) => {show_message(
            term,
            MessageType::Error,
            Message {
                action: "Failed",
                details: "to stop clickhouse container",
            });
            Err(io::Error::new(io::ErrorKind::Other, "Failed to stop clickhouse container"))
    }}
}

pub fn stop_red_panda_container(term: &mut CommandTerminal) -> Result<(), io::Error> {
    let output = docker::stop_container("redpanda-1");

    match output {
        Ok(_) => {
            show_message(
                term,
                MessageType::Success,
                Message {
                    action: "Successfully",
                    details: "stopped redpanda container",
                },
            );
            Ok(())
        },
        Err(_) => {show_message(
            term,
            MessageType::Error,
            Message {
                action: "Failed",
                details: "to stop redpanda container",
            });
            Err(io::Error::new(io::ErrorKind::Other, "Failed to stop redpanda container"))
    }}
}
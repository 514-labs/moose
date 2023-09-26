use std::io::{Error, ErrorKind};

use crate::{cli::{CommandTerminal, display::{show_message, MessageType, Message}}, utilities::docker};


pub fn spin_down(term: &mut CommandTerminal) -> Result<(), Error> {
    stop_red_panda_container(term)?;
    stop_clickhouse_container(term)?;
    Ok(())
}

pub fn stop_clickhouse_container(term: &mut CommandTerminal) -> Result<(), Error> {
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
            Err(Error::new(ErrorKind::Other, "Failed to stop clickhouse container"))
    }}
}

pub fn stop_red_panda_container(term: &mut CommandTerminal) -> Result<(), Error> {
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
            Err(Error::new(ErrorKind::Other, "Failed to stop redpanda container"))
    }}
}
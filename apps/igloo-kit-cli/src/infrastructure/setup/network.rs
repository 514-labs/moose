use crate::{cli::user_messages::show_message, utilities::docker};

pub fn create_docker_network(term: &mut crate::cli::CommandTerminal, network_name: &str) -> Result<(), std::io::Error> {
    let output = docker::create_network(network_name);

    match output {
        Ok(_) =>{
            show_message(
                term,
                crate::cli::user_messages::MessageType::Success,
                crate::cli::user_messages::Message {
                    action: "Successfully",
                    details: "created docker network",
                },
            );
            Ok(())
        },
        Err(_) => {
            show_message(
                term,
                crate::cli::user_messages::MessageType::Error,
                crate::cli::user_messages::Message {
                    action: "Failed",
                    details: "to create docker network",
                },
            );
            Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to create docker network"))
        },
    }
}

pub fn remove_docker_network(term: &mut crate::cli::CommandTerminal, network_name: &str) -> Result<(), std::io::Error> {
    let output = docker::remove_network(network_name);

    match output {
        Ok(_) => {
            show_message(
                term,
                crate::cli::user_messages::MessageType::Success,
                crate::cli::user_messages::Message {
                    action: "Successfully",
                    details: "removed docker network",
                },
            );
            Ok(())
        },
        Err(_) => {
            show_message(
                term,
                crate::cli::user_messages::MessageType::Error,
                crate::cli::user_messages::Message {
                    action: "Failed",
                    details: "to remove docker network",
                },
            );
            Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to remove docker network"))
        },
    }
}
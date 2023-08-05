use std::io::Error;

use crate::{infrastructure::{self, setup::scaffold}, framework::directories};

use super::{CommandTerminal, user_messages::show_message, MessageType, Message};

pub fn start_containers(term: &mut CommandTerminal) -> Result<(), Error> {
    show_message( term, MessageType::Info, Message {
        action: "Running",
        details: "infrastructure spin up",
    });
    
    infrastructure::spin_up(term)?;
    Ok(())
}

pub fn initialize_project(term: &mut CommandTerminal) -> Result<(), Error> {
    scaffold::create_temp_data_volumes(term)?;
    match directories::create_project_directories(term) {
        Ok(_) => {
            show_message( term, MessageType::Success, Message {
                action: "Finished",
                details: "initializing project directory",
            });
        },
        Err(err) => {
            show_message( term, MessageType::Error, Message {
                action: "Failed",
                details: "to create project directories",
            });
            return Err(err)
        }
    };
    Ok(())
}
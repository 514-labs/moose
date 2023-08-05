use console::{style, pad_str};

use super::CommandTerminal;

pub enum MessageType {
    Info,
    Success,
    Warning,
    Error,
}

pub struct Message<'a> {
    pub action: &'a str,
    pub details: &'a str,
}

// Prints a action & message to the terminal and increments the terminal line count by 1. 
// Actions should be things like "Adding", "Removing", "Updating", or a count of things to be done like [1/3].
// Message types control the color of the action text.
pub fn show_message(command_terminal: &mut CommandTerminal, message_type: MessageType, messsage: Message) {
    let padder = 14;
    
    match message_type {
        MessageType::Info => {
            command_terminal.term.write_line(&format!("{} {}", style(pad_str(messsage.action, padder, console::Alignment::Right, Some("..."))).blue().bold() , messsage.details)).expect("failed to write message to terminal");
            command_terminal.counter += 1;
        },
        MessageType::Success => {
            command_terminal.term.write_line(&format!("{} {}", style(pad_str(messsage.action, padder, console::Alignment::Right, Some("..."))).green().bold() , messsage.details)).expect("failed to write message to terminal");
            command_terminal.counter += 1;
        },  
        MessageType::Warning => {
            command_terminal.term.write_line(&format!("{} {}", style(pad_str(messsage.action, padder, console::Alignment::Right, Some("..."))).yellow().bold() , messsage.details)).expect("failed to write message to terminal");
            command_terminal.counter += 1;
        },
        MessageType::Error => {
            command_terminal.term.write_line(&format!("{} {}", style(pad_str(messsage.action, padder, console::Alignment::Right, Some("..."))).red().bold() , messsage.details)).expect("failed to write message to terminal");
            command_terminal.counter += 1;
        },
    };
    
}
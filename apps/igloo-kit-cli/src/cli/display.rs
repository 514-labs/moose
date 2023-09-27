use console::{style, pad_str};

use super::CommandTerminal;

pub enum MessageType {
    Info,
    Success,
    Warning,
    Error,
    Typographic,
    Banner
}

const TYPOGRAPHIC: &str = r#"
      ___         ___         ___         ___     
     /\__\       /\  \       /\  \       /\  \    
    /:/ _/_     /::\  \     /::\  \      \:\  \   
   /:/ /\  \   /:/\:\  \   /:/\:\  \      \:\  \  
  /:/ /::\  \ /:/  \:\  \ /:/  \:\  \ _____\:\  \ 
 /:/_/:/\:\__/:/__/ \:\__/:/__/ \:\__/::::::::\__\
 \:\/:/ /:/  \:\  \ /:/  \:\  \ /:/  \:\~~\~~\/__/
  \::/ /:/  / \:\  /:/  / \:\  /:/  / \:\  \      
   \/_/:/  /   \:\/:/  /   \:\/:/  /   \:\  \     
     /:/  /     \::/  /     \::/  /     \:\__\    
     \/__/       \/__/       \/__/       \/__/    
"#;


fn styled_banner() -> String { format!(r#"

---------------------------------------------------------------------------------------
{} 
We're simplifying how engineers build, deploy and maintain data-intensive applications 
with the first full-stack data-intensive framework.  

Join our community to keep up with our progress, contribute to igloo or join our team:
{}
---------------------------------------------------------------------------------------

"#, style("# Igloo is coming soon").bold(), style("https://discord.gg/WX3V3K4QCc").color256(118).bold())}


pub struct Message<'a> {
    pub action: &'a str,
    pub details: &'a str,
}
impl Message<'_> {
    pub fn new<'a>(action: &'a str, details: &'a str) -> Message<'a> {
        Message {
            action,
            details,
        }
    }
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
        MessageType::Typographic => {
            command_terminal.term.write_line(&format!("{TYPOGRAPHIC}")).expect("failed to write message to terminal");
            command_terminal.counter += TYPOGRAPHIC.lines().count();
        }
        MessageType::Banner => {
            command_terminal.term.write_line(&styled_banner()).expect("failed to write message to terminal");
            command_terminal.counter += styled_banner().lines().count();
        }
    };
    
}
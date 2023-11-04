use console::{pad_str, style};
use std::sync::{Arc, RwLock};

/// # Display Module
/// Standardizes the way we display messages to the user in the CLI
///
/// ## Module Usage
/// ```
/// use igloo::cli::display::{MessageType, Message, show_message};
/// use std::sync::{RwLock, Arc};
///
/// let term = Arc::new(RwLock::new(CommandTerminal::new());
/// show_message(term.clone(), MessageType::Info, Message {
///    action: "Loading Config".to_string(),
///   details: "Reading configuration from ~/.igloo-config.toml".to_string(),
/// });
/// ```
///
/// ## Command Terminal
/// The CommandTerminal struct is used to keep track of the number of lines that have been written to the
/// terminal and gives a handle on the terminal itself to run commands like clearing the terminal.
///
/// ### Usage
/// ```
/// let term = Arc::new(RwLock::new(CommandTerminal::new());
/// show_message(term.clone(), MessageType::Info, Message {
///   action: "Loading Config".to_string(),
///   details: "Reading configuration from ~/.igloo-config.toml".to_string(),
/// });
/// term.clear();
/// ```
///
///
/// ## Message Types
/// - Info: blue action text and white details text. Used for general information.
/// - Success: green action text and white details text. Used for successful actions.
/// - Warning: yellow action text and white details text. Used for warnings.
/// - Error: red action text and white details text. Used for errors.
/// - Typographic: large stylistic text. Used for a text displays.
/// - Banner: multi line text that's used to display a banner that should drive an action from the user
///
/// ## Message Struct
/// ```
/// Message {
///    action: "Loading Config".to_string(),
///    details: "Reading configuration from ~/.igloo-config.toml".to_string(),
/// }
/// ```
///
/// ## Suggested Improvements
/// - remove the need for users to use .to_string() on the action and details fields
/// - add a message type for a "waiting" message
/// - add a message type for a "loading" message with a progress bar
/// - remove the arc and rwlock from show_message and instead pass in a reference to the terminal
///

#[derive(Debug, Clone)]
pub struct CommandTerminal {
    term: console::Term,
    counter: usize,
}

impl CommandTerminal {
    pub fn new() -> CommandTerminal {
        CommandTerminal {
            term: console::Term::stdout(),
            counter: 0,
        }
    }

    pub fn clear(&mut self) {
        self.term
            .clear_last_lines(self.counter)
            .expect("failed to clear the terminal");
        self.counter = 0;
    }

    pub fn clear_with_delay(&mut self, delay_milli: u64) {
        std::thread::sleep(std::time::Duration::from_millis(delay_milli));
        self.clear();
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MessageType {
    Info,
    Success,
    Warning,
    Error,
    Typographic,
    Banner,
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

fn styled_banner() -> String {
    format!(
        r#"

---------------------------------------------------------------------------------------
{} 
We're simplifying how engineers build, deploy and maintain data-intensive applications 
with the first full-stack data-intensive framework.  

Join our community to keep up with our progress, contribute to igloo or join our team:
{}
---------------------------------------------------------------------------------------

"#,
        style("# Igloo is coming soon").bold(),
        style("https://discord.gg/WX3V3K4QCc").color256(118).bold()
    )
}

#[derive(Debug, Clone)]
pub struct Message {
    pub action: String,
    pub details: String,
}
impl Message {
    pub fn new(action: String, details: String) -> Message {
        Message { action, details }
    }
}

/// Prints a action & message to the terminal and increments the terminal line count by 1.
/// Actions should be things like "Adding", "Removing", "Updating", or a count of things to be done like [1/3].
/// Message types control the color of the action text.
pub fn show_message(
    term: Arc<RwLock<CommandTerminal>>,
    message_type: MessageType,
    messsage: Message,
) {
    let padder = 14;
    let mut command_terminal = term.write().unwrap();

    match message_type {
        MessageType::Info => {
            command_terminal
                .term
                .write_line(&format!(
                    "{} {}",
                    style(pad_str(
                        messsage.action.as_str(),
                        padder,
                        console::Alignment::Right,
                        Some("...")
                    ))
                    .blue()
                    .bold(),
                    messsage.details
                ))
                .expect("failed to write message to terminal");
            command_terminal.counter += 1;
        }
        MessageType::Success => {
            command_terminal
                .term
                .write_line(&format!(
                    "{} {}",
                    style(pad_str(
                        messsage.action.as_str(),
                        padder,
                        console::Alignment::Right,
                        Some("...")
                    ))
                    .green()
                    .bold(),
                    messsage.details
                ))
                .expect("failed to write message to terminal");
            command_terminal.counter += 1;
        }
        MessageType::Warning => {
            command_terminal
                .term
                .write_line(&format!(
                    "{} {}",
                    style(pad_str(
                        messsage.action.as_str(),
                        padder,
                        console::Alignment::Right,
                        Some("...")
                    ))
                    .yellow()
                    .bold(),
                    messsage.details
                ))
                .expect("failed to write message to terminal");
            command_terminal.counter += 1;
        }
        MessageType::Error => {
            command_terminal
                .term
                .write_line(&format!(
                    "{} {}",
                    style(pad_str(
                        messsage.action.as_str(),
                        padder,
                        console::Alignment::Right,
                        Some("...")
                    ))
                    .red()
                    .bold(),
                    messsage.details
                ))
                .expect("failed to write message to terminal");
            command_terminal.counter += 1;
        }
        MessageType::Typographic => {
            command_terminal
                .term
                .write_line(&format!("{TYPOGRAPHIC}"))
                .expect("failed to write message to terminal");
            command_terminal.counter += TYPOGRAPHIC.lines().count();
        }
        MessageType::Banner => {
            command_terminal
                .term
                .write_line(&styled_banner())
                .expect("failed to write message to terminal");
            command_terminal.counter += styled_banner().lines().count();
        }
    };
}

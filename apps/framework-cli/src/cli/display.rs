use console::style;
use lazy_static::lazy_static;
use spinners::{Spinner, Spinners};
use std::sync::{Arc, RwLock};
use tokio::macros::support::Future;

/// # Display Module
/// Standardizes the way we display messages to the user in the CLI. This module
/// provides a macro that takes a message type and a message struct and displays
/// the message to the user.
///
///
/// ### Usage
/// ```
/// show_message!(
///     MessageType::Info,
///     Message {
///         action: "Loading Config".to_string(),
///         details: "Reading configuration from ~/.moose/config.toml".to_string(),
///     });
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
///    details: "Reading configuration from ~/.moose/config.toml".to_string(),
/// }
/// ```
///
/// ## Suggested Improvements
/// - add a message type for a "waiting" message
/// - add a message type for a "loading" message with a progress bar
/// - add specific macros for each message type
/// - add a clear screen macro

#[derive(Debug, Clone)]
pub struct CommandTerminal {
    pub term: console::Term,
    pub counter: usize,
}

impl CommandTerminal {
    pub fn new() -> CommandTerminal {
        CommandTerminal {
            term: console::Term::stdout(),
            counter: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MessageType {
    Info,
    Success,
    Error,
    Banner,
}

pub fn styled_banner() -> String {
    format!(
        r#"

---------------------------------------------------------------------------------------
{} 
We're simplifying how engineers build, deploy and maintain data-intensive applications 
with the first full-stack data-intensive framework.  

Join our community to keep up with our progress, contribute to moose or join our team:
{}
---------------------------------------------------------------------------------------

"#,
        style("# Moose is coming soon").bold(),
        style("https://join.slack.com/t/igloocommunity/shared_invite/zt-25gsnx2x2-9ttVTt4L9LYFrRcM6jimcg").color256(118).bold()
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

lazy_static! {
    pub static ref TERM: Arc<RwLock<CommandTerminal>> =
        Arc::new(RwLock::new(CommandTerminal::new()));
}

macro_rules! show_message {
    ($message_type:expr, $message:expr) => {
        use crate::cli::display::styled_banner;
        use crate::cli::display::TERM;
        use console::{pad_str, style};

        let padder = 14;

        match $message_type {
            MessageType::Info => {
                let mut command_terminal = TERM.write().unwrap();
                command_terminal
                    .term
                    .write_line(&format!(
                        "{} {}",
                        style(pad_str(
                            $message.action.as_str(),
                            padder,
                            console::Alignment::Right,
                            Some("...")
                        ))
                        .blue()
                        .bold(),
                        $message.details
                    ))
                    .expect("failed to write message to terminal");
                command_terminal.counter += 1;
            }
            MessageType::Success => {
                let mut command_terminal = TERM.write().unwrap();
                command_terminal
                    .term
                    .write_line(&format!(
                        "{} {}",
                        style(pad_str(
                            $message.action.as_str(),
                            padder,
                            console::Alignment::Right,
                            Some("...")
                        ))
                        .green()
                        .bold(),
                        $message.details
                    ))
                    .expect("failed to write message to terminal");
                command_terminal.counter += 1;
            }
            MessageType::Error => {
                let mut command_terminal = TERM.write().unwrap();
                command_terminal
                    .term
                    .write_line(&format!(
                        "{} {}",
                        style(pad_str(
                            $message.action.as_str(),
                            padder,
                            console::Alignment::Right,
                            Some("...")
                        ))
                        .red()
                        .bold(),
                        $message.details
                    ))
                    .expect("failed to write message to terminal");
                command_terminal.counter += 1;
            }
            MessageType::Banner => {
                let mut command_terminal = TERM.write().unwrap();
                command_terminal
                    .term
                    .write_line(&styled_banner())
                    .expect("failed to write message to terminal");
                command_terminal.counter += styled_banner().lines().count();
            }
        };
    };
}

pub fn with_spinner<F, R>(message: &str, f: F) -> R
where
    F: FnOnce() -> R,
{
    let mut sp = Spinner::new(Spinners::Dots9, message.into());
    let res = f();
    sp.stop_with_newline();
    res
}

pub async fn with_spinner_async<F, R>(message: &str, f: F) -> R
where
F: Future<Output = R>,
{
    let mut sp = Spinner::new(Spinners::Dots9, message.into());

    let res = f.await;
    sp.stop_with_newline();
    res
}

#[cfg(test)]
mod tests {
    
    #[test]
    fn test_with_spinner() {
        use super::*;
        use crate::cli::routines::RoutineFailure;
        use std::time::Duration;
        use std::thread;

        let _ = with_spinner("Test delay for one second", || {
            thread::sleep(Duration::from_secs(1));
            Ok(())
        })
        .map_err(|err| {
            RoutineFailure::new(
                Message::new("Failed".to_string(), "to execute a delay".to_string()),
                err,
            )
        });
        show_message!(MessageType::Info, Message {
            action: "SUCCESS".to_string(),
            details: "Successfully executed a one second delay".to_string(),
        });
    }

    #[tokio::test]
    async fn simple_test_with_spinner_async() {
        use super::*;
        use crate::cli::routines::RoutineFailure;
        use tokio::time::{sleep, Duration};

        let _ = with_spinner_async("Test delay", async {
            sleep(Duration::from_secs(15)).await;
            Ok(())
        }).await
        .map_err(|err| {
            RoutineFailure::new(
                Message::new("Failed".to_string(), "to execute a delay".to_string()),
                err,
            )
        });
        show_message!(MessageType::Info, Message {
            action: "SUCCESS".to_string(),
            details: "Successfully executed a delay".to_string(),
        });
    }

    #[tokio::test]
    async fn command_test_with_spinner_async() {
        use super::*;
        use crate::cli::routines::RoutineFailure;
        use tokio::process::Command;
        use std::process::Stdio;

        let _ = with_spinner_async("Run docker ps command", async {
            let child = Command::new("docker")
                .arg("ps")
                .arg("-a")
                .arg("--no-trunc")
                .arg("--format")
                .arg("json")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;
    
            let output = child.wait_with_output().await?;

            if !output.status.success() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to list Docker containers",
                ));
            }
            let output_str = String::from_utf8_lossy(&output.stdout);
            print!("{}", output_str);
            Ok(())
        }).await
        .map_err(|err| {
            RoutineFailure::new(
                Message::new("Failed".to_string(), "to execute docker ps command".to_string()),
                err,
            )
        });
        show_message!(MessageType::Info, Message {
            action: "SUCCESS".to_string(),
            details: "Successfully executed docker ps command".to_string(),
        });
    }
}
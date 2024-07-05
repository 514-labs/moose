use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, ContentArrangement, Table};
use console::{pad_str, style};
use lazy_static::lazy_static;
use serde::Deserialize;
use spinners::{Spinner, Spinners};
use std::sync::{Arc, RwLock};
use tokio::macros::support::Future;

use crate::framework::core::{
    infrastructure_map::{ApiChange, Change, OlapChange, ProcessChange, StreamingChange},
    plan::InfraPlan,
};

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

#[derive(Deserialize, Debug, Clone, Copy)]
pub enum MessageType {
    Info,
    Success,
    Error,
    Highlight,
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

pub fn infra_added(message: &str) {
    let command_terminal = TERM.write().unwrap();
    let padder = 14;

    command_terminal
        .term
        .write_line(&format!(
            "{} {}",
            style(pad_str("+", padder, console::Alignment::Right, Some("..."))).green(),
            message
        ))
        .expect("failed to write message to terminal");
}

pub fn infra_removed(message: &str) {
    let command_terminal = TERM.write().unwrap();
    let padder = 14;

    command_terminal
        .term
        .write_line(&format!(
            "{} {}",
            style(pad_str("-", padder, console::Alignment::Right, Some("..."))).red(),
            message
        ))
        .expect("failed to write message to terminal");
}

pub fn infra_updated(message: &str) {
    let command_terminal = TERM.write().unwrap();
    let padder = 14;

    command_terminal
        .term
        .write_line(&format!(
            "{} {}",
            style(pad_str("~", padder, console::Alignment::Right, Some("..."))).yellow(),
            message
        ))
        .expect("failed to write message to terminal");
}

macro_rules! show_message {
    ($message_type:expr, $message:expr) => {
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
                        .cyan()
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
            MessageType::Highlight => {
                let mut command_terminal = TERM.write().unwrap();
                command_terminal
                    .term
                    .write_line(&format!(
                        "{} {}",
                        style(pad_str(
                            $message.action.as_str(),
                            padder,
                            console::Alignment::Center,
                            Some("...")
                        ))
                        .on_green()
                        .bold(),
                        style($message.details.as_str()).white().bright()
                    ))
                    .expect("failed to write message to terminal");
                command_terminal.counter += 1;
            }
        };
    };
}

pub fn with_spinner<F, R>(message: &str, f: F, activate: bool) -> R
where
    F: FnOnce() -> R,
{
    if !activate {
        return f();
    }

    let mut sp = Spinner::with_stream(Spinners::Dots9, message.into(), spinners::Stream::Stdout);
    let res = f();
    sp.stop_with_newline();
    res
}

pub async fn with_spinner_async<F, R>(message: &str, f: F, activate: bool) -> R
where
    F: Future<Output = R>,
{
    if !activate {
        return f.await;
    }

    let mut sp = Spinner::with_stream(Spinners::Dots9, message.into(), spinners::Stream::Stdout);
    let res = f.await;
    sp.stop_with_newline();
    res
}

pub fn show_table(headers: Vec<String>, rows: Vec<Vec<String>>) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS);
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(headers.into_iter().map(|s| s.to_string()));

    for row in rows {
        table.add_row(row);
    }

    show_message!(
        MessageType::Info,
        Message {
            action: "".to_string(),
            details: format!("\n{}", table),
        }
    );
}

pub fn show_changes(infra_plan: &InfraPlan) {
    TERM.write()
        .unwrap()
        .term
        .write_line("")
        .expect("failed to write message to terminal");
    // TODO there is probably a better way to do the following through
    // https://crates.io/crates/enum_dispatch or something similar
    infra_plan
        .changes
        .streaming_engine_changes
        .iter()
        .for_each(|change| match change {
            StreamingChange::Topic(Change::Added(infra)) => {
                infra_added(&infra.expanded_display());
            }
            StreamingChange::Topic(Change::Removed(infra)) => {
                infra_removed(&infra.short_display());
            }
            StreamingChange::Topic(Change::Updated { before, after: _ }) => {
                infra_updated(&before.expanded_display());
            }
        });

    infra_plan
        .changes
        .olap_changes
        .iter()
        .for_each(|change| match change {
            OlapChange::Table(Change::Added(infra)) => {
                infra_added(&infra.expanded_display());
            }
            OlapChange::Table(Change::Removed(infra)) => {
                infra_removed(&infra.short_display());
            }
            OlapChange::Table(Change::Updated { before, after: _ }) => {
                infra_updated(&before.expanded_display());
            }
        });

    infra_plan
        .changes
        .processes_changes
        .iter()
        .for_each(|change| match change {
            ProcessChange::TopicToTableSyncProcess(Change::Added(infra)) => {
                infra_added(&infra.expanded_display());
            }
            ProcessChange::TopicToTableSyncProcess(Change::Removed(infra)) => {
                infra_removed(&infra.short_display());
            }
            ProcessChange::TopicToTableSyncProcess(Change::Updated { before, after: _ }) => {
                infra_updated(&before.expanded_display());
            }
            ProcessChange::FunctionProcess(Change::Added(infra)) => {
                infra_added(&infra.expanded_display());
            }
            ProcessChange::FunctionProcess(Change::Removed(infra)) => {
                infra_removed(&infra.short_display());
            }
            ProcessChange::FunctionProcess(Change::Updated { before, after: _ }) => {
                infra_updated(&before.expanded_display());
            }
            ProcessChange::OlapProcess(Change::Added(infra)) => {
                infra_added(&infra.expanded_display());
            }
            ProcessChange::OlapProcess(Change::Removed(infra)) => {
                infra_added(&infra.expanded_display());
            }
            ProcessChange::OlapProcess(Change::Updated { before, after: _ }) => {
                infra_updated(&before.expanded_display());
            }
            ProcessChange::ConsumptionApiWebServer(Change::Added(_)) => {
                infra_added("Starting Consumption WebServer...");
            }
            ProcessChange::ConsumptionApiWebServer(Change::Removed(_)) => {
                infra_removed("Stoping Consumption WebServer...");
            }
            ProcessChange::ConsumptionApiWebServer(Change::Updated {
                before: _,
                after: _,
            }) => {
                infra_updated("Reloading Consumption WebServer...");
            }
        });

    infra_plan
        .changes
        .api_changes
        .iter()
        .for_each(|change| match change {
            ApiChange::ApiEndpoint(Change::Added(infra)) => {
                infra_added(&infra.expanded_display());
            }
            ApiChange::ApiEndpoint(Change::Removed(infra)) => {
                infra_removed(&infra.short_display());
            }
            ApiChange::ApiEndpoint(Change::Updated { before, after: _ }) => {
                infra_updated(&before.expanded_display());
            }
        });
}

#[cfg(test)]
mod tests {
    use crate::cli::routines::RoutineFailure;

    #[test]
    fn test_with_spinner() {
        use super::*;
        use std::thread;
        use std::time::Duration;

        let _ = with_spinner(
            "Test delay for one second",
            || {
                thread::sleep(Duration::from_secs(1));
                Ok(())
            },
            true,
        )
        .map_err(|err: std::io::Error| {
            RoutineFailure::new(
                Message::new("Failed".to_string(), "to execute a delay".to_string()),
                err,
            )
        });
        show_message!(
            MessageType::Info,
            Message {
                action: "SUCCESS".to_string(),
                details: "Successfully executed a one second delay".to_string(),
            }
        );
    }

    #[tokio::test]
    async fn simple_test_with_spinner_async() -> Result<(), RoutineFailure> {
        use super::*;
        use crate::cli::routines::RoutineFailure;
        use tokio::time::{sleep, Duration};

        let result = with_spinner_async(
            "Test delay",
            async {
                sleep(Duration::from_secs(15)).await;
                Ok(())
            },
            true,
        )
        .await
        .map_err(|err: std::io::Error| {
            RoutineFailure::new(
                Message::new("Failed".to_string(), "to execute a delay".to_string()),
                err,
            )
        });
        show_message!(
            MessageType::Info,
            Message {
                action: "SUCCESS".to_string(),
                details: "Successfully executed a delay".to_string(),
            }
        );
        result
    }
}

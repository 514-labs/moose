use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, ContentArrangement, Table};
use console::{pad_str, style};
use lazy_static::lazy_static;
use log::info;
use serde::Deserialize;
use spinners::{Spinner, Spinners};
use std::sync::{Arc, RwLock};
use tokio::macros::support::Future;

use crate::framework::core::infrastructure_map::TableChange;
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
///
/// A terminal instance with associated state for displaying CLI messages.
///
/// The CommandTerminal wraps a console::Term for stdout handling and tracks the
/// number of messages written for potential future needs (such as clearing or
/// updating specific messages).
#[derive(Debug, Clone)]
pub struct CommandTerminal {
    /// The underlying terminal output handler
    pub term: console::Term,
    /// Counter tracking the number of messages written to the terminal
    pub counter: usize,
}

impl CommandTerminal {
    /// Creates a new CommandTerminal instance for stdout.
    ///
    /// # Returns
    ///
    /// * `CommandTerminal` - A new terminal instance with counter initialized to 0
    pub fn new() -> CommandTerminal {
        CommandTerminal {
            term: console::Term::stdout(),
            counter: 0,
        }
    }
}

/// Types of messages that can be displayed to the user.
///
/// Each message type corresponds to a different visual style in the terminal
/// to help distinguish between different kinds of information.
#[derive(Deserialize, Debug, Clone, Copy)]
pub enum MessageType {
    /// Used for general information with cyan highlighting
    Info,
    /// Used for successful actions with green highlighting
    Success,
    /// Used for errors with red highlighting
    Error,
    /// Used for important information with green background
    Highlight,
}

/// A message to be displayed to the user in the CLI.
///
/// Messages consist of an action (displayed with color based on MessageType)
/// and details (the main content of the message).
#[derive(Debug, Clone)]
pub struct Message {
    /// The action or category of the message, displayed with color
    pub action: String,
    /// The main content or details of the message
    pub details: String,
}

impl Message {
    /// Creates a new Message with the specified action and details.
    ///
    /// # Arguments
    ///
    /// * `action` - The action or category of the message
    /// * `details` - The main content or details of the message
    ///
    /// # Returns
    ///
    /// * `Message` - A new message instance
    pub fn new(action: String, details: String) -> Message {
        Message { action, details }
    }
}

lazy_static! {
    pub static ref TERM: Arc<RwLock<CommandTerminal>> =
        Arc::new(RwLock::new(CommandTerminal::new()));
}

/// Displays a message about infrastructure being added.
///
/// # Arguments
///
/// * `message` - The message describing the added infrastructure
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

    info!("+ {}", message.trim());
}

/// Displays a message about infrastructure being removed.
///
/// # Arguments
///
/// * `message` - The message describing the removed infrastructure
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

    info!("- {}", message.trim());
}

/// Displays a message about infrastructure being updated.
///
/// # Arguments
///
/// * `message` - The message describing the updated infrastructure
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

    info!("~ {}", message.trim());
}

macro_rules! show_message {
    (@inner $message_type:expr, $message:expr, $log:expr) => {
        use crate::cli::display::TERM;
        use console::{pad_str, style};

        let padder = 14;
        let action = $message.action.clone();
        let details = $message.details.clone();

        {
            let mut command_terminal = TERM.write().unwrap();
            let color = match $message_type {
                MessageType::Info => style(pad_str(action.as_str(), padder, console::Alignment::Right, Some("..."))).cyan().bold(),
                MessageType::Success => style(pad_str(action.as_str(), padder, console::Alignment::Right, Some("..."))).green().bold(),
                MessageType::Error => style(pad_str(action.as_str(), padder, console::Alignment::Right, Some("..."))).red().bold(),
                MessageType::Highlight => style(pad_str(action.as_str(), padder, console::Alignment::Center, Some("..."))).on_green().bold(),
            };

            command_terminal
                .term
                .write_line(&format!("{} {}", color, details))
                .expect("failed to write message to terminal");
            command_terminal.counter += 1;

            if $log {
                use log::info;

                let log_action = action.replace("\n", " ");
                let log_details = details.replace("\n", " ");
                info!("{}", format!("{} {}", log_action.trim(), log_details.trim()));
            }
        }
    };

    ($message_type:expr, $message:expr) => {
        show_message!(@inner $message_type, $message, true);
    };

    // Print message to terminal, but don't output to log file
    // i.e. moose logs (so it doesn't recursively log)
    ($message_type:expr, $message:expr, $no_log:expr) => {
        show_message!(@inner $message_type, $message, false);
    };
}

/// Executes a function with a spinner displayed during execution.
///
/// # Arguments
///
/// * `message` - The message to display alongside the spinner
/// * `f` - The function to execute
/// * `activate` - Whether to actually show the spinner (if false, just runs the function)
///
/// # Returns
///
/// * The result of the function execution
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

/// Executes an asynchronous function with a spinner displayed during execution.
///
/// # Arguments
///
/// * `message` - The message to display alongside the spinner
/// * `f` - The async function to execute
/// * `activate` - Whether to actually show the spinner (if false, just runs the function)
///
/// # Returns
///
/// * The result of the async function execution
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

/// Displays a table with headers and rows.
///
/// # Arguments
///
/// * `headers` - The column headers for the table
/// * `rows` - The data rows for the table
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

/// Wrapper for the show_message macro to allow calling from non-macro contexts.
///
/// # Arguments
///
/// * `message_type` - The type of message to display
/// * `message` - The message to display
pub fn show_message_wrapper(message_type: MessageType, message: Message) {
    // Probably shouldn't do macro_export so we just wrap it
    show_message!(message_type, message, false);
}

/// Displays a message about a batch being inserted into a database table.
///
/// # Arguments
///
/// * `count` - The number of rows inserted
/// * `table_name` - The name of the table rows were inserted into
pub fn batch_inserted(count: usize, table_name: &str) {
    show_message!(
        MessageType::Info,
        Message {
            action: "[DB]".to_string(),
            details: format!("{count} row(s) successfully written to DB table ({table_name})"),
        }
    );
}

/// Displays OLAP changes (tables and views) to the user.
///
/// # Arguments
///
/// * `olap_changes` - The vector of OLAP changes to display
pub fn show_olap_changes(olap_changes: &[OlapChange]) {
    olap_changes.iter().for_each(|change| match change {
        OlapChange::Table(TableChange::Added(infra)) => {
            infra_added(&infra.expanded_display());
        }
        OlapChange::Table(TableChange::Removed(infra)) => {
            infra_removed(&infra.short_display());
        }
        OlapChange::Table(TableChange::Updated {
            name,
            column_changes,
            order_by_change,
            before,
            after,
        }) => {
            if after.deduplicate != before.deduplicate {
                infra_removed(&before.expanded_display());
                infra_added(&after.expanded_display());
            } else {
                infra_updated(&format!(
                    "Table {} with column changes: {:?} and order by changes: {:?}",
                    name, column_changes, order_by_change
                ));
            }
        }
        OlapChange::View(Change::Added(infra)) => {
            infra_added(&infra.expanded_display());
        }
        OlapChange::View(Change::Removed(infra)) => {
            infra_removed(&infra.short_display());
        }
        OlapChange::View(Change::Updated { before, after: _ }) => {
            infra_updated(&before.expanded_display());
        }
        OlapChange::SqlResource(Change::Added(sql_resource)) => {
            infra_added(&format!("SQL Resource: {}", sql_resource.name));
        }
        OlapChange::SqlResource(Change::Removed(sql_resource)) => {
            infra_removed(&format!("SQL Resource: {}", sql_resource.name));
        }
        OlapChange::SqlResource(Change::Updated { before, after: _ }) => {
            infra_updated(&format!("SQL Resource: {}", before.name));
        }
    });
}

/// Displays all infrastructure changes from an InfraPlan to the user.
///
/// Shows changes to streaming, OLAP, processes, and API endpoints.
///
/// # Arguments
///
/// * `infra_plan` - The infrastructure plan containing all changes
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
            StreamingChange::Topic(Change::Updated { before: _, after }) => {
                infra_updated(&after.expanded_display());
            }
        });

    show_olap_changes(&infra_plan.changes.olap_changes);

    infra_plan
        .changes
        .processes_changes
        .iter()
        .for_each(|change| match change {
            ProcessChange::TopicToTopicSyncProcess(Change::Added(infra)) => {
                infra_added(&infra.expanded_display());
            }
            ProcessChange::TopicToTopicSyncProcess(Change::Removed(infra)) => {
                infra_removed(&infra.short_display());
            }
            ProcessChange::TopicToTopicSyncProcess(Change::Updated { before, after: _ }) => {
                infra_updated(&before.expanded_display());
            }
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
                infra_removed("Stopping Consumption WebServer...");
            }
            ProcessChange::ConsumptionApiWebServer(Change::Updated { .. }) => {
                infra_updated("Reloading Consumption WebServer...");
            }
            ProcessChange::OrchestrationWorker(Change::Added(_)) => {
                infra_added("Starting Orchestration worker...");
            }
            ProcessChange::OrchestrationWorker(Change::Removed(_)) => {
                infra_removed("Stopping Orchestration worker...");
            }
            ProcessChange::OrchestrationWorker(Change::Updated {
                before: _,
                after: _,
            }) => {
                infra_updated("Reloading Orchestration worker...");
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

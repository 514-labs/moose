use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, ContentArrangement, Table};
use log::info;
use serde::Deserialize;
use std::io::{stdout, IsTerminal};
use std::sync::Arc;
use tokio::macros::support::Future;

// Crossterm imports for migration
use crossterm::style::{
    Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
};

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
/// The CommandTerminal wraps stdout handling and tracks the
/// number of messages written for potential future needs (such as clearing or
/// updating specific messages).

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

/// Displays a message about infrastructure being added.
///
/// # Arguments
///
/// * `message` - The message describing the added infrastructure
pub fn infra_added(message: &str) {
    let styled_text = crossterm_utils::StyledText::new("+ ".to_string()).green();
    // Use the new styled message writer for clean formatting
    crossterm_utils::write_styled_line(&styled_text, message)
        .expect("failed to write message to terminal");

    info!("+ {}", message.trim());
}

/// Displays a message about infrastructure being removed.
///
/// # Arguments
///
/// * `message` - The message describing the removed infrastructure
pub fn infra_removed(message: &str) {
    let styled_text = crossterm_utils::StyledText::new("- ".to_string()).red();

    // Use the new styled message writer for clean formatting
    crossterm_utils::write_styled_line(&styled_text, message)
        .expect("failed to write message to terminal");

    info!("- {}", message.trim());
}

/// Displays a message about infrastructure being updated.
///
/// # Arguments
///
/// * `message` - The message describing the updated infrastructure
pub fn infra_updated(message: &str) {
    let styled_text = crossterm_utils::StyledText::new("~ ".to_string()).yellow();

    // Use the new styled message writer for clean formatting
    crossterm_utils::write_styled_line(&styled_text, message)
        .expect("failed to write message to terminal");

    info!("~ {}", message.trim());
}

macro_rules! show_message {
    (@inner $message_type:expr, $message:expr, $log:expr) => {
        use crate::cli::display::crossterm_utils;

        let evaluated_message = $message;
        let action = evaluated_message.action.clone();
        let details = evaluated_message.details.clone();

        {

            // Create styled prefix based on message type
            let styled_prefix = match $message_type {
                MessageType::Info => crossterm_utils::StyledText::new(action.clone()).cyan().bold(),
                MessageType::Success => crossterm_utils::StyledText::new(action.clone()).green().bold(),
                MessageType::Error => crossterm_utils::StyledText::new(action.clone()).red().bold(),
                MessageType::Highlight => crossterm_utils::StyledText::new(action.clone()).on_green().bold(),
            };

            // Write styled prefix and details in one line
            crossterm_utils::write_styled_line(&styled_prefix, &details)
                .expect("failed to write message to terminal");

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
/// * `activate` - Whether to actually show the spinner (if false or not terminal, just runs the function)
///
/// # Returns
///
/// * The result of the function execution
pub fn with_spinner<F, R>(message: &str, f: F, activate: bool) -> R
where
    F: FnOnce() -> R,
{
    let sp = if activate && stdout().is_terminal() {
        use crossterm_utils::TerminalComponent;
        let mut spinner = crossterm_utils::SpinnerComponent::new(message);
        let _ = spinner.start();
        Some(spinner)
    } else {
        None
    };

    let res = f();

    if let Some(mut spinner) = sp {
        use crossterm_utils::TerminalComponent;
        let _ = spinner.stop();
        let _ = spinner.cleanup();
    }

    res
}

/// Executes an asynchronous function with a spinner displayed during execution.
///
/// # Arguments
///
/// * `message` - The message to display alongside the spinner
/// * `f` - The async function to execute
/// * `activate` - Whether to actually show the spinner (if false or not terminal, just runs the function)
///
/// # Returns
///
/// * The result of the async function execution
pub async fn with_spinner_async<F, R>(message: &str, f: F, activate: bool) -> R
where
    F: Future<Output = R>,
{
    let sp = if activate && stdout().is_terminal() {
        use crossterm_utils::TerminalComponent;
        let mut spinner = crossterm_utils::SpinnerComponent::new(message);
        let _ = spinner.start();
        Some(spinner)
    } else {
        None
    };

    let res = f.await;

    if let Some(mut spinner) = sp {
        use crossterm_utils::TerminalComponent;
        let _ = spinner.stop();
    }

    res
}

/// Displays a table with headers and rows.
///
/// # Arguments
///
/// * `headers` - The column headers for the table
/// * `rows` - The data rows for the table
pub fn show_table(title: String, headers: Vec<String>, rows: Vec<Vec<String>>) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS);
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(headers);

    for row in rows {
        table.add_row(row);
    }

    println!("{title}\n{table}");
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
                    "Table {name} with column changes: {column_changes:?} and order by changes: {order_by_change:?}"
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

/// Crossterm utility functions to replace console crate functionality
pub mod crossterm_utils {
    use super::*;
    use crossterm::execute;

    /// Base trait for terminal output components
    /// Each component manages its own terminal state and cleanup
    pub trait TerminalComponent {
        /// Start the component (display initial state)
        fn start(&mut self) -> std::io::Result<()>;

        /// Stop the component and clean up terminal state
        fn stop(&mut self) -> std::io::Result<()>;

        /// Ensure terminal is ready for the next component
        fn cleanup(&mut self) -> std::io::Result<()> {
            // Default implementation - print newline to ensure next component starts fresh
            use crossterm::{execute, style::Print};
            execute!(std::io::stdout(), Print("\n"))?;
            Ok(())
        }
    }

    /// Ephemeral spinner component that disappears when done
    pub struct SpinnerComponent {
        message: String,
        handle: Option<std::thread::JoinHandle<()>>,
        stop_signal: Arc<std::sync::atomic::AtomicBool>,
        started: bool,
    }

    impl SpinnerComponent {
        pub fn new(message: &str) -> Self {
            Self {
                message: message.to_string(),
                handle: None,
                stop_signal: Arc::new(std::sync::atomic::AtomicBool::new(false)),
                started: false,
            }
        }
    }

    impl TerminalComponent for SpinnerComponent {
        fn start(&mut self) -> std::io::Result<()> {
            if self.started {
                return Ok(());
            }

            use std::sync::atomic::Ordering;
            use std::time::Duration;

            // Dots9 animation frames
            const DOTS9_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

            let message = self.message.clone();
            let stop_signal = self.stop_signal.clone();

            // Start on a fresh line using crossterm execute!
            execute!(
                std::io::stdout(),
                Print(&format!("{} {}", DOTS9_FRAMES[0], message))
            )?;

            self.handle = Some(std::thread::spawn(move || {
                let mut frame_index = 0;
                while !stop_signal.load(Ordering::Relaxed) {
                    std::thread::sleep(Duration::from_millis(80));

                    if !stop_signal.load(Ordering::Relaxed) {
                        // Update spinner on same line using crossterm
                        let _ = execute!(
                            std::io::stdout(),
                            Print(&format!("\r{} {}", DOTS9_FRAMES[frame_index], message))
                        );
                        frame_index = (frame_index + 1) % DOTS9_FRAMES.len();
                    }
                }
            }));

            self.started = true;
            Ok(())
        }

        fn stop(&mut self) -> std::io::Result<()> {
            if !self.started {
                return Ok(());
            }

            use std::sync::atomic::Ordering;

            // Signal the thread to stop
            self.stop_signal.store(true, Ordering::Relaxed);

            // Wait for the thread to finish gracefully
            if let Some(handle) = self.handle.take() {
                // Try to join the thread with a reasonable timeout
                let _ = std::thread::spawn(move || {
                    let _ = handle.join();
                });

                // Give the thread time to clean up naturally
                std::thread::sleep(std::time::Duration::from_millis(50));

                // Don't wait for the cleanup thread to finish - just let it complete in background
            }

            // Clean up the current spinner line gracefully
            execute!(
                std::io::stdout(),
                crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine),
                crossterm::cursor::MoveToColumn(0)
            )?;

            self.started = false;
            Ok(())
        }

        fn cleanup(&mut self) -> std::io::Result<()> {
            // SpinnerComponent disappears completely, so no newline needed
            // Terminal is already clean at the beginning of the line
            Ok(())
        }
    }

    impl Drop for SpinnerComponent {
        fn drop(&mut self) {
            // Gracefully clean up if needed, but don't force aggressive cleanup
            if self.started {
                let _ = self.stop();
            }
        }
    }

    /// Permanent message component that stays in the output
    pub struct MessageComponent {
        styled_text: StyledText,
        message: String,
        displayed: bool,
    }

    impl TerminalComponent for MessageComponent {
        fn start(&mut self) -> std::io::Result<()> {
            if self.displayed {
                return Ok(());
            }

            // Write the styled message (this stays in the output)
            write_styled_line(&self.styled_text, &self.message)?;
            self.displayed = true;
            Ok(())
        }

        fn stop(&mut self) -> std::io::Result<()> {
            // Messages don't need to be stopped - they stay in output
            Ok(())
        }

        fn cleanup(&mut self) -> std::io::Result<()> {
            // Messages already end with newline, terminal is ready for next component
            Ok(())
        }
    }

    /// Styled text builder for crossterm
    pub struct StyledText {
        text: String,
        foreground: Option<Color>,
        background: Option<Color>,
        bold: bool,
    }

    impl StyledText {
        pub fn new(text: String) -> Self {
            Self {
                text,
                foreground: None,
                background: None,
                bold: false,
            }
        }

        pub fn cyan(mut self) -> Self {
            self.foreground = Some(Color::Cyan);
            self
        }

        pub fn green(mut self) -> Self {
            self.foreground = Some(Color::Green);
            self
        }

        pub fn yellow(mut self) -> Self {
            self.foreground = Some(Color::Yellow);
            self
        }

        pub fn red(mut self) -> Self {
            self.foreground = Some(Color::Red);
            self
        }

        pub fn bold(mut self) -> Self {
            self.bold = true;
            self
        }

        pub fn on_green(mut self) -> Self {
            self.background = Some(Color::Green);
            self
        }
    }

    const ACTION_WIDTH: usize = 15;

    /// Write a styled line in one operation with fixed 12-character right-aligned action
    pub fn write_styled_line(styled_text: &StyledText, message: &str) -> std::io::Result<()> {
        let mut stdout = std::io::stdout();

        // Ensure action is exactly 12 characters, right-aligned
        let truncated_action = if styled_text.text.len() > ACTION_WIDTH {
            &styled_text.text[..ACTION_WIDTH]
        } else {
            &styled_text.text
        };
        let padded_action = format!("{truncated_action:>ACTION_WIDTH$}");

        // Apply foreground color
        if let Some(color) = styled_text.foreground {
            execute!(stdout, SetForegroundColor(color))?;
        }

        // Apply background color
        if let Some(color) = styled_text.background {
            execute!(stdout, SetBackgroundColor(color))?;
        }

        // Apply bold
        if styled_text.bold {
            execute!(stdout, SetAttribute(Attribute::Bold))?;
        }

        // Write the styled, right-aligned action text
        execute!(stdout, Print(&padded_action))?;

        // Reset styling before writing the message
        execute!(stdout, ResetColor)?;
        if styled_text.bold {
            execute!(stdout, SetAttribute(Attribute::Reset))?;
        }

        // Write separator and message
        execute!(stdout, Print(" "), Print(message), Print("\n"))?;

        Ok(())
    }
}

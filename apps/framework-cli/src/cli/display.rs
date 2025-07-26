//! # Display Module
//!
//! This module provides a standardized interface for displaying messages and information
//! to users in the CLI. It handles terminal output with consistent formatting, colors,
//! and interactive elements like spinners.
//!
//! The module is built around the concept of different message types that correspond
//! to different visual styles, helping users quickly identify the nature of information
//! being displayed.
//!
//! ## Core Components
//!
//! - **Message Types**: Categorized message styles (Info, Success, Error, Highlight)
//! - **Message Display**: Unified message formatting with color coding
//! - **Infrastructure Changes**: Specialized display for infrastructure plan changes
//! - **Interactive Elements**: Spinners for long-running operations
//! - **Table Display**: Formatted table output for structured data
//!
//! ## Usage Examples
//!
//! ### Basic Message Display
//! ```rust
//! use crate::cli::display::{show_message, MessageType, Message};
//!
//! show_message!(
//!     MessageType::Info,
//!     Message::new(
//!         "Loading Config".to_string(),
//!         "Reading configuration from ~/.moose/config.toml".to_string()
//!     )
//! );
//! ```
//!
//! ### Using Spinners
//! ```rust
//! use crate::cli::display::with_spinner;
//!
//! let result = with_spinner("Processing data", || {
//!     // Long-running operation
//!     expensive_computation()
//! }, true);
//! ```
//!
//! ### Infrastructure Change Display
//! ```rust
//! use crate::cli::display::show_changes;
//!
//! show_changes(&infrastructure_plan);
//! ```

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

/// Types of messages that can be displayed to the user.
///
/// Each message type corresponds to a different visual style in the terminal
/// to help distinguish between different kinds of information. The styling
/// affects the color and formatting of the action portion of messages.
///
/// # Visual Styles
///
/// - `Info`: Cyan text with bold formatting for general information
/// - `Success`: Green text with bold formatting for successful operations
/// - `Error`: Red text with bold formatting for error conditions
/// - `Highlight`: Text with green background and bold formatting for emphasis
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
/// Messages consist of two parts:
/// - An action (displayed with color based on MessageType) that appears in a fixed-width column
/// - Details (the main content) that contains the descriptive text
///
/// The action is right-aligned in a 15-character column to provide consistent
/// visual alignment across different message types.
///
/// # Examples
///
/// ```rust
/// use crate::cli::display::Message;
///
/// let message = Message::new(
///     "Config".to_string(),
///     "Successfully loaded configuration file".to_string()
/// );
/// ```
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
    /// * `action` - The action or category of the message (displayed in colored column)
    /// * `details` - The main content or details of the message
    ///
    /// # Returns
    ///
    /// A new `Message` instance ready for display
    ///
    /// # Examples
    ///
    /// ```rust
    /// let msg = Message::new("Deploy".to_string(), "Application deployed successfully".to_string());
    /// ```
    pub fn new(action: String, details: String) -> Message {
        Message { action, details }
    }
}

/// Displays a message about infrastructure being added.
///
/// Uses green styling with a "+" prefix to indicate addition.
/// The message is both displayed to the terminal and logged.
///
/// # Arguments
///
/// * `message` - The message describing the added infrastructure
///
/// # Examples
///
/// ```rust
/// infra_added("Database table 'users' created");
/// ```
pub fn infra_added(message: &str) {
    let styled_text = crossterm_utils::StyledText::new("+ ".to_string()).green();
    // Use the new styled message writer for clean formatting
    crossterm_utils::write_styled_line(&styled_text, message)
        .expect("failed to write message to terminal");

    info!("+ {}", message.trim());
}

/// Displays a message about infrastructure being removed.
///
/// Uses red styling with a "-" prefix to indicate removal.
/// The message is both displayed to the terminal and logged.
///
/// # Arguments
///
/// * `message` - The message describing the removed infrastructure
///
/// # Examples
///
/// ```rust
/// infra_removed("Database table 'temp_data' dropped");
/// ```
pub fn infra_removed(message: &str) {
    let styled_text = crossterm_utils::StyledText::new("- ".to_string()).red();

    // Use the new styled message writer for clean formatting
    crossterm_utils::write_styled_line(&styled_text, message)
        .expect("failed to write message to terminal");

    info!("- {}", message.trim());
}

/// Displays a message about infrastructure being updated.
///
/// Uses yellow styling with a "~" prefix to indicate modification.
/// The message is both displayed to the terminal and logged.
///
/// # Arguments
///
/// * `message` - The message describing the updated infrastructure
///
/// # Examples
///
/// ```rust
/// infra_updated("Database table 'users' schema modified");
/// ```
pub fn infra_updated(message: &str) {
    let styled_text = crossterm_utils::StyledText::new("~ ".to_string()).yellow();

    // Use the new styled message writer for clean formatting
    crossterm_utils::write_styled_line(&styled_text, message)
        .expect("failed to write message to terminal");

    info!("~ {}", message.trim());
}

/// Macro for displaying styled messages to the terminal.
///
/// This macro provides a unified interface for displaying messages with consistent
/// formatting and optional logging. It handles the styling based on message type
/// and ensures proper terminal output.
///
/// # Syntax
///
/// - `show_message!(message_type, message)` - Display and log the message
/// - `show_message!(message_type, message, no_log)` - Display only, skip logging
///
/// # Arguments
///
/// * `message_type` - A `MessageType` enum value determining the visual style
/// * `message` - A `Message` struct containing the action and details
/// * `no_log` - Optional boolean to disable logging (when present, logging is skipped)
///
/// # Examples
///
/// ```rust
/// // Standard usage with logging
/// show_message!(
///     MessageType::Success,
///     Message::new("Deploy".to_string(), "Application deployed successfully".to_string())
/// );
///
/// // Skip logging (useful for log output itself)
/// show_message!(
///     MessageType::Info,
///     Message::new("Log".to_string(), "Debug information".to_string()),
///     true
/// );
/// ```
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
/// This function provides visual feedback for long-running operations by displaying
/// an animated spinner. The spinner automatically disappears when the operation
/// completes, leaving the terminal clean for subsequent output.
///
/// # Arguments
///
/// * `message` - The message to display alongside the spinner
/// * `f` - The function to execute while the spinner is active
/// * `activate` - Whether to actually show the spinner (false or non-terminal disables spinner)
///
/// # Returns
///
/// The result of the function execution, unchanged
///
/// # Behavior
///
/// - If `activate` is false or stdout is not a terminal, the function runs without spinner
/// - The spinner uses a dots animation and updates every 80ms
/// - Terminal state is properly cleaned up after completion
///
/// # Examples
///
/// ```rust
/// let result = with_spinner("Loading configuration", || {
///     load_config_file()
/// }, true);
/// ```
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
/// This is the async version of `with_spinner`, providing the same visual feedback
/// for long-running async operations.
///
/// # Arguments
///
/// * `message` - The message to display alongside the spinner
/// * `f` - The async function to execute while the spinner is active
/// * `activate` - Whether to actually show the spinner (false or non-terminal disables spinner)
///
/// # Returns
///
/// The result of the async function execution, unchanged
///
/// # Examples
///
/// ```rust
/// let result = with_spinner_async("Fetching data", async {
///     fetch_remote_data().await
/// }, true).await;
/// ```
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

/// Displays a formatted table with headers and data rows.
///
/// Creates a visually appealing table using UTF-8 characters with rounded corners
/// and full borders. The table automatically adjusts column widths based on content.
///
/// # Arguments
///
/// * `title` - The title to display above the table
/// * `headers` - Column headers for the table
/// * `rows` - Data rows, where each row is a vector of column values
///
/// # Examples
///
/// ```rust
/// show_table(
///     "User Data".to_string(),
///     vec!["ID".to_string(), "Name".to_string(), "Email".to_string()],
///     vec![
///         vec!["1".to_string(), "Alice".to_string(), "alice@example.com".to_string()],
///         vec!["2".to_string(), "Bob".to_string(), "bob@example.com".to_string()],
///     ]
/// );
/// ```
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

/// Wrapper function for the show_message macro.
///
/// This function allows calling the show_message functionality from contexts
/// where macros cannot be used directly. Messages displayed through this
/// function are not logged to avoid potential recursion in logging contexts.
///
/// # Arguments
///
/// * `message_type` - The type of message to display
/// * `message` - The message to display
///
/// # Examples
///
/// ```rust
/// show_message_wrapper(
///     MessageType::Error,
///     Message::new("Failed".to_string(), "Operation could not be completed".to_string())
/// );
/// ```
pub fn show_message_wrapper(message_type: MessageType, message: Message) {
    // Probably shouldn't do macro_export so we just wrap it
    show_message!(message_type, message, false);
}

/// Displays a message about a batch database insertion.
///
/// This function provides standardized messaging for database operations,
/// showing the number of rows inserted and the target table name.
///
/// # Arguments
///
/// * `count` - The number of rows successfully inserted
/// * `table_name` - The name of the database table
///
/// # Examples
///
/// ```rust
/// batch_inserted(150, "user_events");
/// // Displays: "[DB] 150 row(s) successfully written to DB table (user_events)"
/// ```
pub fn batch_inserted(count: usize, table_name: &str) {
    show_message!(
        MessageType::Info,
        Message {
            action: "[DB]".to_string(),
            details: format!("{count} row(s) successfully written to DB table ({table_name})"),
        }
    );
}

/// Displays OLAP (Online Analytical Processing) infrastructure changes.
///
/// This function handles the display of changes to OLAP components including
/// tables, views, and SQL resources. Each change type is displayed with
/// appropriate styling and formatting.
///
/// # Arguments
///
/// * `olap_changes` - A slice of OLAP changes to display
///
/// # Change Types Handled
///
/// - **Table Changes**: Added, removed, or updated database tables
/// - **View Changes**: Added, removed, or updated database views  
/// - **SQL Resource Changes**: Added, removed, or updated SQL resources
///
/// # Examples
///
/// ```rust
/// show_olap_changes(&infrastructure_plan.changes.olap_changes);
/// ```
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

/// Displays all infrastructure changes from an InfraPlan.
///
/// This function provides a comprehensive display of all infrastructure changes
/// including streaming engine changes, OLAP changes, process changes, and API changes.
/// Each category of changes is displayed with appropriate formatting and styling.
///
/// # Arguments
///
/// * `infra_plan` - The infrastructure plan containing all changes to display
///
/// # Change Categories Displayed
///
/// - **Streaming Changes**: Topics and streaming infrastructure
/// - **OLAP Changes**: Tables, views, and SQL resources (via `show_olap_changes`)
/// - **Process Changes**: Various process types including sync processes, functions, and web servers
/// - **API Changes**: API endpoints and related infrastructure
///
/// # Examples
///
/// ```rust
/// show_changes(&infrastructure_plan);
/// ```
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

/// Crossterm utility functions and components for terminal output.
///
/// This module provides low-level terminal manipulation utilities using the crossterm
/// crate. It includes components for displaying spinners, styled text, and managing
/// terminal state during CLI operations.
///
/// # Components
///
/// - **StyledText**: Builder for creating styled terminal text with colors and formatting
/// - **SpinnerComponent**: Animated spinner for long-running operations
/// - **MessageComponent**: Permanent message display component
/// - **TerminalComponent**: Base trait for terminal output components
///
/// # Design Philosophy
///
/// The utilities are designed to provide clean, ephemeral feedback that doesn't clutter
/// the terminal output. Spinners disappear completely when operations finish, while
/// messages remain as permanent records of actions taken.
pub mod crossterm_utils {
    use super::*;
    use crossterm::execute;

    /// Base trait for terminal output components.
    ///
    /// Each component manages its own terminal state and cleanup, ensuring
    /// consistent behavior and proper resource management. Components can
    /// be ephemeral (like spinners) or permanent (like messages).
    ///
    /// # Design Principles
    ///
    /// - Components are responsible for their own state management
    /// - Proper cleanup prevents terminal corruption
    /// - Graceful handling of start/stop cycles
    /// - Non-blocking operation where possible
    pub trait TerminalComponent {
        /// Start the component and display its initial state.
        ///
        /// # Returns
        ///
        /// `Ok(())` on success, or an IO error if terminal operations fail
        fn start(&mut self) -> std::io::Result<()>;

        /// Stop the component and clean up its terminal state.
        ///
        /// # Returns
        ///
        /// `Ok(())` on success, or an IO error if cleanup fails
        fn stop(&mut self) -> std::io::Result<()>;

        /// Ensure the terminal is ready for the next component.
        ///
        /// Default implementation prints a newline to ensure proper spacing.
        /// Components can override this for specific cleanup behavior.
        ///
        /// # Returns
        ///
        /// `Ok(())` on success, or an IO error if terminal operations fail
        fn cleanup(&mut self) -> std::io::Result<()> {
            // Default implementation - print newline to ensure next component starts fresh
            use crossterm::{execute, style::Print};
            execute!(std::io::stdout(), Print("\n"))?;
            Ok(())
        }
    }

    /// An animated spinner component that disappears when the operation completes.
    ///
    /// The spinner provides visual feedback for long-running operations using
    /// a dots animation. It runs in a separate thread to avoid blocking the
    /// main operation and automatically cleans up when the operation finishes.
    ///
    /// # Animation
    ///
    /// Uses a 10-frame dots animation (⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏) that updates every 80ms
    /// for smooth visual feedback without being distracting.
    ///
    /// # Thread Safety
    ///
    /// The spinner uses atomic operations for thread-safe communication
    /// between the main thread and the animation thread.
    pub struct SpinnerComponent {
        message: String,
        handle: Option<std::thread::JoinHandle<()>>,
        stop_signal: Arc<std::sync::atomic::AtomicBool>,
        started: bool,
    }

    impl SpinnerComponent {
        /// Creates a new spinner component with the specified message.
        ///
        /// # Arguments
        ///
        /// * `message` - The message to display alongside the spinner animation
        ///
        /// # Returns
        ///
        /// A new `SpinnerComponent` ready to be started
        ///
        /// # Examples
        ///
        /// ```rust
        /// let mut spinner = SpinnerComponent::new("Loading data");
        /// ```
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

    /// A permanent message component that remains in the terminal output.
    ///
    /// Unlike spinners, message components create permanent output that becomes
    /// part of the terminal history. They are used for displaying important
    /// information that users may need to reference later.
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

    /// Builder for creating styled terminal text with colors and formatting.
    ///
    /// This struct provides a fluent interface for building styled text that
    /// can be displayed in the terminal with various colors and attributes.
    /// The styling is applied when the text is written to the terminal.
    ///
    /// # Supported Styling
    ///
    /// - **Foreground Colors**: cyan, green, yellow, red
    /// - **Background Colors**: green background (on_green)
    /// - **Attributes**: bold text
    ///
    /// # Examples
    ///
    /// ```rust
    /// let styled = StyledText::new("Success".to_string())
    ///     .green()
    ///     .bold();
    /// ```
    pub struct StyledText {
        text: String,
        foreground: Option<Color>,
        background: Option<Color>,
        bold: bool,
    }

    impl StyledText {
        /// Creates a new StyledText with the specified text content.
        ///
        /// # Arguments
        ///
        /// * `text` - The text content to be styled
        ///
        /// # Returns
        ///
        /// A new `StyledText` instance with no styling applied
        pub fn new(text: String) -> Self {
            Self {
                text,
                foreground: None,
                background: None,
                bold: false,
            }
        }

        /// Sets the foreground color to cyan.
        ///
        /// # Returns
        ///
        /// Self for method chaining
        pub fn cyan(mut self) -> Self {
            self.foreground = Some(Color::Cyan);
            self
        }

        /// Sets the foreground color to green.
        ///
        /// # Returns
        ///
        /// Self for method chaining
        pub fn green(mut self) -> Self {
            self.foreground = Some(Color::Green);
            self
        }

        /// Sets the foreground color to yellow.
        ///
        /// # Returns
        ///
        /// Self for method chaining
        pub fn yellow(mut self) -> Self {
            self.foreground = Some(Color::Yellow);
            self
        }

        /// Sets the foreground color to red.
        ///
        /// # Returns
        ///
        /// Self for method chaining
        pub fn red(mut self) -> Self {
            self.foreground = Some(Color::Red);
            self
        }

        /// Applies bold formatting to the text.
        ///
        /// # Returns
        ///
        /// Self for method chaining
        pub fn bold(mut self) -> Self {
            self.bold = true;
            self
        }

        /// Sets the background color to green.
        ///
        /// # Returns
        ///
        /// Self for method chaining
        pub fn on_green(mut self) -> Self {
            self.background = Some(Color::Green);
            self
        }
    }

    /// Width of the action column in terminal output
    const ACTION_WIDTH: usize = 15;

    /// Writes a styled line to the terminal with consistent formatting.
    ///
    /// This function handles the complex terminal operations needed to display
    /// styled text with proper alignment and formatting. The action text is
    /// right-aligned in a fixed-width column for visual consistency.
    ///
    /// # Arguments
    ///
    /// * `styled_text` - The styled text configuration for the action portion
    /// * `message` - The main message content to display
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an IO error if terminal operations fail
    ///
    /// # Format
    ///
    /// The output format is: `[ACTION (15 chars, right-aligned)] [message]`
    /// where ACTION is styled according to the StyledText configuration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let styled = StyledText::new("Success".to_string()).green().bold();
    /// write_styled_line(&styled, "Operation completed successfully")?;
    /// ```
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

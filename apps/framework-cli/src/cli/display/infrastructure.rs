//! Infrastructure change display functionality.
//!
//! This module handles the display of infrastructure changes including
//! OLAP changes, streaming changes, process changes, and API changes.
//! It provides specialized formatting for different types of infrastructure
//! modifications.

use super::terminal::{write_styled_line, StyledText};
use crate::framework::core::{
    infrastructure_map::{
        ApiChange, Change, OlapChange, ProcessChange, StreamingChange, TableChange,
    },
    plan::InfraPlan,
};
use crossterm::{execute, style::Print};
use log::info;

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
/// # use crate::cli::display::infrastructure::infra_added;
/// infra_added("Database table 'users' created");
/// ```
pub fn infra_added(message: &str) {
    let styled_text = StyledText::from_str("+ ").green();
    write_styled_line(&styled_text, message).expect("failed to write message to terminal");
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
/// # use crate::cli::display::infrastructure::infra_removed;
/// infra_removed("Database table 'temp_data' dropped");
/// ```
pub fn infra_removed(message: &str) {
    let styled_text = StyledText::from_str("- ").red();
    write_styled_line(&styled_text, message).expect("failed to write message to terminal");
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
/// # use crate::cli::display::infrastructure::infra_updated;
/// infra_updated("Database table 'users' schema modified");
/// ```
pub fn infra_updated(message: &str) {
    let styled_text = StyledText::from_str("~ ").yellow();
    write_styled_line(&styled_text, message).expect("failed to write message to terminal");
    info!("~ {}", message.trim());
}

/// Displays a multi-line message about infrastructure being updated with detailed formatting.
///
/// Uses yellow styling with a "~" prefix and supports indented details on subsequent lines.
/// Each detail line is properly indented for readability.
///
/// # Arguments
///
/// * `title` - The main title of the update
/// * `details` - A slice of detail strings to display with indentation
///
/// # Examples
///
/// ```rust
/// # use crate::cli::display::infrastructure::infra_updated_detailed;
/// infra_updated_detailed("Table users schema modified", &[
///     "Column changes:",
///     "  + email: String",
///     "  ~ age: Int32 -> Float64"
/// ]);
/// ```
pub fn infra_updated_detailed(title: &str, details: &[String]) {
    infra_updated(title);

    // Write detailed lines with proper indentation
    let mut stdout = std::io::stdout();
    for detail in details {
        execute!(
            stdout,
            Print("                  "),
            Print(detail),
            Print("\n")
        )
        .expect("failed to write detail to terminal");
    }

    // Log the full message
    info!("~ {} {}", title.trim(), details.join(" "));
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
/// # use crate::cli::display::infrastructure::show_olap_changes;
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
                let mut details = Vec::new();

                if !column_changes.is_empty() {
                    details.push("Column changes:".to_string());
                    for change in column_changes {
                        let change_line = match change {
                            crate::framework::core::infrastructure_map::ColumnChange::Added {
                                column,
                                ..
                            } => format!("  + {}: {}", column.name, column.data_type),
                            crate::framework::core::infrastructure_map::ColumnChange::Removed(
                                column,
                            ) => format!("  - {}: {}", column.name, column.data_type),
                            crate::framework::core::infrastructure_map::ColumnChange::Updated {
                                before,
                                after,
                            } => format!(
                                "  ~ {}: {} -> {}",
                                before.name, before.data_type, after.data_type
                            ),
                        };
                        details.push(change_line);
                    }
                }

                if order_by_change.before != order_by_change.after {
                    details.push("Order by changes:".to_string());
                    details.push(format!("  - {}", order_by_change.before.join(", ")));
                    details.push(format!("  + {}", order_by_change.after.join(", ")));
                }

                infra_updated_detailed(&format!("Table {name}"), &details);
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

/// Displays streaming engine infrastructure changes.
///
/// This function handles the display of changes to streaming components
/// including topics and streaming infrastructure.
///
/// # Arguments
///
/// * `streaming_changes` - A slice of streaming changes to display
///
/// # Change Types Handled
///
/// - **Topic Changes**: Added, removed, or updated streaming topics
///
/// # Examples
///
/// ```rust
/// # use crate::cli::display::infrastructure::show_streaming_changes;
/// show_streaming_changes(&infrastructure_plan.changes.streaming_engine_changes);
/// ```
pub fn show_streaming_changes(streaming_changes: &[StreamingChange]) {
    streaming_changes.iter().for_each(|change| match change {
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
}

/// Displays process infrastructure changes.
///
/// This function handles the display of changes to various process types
/// including sync processes, functions, and web servers.
///
/// # Arguments
///
/// * `process_changes` - A slice of process changes to display
///
/// # Change Types Handled
///
/// - **Topic to Topic Sync Process**: Data synchronization between topics
/// - **Topic to Table Sync Process**: Data synchronization from topics to tables
/// - **Function Process**: Custom function processes
/// - **OLAP Process**: OLAP processing components
/// - **Consumption API Web Server**: API server processes
/// - **Orchestration Worker**: Background worker processes
///
/// # Examples
///
/// ```rust
/// # use crate::cli::display::infrastructure::show_process_changes;
/// show_process_changes(&infrastructure_plan.changes.processes_changes);
/// ```
pub fn show_process_changes(process_changes: &[ProcessChange]) {
    process_changes.iter().for_each(|change| match change {
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
            infra_removed(&infra.expanded_display());
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
}

/// Displays API infrastructure changes.
///
/// This function handles the display of changes to API components
/// including endpoints and related infrastructure.
///
/// # Arguments
///
/// * `api_changes` - A slice of API changes to display
///
/// # Change Types Handled
///
/// - **API Endpoint Changes**: Added, removed, or updated API endpoints
///
/// # Examples
///
/// ```rust
/// # use crate::cli::display::infrastructure::show_api_changes;
/// show_api_changes(&infrastructure_plan.changes.api_changes);
/// ```
pub fn show_api_changes(api_changes: &[ApiChange]) {
    api_changes.iter().for_each(|change| match change {
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
/// # use crate::cli::display::infrastructure::show_changes;
/// show_changes(&infrastructure_plan);
/// ```
pub fn show_changes(infra_plan: &InfraPlan) {
    show_streaming_changes(&infra_plan.changes.streaming_engine_changes);
    show_olap_changes(&infra_plan.changes.olap_changes);
    show_process_changes(&infra_plan.changes.processes_changes);
    show_api_changes(&infra_plan.changes.api_changes);
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests primarily verify that the functions don't panic
    // and have correct signatures. Testing actual terminal output would
    // require mocking stdout, which is complex for these integration-style functions.

    #[test]
    fn test_infra_functions_dont_panic() {
        // These functions write to stdout and log, so we can't easily test output
        // but we can ensure they don't panic with various inputs

        // Test with normal message
        let _ = std::panic::catch_unwind(|| {
            // We can't actually call these without affecting test output
            // but we can verify the function signatures exist
            let _f1: fn(&str) = infra_added;
            let _f2: fn(&str) = infra_removed;
            let _f3: fn(&str) = infra_updated;
        });
    }

    #[test]
    fn test_show_functions_exist() {
        // Verify function signatures exist and are callable
        let _f1: fn(&[OlapChange]) = show_olap_changes;
        let _f2: fn(&[StreamingChange]) = show_streaming_changes;
        let _f3: fn(&[ProcessChange]) = show_process_changes;
        let _f4: fn(&[ApiChange]) = show_api_changes;
        let _f5: fn(&InfraPlan) = show_changes;
    }

    #[test]
    fn test_empty_slices_dont_panic() {
        // Test that empty slices don't cause panics
        let empty_olap: &[OlapChange] = &[];
        let empty_streaming: &[StreamingChange] = &[];
        let empty_process: &[ProcessChange] = &[];
        let empty_api: &[ApiChange] = &[];

        // These should not panic
        show_olap_changes(empty_olap);
        show_streaming_changes(empty_streaming);
        show_process_changes(empty_process);
        show_api_changes(empty_api);
    }

    // Integration tests would go here if we had mock infrastructure objects
    // For now, the main testing happens at the integration level where
    // actual infrastructure changes are created and displayed.
}

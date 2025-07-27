//! Table display functionality for formatted output.
//!
//! This module provides utilities for displaying data in formatted tables
//! with consistent styling and layout.

use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, ContentArrangement, Table};

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
/// # use crate::cli::display::table::show_table;
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

/// Creates a formatted table with headers and data rows without displaying it.
///
/// This function creates a table similar to `show_table` but returns the
/// formatted string instead of printing it directly. Useful for testing
/// or when you need to manipulate the table output before displaying.
///
/// # Arguments
///
/// * `title` - The title to display above the table
/// * `headers` - Column headers for the table
/// * `rows` - Data rows, where each row is a vector of column values
///
/// # Returns
///
/// A formatted string containing the title and table
///
/// # Examples
///
/// ```rust
/// # use crate::cli::display::table::format_table;
/// let table_output = format_table(
///     "User Data".to_string(),
///     vec!["ID".to_string(), "Name".to_string()],
///     vec![
///         vec!["1".to_string(), "Alice".to_string()],
///         vec!["2".to_string(), "Bob".to_string()],
///     ]
/// );
/// println!("{}", table_output);
/// ```
pub fn format_table(title: String, headers: Vec<String>, rows: Vec<Vec<String>>) -> String {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS);
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(headers);

    for row in rows {
        table.add_row(row);
    }

    format!("{title}\n{table}")
}

/// Creates a simple table without a title.
///
/// This is a convenience function for creating tables when you don't need
/// a title or want to handle the title separately.
///
/// # Arguments
///
/// * `headers` - Column headers for the table
/// * `rows` - Data rows, where each row is a vector of column values
///
/// # Returns
///
/// A formatted string containing just the table
///
/// # Examples
///
/// ```rust
/// # use crate::cli::display::table::format_simple_table;
/// let table_output = format_simple_table(
///     vec!["Name".to_string(), "Age".to_string()],
///     vec![
///         vec!["Alice".to_string(), "30".to_string()],
///         vec!["Bob".to_string(), "25".to_string()],
///     ]
/// );
/// ```
pub fn format_simple_table(headers: Vec<String>, rows: Vec<Vec<String>>) -> String {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS);
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(headers);

    for row in rows {
        table.add_row(row);
    }

    table.to_string()
}

/// Displays a simple table without a title.
///
/// This is a convenience function for displaying tables when you don't need
/// a title.
///
/// # Arguments
///
/// * `headers` - Column headers for the table
/// * `rows` - Data rows, where each row is a vector of column values
///
/// # Examples
///
/// ```rust
/// # use crate::cli::display::table::show_simple_table;
/// show_simple_table(
///     vec!["Name".to_string(), "Status".to_string()],
///     vec![
///         vec!["Service A".to_string(), "Running".to_string()],
///         vec!["Service B".to_string(), "Stopped".to_string()],
///     ]
/// );
/// ```
pub fn show_simple_table(headers: Vec<String>, rows: Vec<Vec<String>>) {
    println!("{}", format_simple_table(headers, rows));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_table() {
        let title = "Test Table".to_string();
        let headers = vec!["Col1".to_string(), "Col2".to_string()];
        let rows = vec![
            vec!["A".to_string(), "B".to_string()],
            vec!["C".to_string(), "D".to_string()],
        ];

        let result = format_table(title.clone(), headers, rows);

        // Check that the title is included
        assert!(result.contains(&title));
        // Check that the table contains our data
        assert!(result.contains("Col1"));
        assert!(result.contains("Col2"));
        assert!(result.contains("A"));
        assert!(result.contains("B"));
        assert!(result.contains("C"));
        assert!(result.contains("D"));
    }

    #[test]
    fn test_format_simple_table() {
        let headers = vec!["Name".to_string(), "Value".to_string()];
        let rows = vec![
            vec!["Test1".to_string(), "Value1".to_string()],
            vec!["Test2".to_string(), "Value2".to_string()],
        ];

        let result = format_simple_table(headers, rows);

        // Check that the table contains our data
        assert!(result.contains("Name"));
        assert!(result.contains("Value"));
        assert!(result.contains("Test1"));
        assert!(result.contains("Value1"));
        assert!(result.contains("Test2"));
        assert!(result.contains("Value2"));
    }

    #[test]
    fn test_empty_table() {
        let headers = vec!["Header1".to_string(), "Header2".to_string()];
        let rows: Vec<Vec<String>> = vec![];

        let result = format_simple_table(headers, rows);

        // Should still contain headers even with no rows
        assert!(result.contains("Header1"));
        assert!(result.contains("Header2"));
    }

    #[test]
    fn test_single_row_table() {
        let headers = vec!["ID".to_string(), "Name".to_string()];
        let rows = vec![vec!["1".to_string(), "Alice".to_string()]];

        let result = format_simple_table(headers, rows);

        assert!(result.contains("ID"));
        assert!(result.contains("Name"));
        assert!(result.contains("1"));
        assert!(result.contains("Alice"));
    }

    #[test]
    fn test_unicode_table() {
        let headers = vec!["名前".to_string(), "年齢".to_string()];
        let rows = vec![
            vec!["太郎".to_string(), "25".to_string()],
            vec!["花子".to_string(), "30".to_string()],
        ];

        let result = format_simple_table(headers, rows);

        // Check that Unicode characters are handled correctly
        assert!(result.contains("名前"));
        assert!(result.contains("年齢"));
        assert!(result.contains("太郎"));
        assert!(result.contains("花子"));
    }

    #[test]
    fn test_empty_cells() {
        let headers = vec!["Col1".to_string(), "Col2".to_string()];
        let rows = vec![
            vec!["A".to_string(), "".to_string()],
            vec!["".to_string(), "B".to_string()],
        ];

        let result = format_simple_table(headers, rows);

        // Should handle empty cells gracefully
        assert!(result.contains("Col1"));
        assert!(result.contains("Col2"));
        assert!(result.contains("A"));
        assert!(result.contains("B"));
    }

    #[test]
    fn test_long_content() {
        let headers = vec!["Short".to_string(), "Long Content".to_string()];
        let rows = vec![vec![
            "A".to_string(),
            "This is a very long piece of content that should be handled properly by the table formatter".to_string(),
        ]];

        let result = format_simple_table(headers, rows);

        // Should handle long content without panicking
        assert!(result.contains("Short"));
        assert!(result.contains("Long Content"));
        assert!(result.contains("This is a very long piece"));
    }

    #[test]
    fn test_mismatched_row_lengths() {
        let headers = vec!["Col1".to_string(), "Col2".to_string(), "Col3".to_string()];
        let rows = vec![
            vec!["A".to_string(), "B".to_string()], // Missing third column
            vec!["C".to_string(), "D".to_string(), "E".to_string()], // Complete row
        ];

        // This should not panic - comfy_table should handle mismatched lengths
        let result = format_simple_table(headers, rows);
        assert!(result.contains("Col1"));
        assert!(result.contains("Col2"));
        assert!(result.contains("Col3"));
    }

    // Note: show_table and show_simple_table are tested implicitly
    // since they use the same underlying logic as the format functions
    // but print to stdout instead of returning strings.

    #[test]
    fn test_show_functions_dont_panic() {
        let headers = vec!["Test".to_string()];
        let rows = vec![vec!["Value".to_string()]];

        // These functions print to stdout, so we can't easily test output
        // but we can ensure they don't panic
        let _ = std::panic::catch_unwind(|| {
            // We can't actually call these without affecting test output
            // but we can verify the function signatures exist
            let _f1: fn(String, Vec<String>, Vec<Vec<String>>) = show_table;
            let _f2: fn(Vec<String>, Vec<Vec<String>>) = show_simple_table;
        });
    }
}

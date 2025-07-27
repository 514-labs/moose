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
//! - **Terminal Utilities**: Low-level terminal manipulation and styling
//!
//! ## Module Structure
//!
//! - [`message`]: Core message types and structures
//! - [`message_display`]: Message display functionality and macros
//! - [`terminal`]: Terminal utilities and styling components
//! - [`spinner`]: Spinner components for progress indication
//! - [`infrastructure`]: Infrastructure change display functionality
//! - [`table`]: Table formatting and display utilities
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
//! use crate::cli::display::spinner::with_spinner;
//!
//! let result = with_spinner("Processing data", || {
//!     // Long-running operation
//!     expensive_computation()
//! }, true);
//! ```
//!
//! ### Infrastructure Change Display
//! ```rust
//! use crate::cli::display::infrastructure::show_changes;
//!
//! show_changes(&infrastructure_plan);
//! ```
//!
//! ### Table Display
//! ```rust
//! use crate::cli::display::table::show_table;
//!
//! show_table(
//!     "User Data".to_string(),
//!     vec!["ID".to_string(), "Name".to_string()],
//!     vec![vec!["1".to_string(), "Alice".to_string()]]
//! );
//! ```

#[macro_use]
pub mod message_display;

pub mod infrastructure;
pub mod message;
pub mod spinner;
pub mod table;
pub mod terminal;

// Re-export commonly used types and functions for convenience
pub use infrastructure::show_changes;
pub use message::{Message, MessageType};
pub use message_display::{batch_inserted, show_message_wrapper};
pub use spinner::{with_spinner, with_spinner_async};
pub use table::show_table;

// Legacy compatibility - maintain the crossterm_utils module for existing code
pub mod crossterm_utils {
    //! Legacy compatibility module for crossterm utilities.
    //!
    //! This module re-exports the terminal utilities under the original
    //! crossterm_utils namespace for backward compatibility with existing code.
    //!
    //! Note: These re-exports are maintained for backward compatibility but may
    //! not be actively used. The main display module provides the primary API.

    #[allow(unused_imports)]
    pub use super::spinner::SpinnerComponent;
    #[allow(unused_imports)]
    pub use super::terminal::{write_styled_line, StyledText, TerminalComponent};
}

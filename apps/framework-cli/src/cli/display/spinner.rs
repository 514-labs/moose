//! Spinner components for displaying progress during long-running operations.
//!
//! This module provides animated spinner functionality that gives users visual
//! feedback during operations that take time to complete. Spinners are ephemeral
//! and disappear completely when operations finish.

use super::terminal::TerminalComponent;
use crossterm::{
    cursor::MoveToColumn,
    execute,
    style::Print,
    terminal::{Clear, ClearType},
};
use std::io::IsTerminal;
use std::{
    io::{stdout, Result as IoResult},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};
use tokio::macros::support::Future;

/// Dots9 animation frames for the spinner
const DOTS9_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// Frame update interval in milliseconds
const FRAME_INTERVAL_MS: u64 = 80;

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
///
/// # Examples
///
/// ```rust
/// # use crate::cli::display::spinner::SpinnerComponent;
/// # use crate::cli::display::terminal::TerminalComponent;
/// let mut spinner = SpinnerComponent::new("Loading data");
/// spinner.start()?;
/// // ... long running operation ...
/// spinner.stop()?;
/// # Ok::<(), std::io::Error>(())
/// ```
pub struct SpinnerComponent {
    message: String,
    handle: Option<JoinHandle<()>>,
    stop_signal: Arc<AtomicBool>,
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
    /// # use crate::cli::display::spinner::SpinnerComponent;
    /// let mut spinner = SpinnerComponent::new("Loading data");
    /// ```
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
            handle: None,
            stop_signal: Arc::new(AtomicBool::new(false)),
            started: false,
        }
    }

    /// Checks if the spinner is currently running.
    ///
    /// # Returns
    ///
    /// `true` if the spinner is started and running
    pub fn is_running(&self) -> bool {
        self.started && !self.stop_signal.load(Ordering::Relaxed)
    }

    /// Gets the message displayed with the spinner.
    ///
    /// # Returns
    ///
    /// A reference to the spinner message
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl TerminalComponent for SpinnerComponent {
    fn start(&mut self) -> IoResult<()> {
        if self.started {
            return Ok(());
        }

        let message = self.message.clone();
        let stop_signal = self.stop_signal.clone();

        // Start on a fresh line using crossterm execute!
        execute!(stdout(), Print(&format!("{} {}", DOTS9_FRAMES[0], message)))?;

        self.handle = Some(thread::spawn(move || {
            let mut frame_index = 0;
            while !stop_signal.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(FRAME_INTERVAL_MS));

                if !stop_signal.load(Ordering::Relaxed) {
                    // Update spinner on same line using crossterm
                    let _ = execute!(
                        stdout(),
                        Print(&format!("\r{} {}", DOTS9_FRAMES[frame_index], message))
                    );
                    frame_index = (frame_index + 1) % DOTS9_FRAMES.len();
                }
            }
        }));

        self.started = true;
        Ok(())
    }

    fn stop(&mut self) -> IoResult<()> {
        if !self.started {
            return Ok(());
        }

        // Signal the thread to stop
        self.stop_signal.store(true, Ordering::Relaxed);

        // Wait for the thread to finish gracefully
        if let Some(handle) = self.handle.take() {
            // Join the thread directly - this ensures it has completely stopped
            // before we clean up the terminal. This eliminates race conditions
            // and prevents terminal corruption.
            let _ = handle.join();
        }

        // Clean up the current spinner line gracefully
        execute!(stdout(), Clear(ClearType::CurrentLine), MoveToColumn(0))?;

        self.started = false;
        Ok(())
    }

    fn cleanup(&mut self) -> IoResult<()> {
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
/// # use crate::cli::display::spinner::with_spinner;
/// let result = with_spinner("Loading configuration", || {
///     // Long-running operation
///     42
/// }, true);
/// assert_eq!(result, 42);
/// ```
pub fn with_spinner<F, R>(message: &str, f: F, activate: bool) -> R
where
    F: FnOnce() -> R,
{
    let sp = if activate && stdout().is_terminal() {
        let mut spinner = SpinnerComponent::new(message);
        let _ = spinner.start();
        Some(spinner)
    } else {
        None
    };

    let res = f();

    if let Some(mut spinner) = sp {
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
/// # use crate::cli::display::spinner::with_spinner_async;
/// # async fn example() {
/// let result = with_spinner_async("Fetching data", async {
///     // Async operation
///     42
/// }, true).await;
/// assert_eq!(result, 42);
/// # }
/// ```
pub async fn with_spinner_async<F, R>(message: &str, f: F, activate: bool) -> R
where
    F: Future<Output = R>,
{
    let sp = if activate && stdout().is_terminal() {
        let mut spinner = SpinnerComponent::new(message);
        let _ = spinner.start();
        Some(spinner)
    } else {
        None
    };

    let res = f.await;

    if let Some(mut spinner) = sp {
        let _ = spinner.stop();
    }

    res
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_spinner_component_new() {
        let spinner = SpinnerComponent::new("Test message");
        assert_eq!(spinner.message(), "Test message");
        assert!(!spinner.is_running());
    }

    #[test]
    fn test_spinner_component_message() {
        let spinner = SpinnerComponent::new("Loading data");
        assert_eq!(spinner.message(), "Loading data");
    }

    #[test]
    fn test_spinner_component_unicode_message() {
        let spinner = SpinnerComponent::new("🚀 Deploying");
        assert_eq!(spinner.message(), "🚀 Deploying");
    }

    #[test]
    fn test_spinner_component_empty_message() {
        let spinner = SpinnerComponent::new("");
        assert_eq!(spinner.message(), "");
    }

    #[test]
    fn test_with_spinner_returns_result() {
        let result = with_spinner("Test", || 42, false);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_with_spinner_with_closure() {
        let value = 10;
        let result = with_spinner("Test", || value * 2, false);
        assert_eq!(result, 20);
    }

    #[tokio::test]
    async fn test_with_spinner_async_returns_result() {
        let result = with_spinner_async("Test", async { 42 }, false).await;
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_with_spinner_async_with_delay() {
        let result = with_spinner_async(
            "Test",
            async {
                tokio::time::sleep(Duration::from_millis(10)).await;
                "completed"
            },
            false,
        )
        .await;
        assert_eq!(result, "completed");
    }

    #[test]
    fn test_with_spinner_error_handling() {
        let result: Result<i32, &str> = with_spinner("Test", || Err("test error"), false);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "test error");
    }

    #[tokio::test]
    async fn test_with_spinner_async_error_handling() {
        let result: Result<i32, &str> =
            with_spinner_async("Test", async { Err("test error") }, false).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "test error");
    }

    #[test]
    fn test_dots9_frames_constant() {
        assert_eq!(DOTS9_FRAMES.len(), 10);
        assert_eq!(DOTS9_FRAMES[0], "⠋");
        assert_eq!(DOTS9_FRAMES[9], "⠏");
    }

    #[test]
    fn test_frame_interval_constant() {
        assert_eq!(FRAME_INTERVAL_MS, 80);
    }

    // Integration test that actually starts and stops a spinner
    // This is commented out because it requires terminal interaction
    // but demonstrates how the spinner would be tested in integration tests
    /*
    #[test]
    fn test_spinner_start_stop_integration() {
        let mut spinner = SpinnerComponent::new("Test spinner");

        // Start the spinner
        spinner.start().expect("Failed to start spinner");
        assert!(spinner.is_running());

        // Let it run briefly
        std::thread::sleep(Duration::from_millis(200));

        // Stop the spinner
        spinner.stop().expect("Failed to stop spinner");
        assert!(!spinner.is_running());
    }
    */
}

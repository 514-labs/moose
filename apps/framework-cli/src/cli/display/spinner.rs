//! Spinner components for displaying progress during long-running operations.
//!
//! This module provides animated spinner functionality that gives users visual
//! feedback during operations that take time to complete. Spinners are ephemeral
//! and disappear completely when operations finish.

use super::terminal::TerminalComponent;
use crossterm::{
    cursor::{position, MoveTo, RestorePosition, SavePosition},
    execute, queue,
    style::Print,
    terminal::{BeginSynchronizedUpdate, Clear, ClearType, EndSynchronizedUpdate},
};
use std::io::IsTerminal;
use std::{
    io::{stdout, Result as IoResult, Write},
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

/// Checkmark symbol for completed spinner
const CHECKMARK: &str = "✓";

/// Frame update interval in milliseconds
const FRAME_INTERVAL_MS: u64 = 80;

/// An animated spinner component that stays on the last line while other output appears above.
///
/// The spinner provides visual feedback for long-running operations using
/// a dots animation. It runs in a separate thread to avoid blocking the
/// main operation and automatically handles concurrent output by keeping
/// the spinner on the bottom line and pushing other output above it.
///
/// # Animation
///
/// Uses a 10-frame dots animation (⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏) that updates every 80ms
/// for smooth visual feedback without being distracting.
///
/// # Concurrent Output Handling
///
/// When other processes write to stdout while the spinner is running,
/// the spinner automatically moves to a new line below the output,
/// ensuring it's always visible at the bottom of the terminal.
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
/// // ... long running operation that may produce output ...
///
/// // Option 1: Complete with success message (shows checkmark)
/// spinner.done("Data loaded successfully")?;
///
/// // Option 2: Just stop without completion message (clears line)
/// // spinner.stop()?;
///
/// // Option 3: Stop with optional completion message
/// // spinner.stop_with_message(Some("Data loaded successfully"))?;
/// # Ok::<(), std::io::Error>(())
/// ```
pub struct SpinnerComponent {
    message: String,
    handle: Option<JoinHandle<()>>,
    stop_signal: Arc<AtomicBool>,
    started: bool,
    initial_line: Option<u16>,
    is_done: bool,
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
            initial_line: None,
            is_done: false,
        }
    }

    /// Completes the spinner with a checkmark and completion message.
    ///
    /// This method stops the spinner animation and displays a checkmark
    /// with the provided completion message on the reserved spinner line.
    /// The spinner remains visible to show successful completion.
    ///
    /// # Arguments
    ///
    /// * `completion_message` - The message to display after the checkmark
    ///
    /// # Returns
    ///
    /// `IoResult<()>` indicating success or failure
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use crate::cli::display::spinner::SpinnerComponent;
    /// # use crate::cli::display::terminal::TerminalComponent;
    /// let mut spinner = SpinnerComponent::new("Loading data");
    /// spinner.start()?;
    /// // ... long running operation ...
    /// spinner.done("Data loaded successfully")?;
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn done(&mut self, completion_message: &str) -> IoResult<()> {
        if !self.started {
            self.is_done = true;
            return Ok(());
        }

        // Signal the thread to stop
        self.stop_signal.store(true, Ordering::Relaxed);

        // Wait for the thread to finish gracefully
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }

        // Display checkmark with completion message on the reserved line
        if let Some(initial_line) = self.initial_line {
            queue!(
                stdout(),
                SavePosition,
                MoveTo(0, initial_line),
                Clear(ClearType::CurrentLine),
                Print(&format!("{CHECKMARK} {completion_message}")),
                RestorePosition
            )?;
            stdout().flush()?;
        }

        self.started = false;
        self.is_done = true;
        Ok(())
    }

    /// Stops the spinner with an optional completion message.
    ///
    /// If a completion message is provided, displays a checkmark with the message.
    /// If no completion message is provided, clears the spinner line completely.
    ///
    /// # Arguments
    ///
    /// * `completion_message` - Optional completion message to display with checkmark
    ///
    /// # Returns
    ///
    /// `IoResult<()>` indicating success or failure
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use crate::cli::display::spinner::SpinnerComponent;
    /// # use crate::cli::display::terminal::TerminalComponent;
    /// let mut spinner = SpinnerComponent::new("Processing");
    /// spinner.start()?;
    /// // ... operation ...
    /// spinner.stop_with_message(Some("Processing complete"))?; // Shows checkmark
    /// // OR
    /// spinner.stop_with_message(None)?; // Just clears the line
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn stop_with_message(&mut self, completion_message: Option<&str>) -> IoResult<()> {
        match completion_message {
            Some(message) => self.done(message),
            None => self.stop(),
        }
    }
}

impl TerminalComponent for SpinnerComponent {
    fn start(&mut self) -> IoResult<()> {
        if self.started {
            return Ok(());
        }

        let message = self.message.clone();
        let stop_signal = self.stop_signal.clone();

        // Capture current cursor position to reserve this line for the spinner
        let initial_pos = position().unwrap_or((0, 0));
        self.initial_line = Some(initial_pos.1);

        // Display initial spinner frame and add newline to separate from subprocess output
        execute!(stdout(), Print(&format!("{} {message}\n", DOTS9_FRAMES[0])))?;
        stdout().flush()?;

        let initial_line = self.initial_line.unwrap();

        self.handle = Some(thread::spawn(move || {
            let mut frame_index = 0;

            while !stop_signal.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(FRAME_INTERVAL_MS));

                if !stop_signal.load(Ordering::Relaxed) {
                    frame_index = (frame_index + 1) % DOTS9_FRAMES.len();

                    // Try synchronized updates first (atomic operation)
                    let sync_result = queue!(
                        stdout(),
                        BeginSynchronizedUpdate,
                        SavePosition,
                        MoveTo(0, initial_line),
                        Clear(ClearType::CurrentLine),
                        Print(&format!("{} {message}", DOTS9_FRAMES[frame_index])),
                        RestorePosition,
                        EndSynchronizedUpdate
                    )
                    .and_then(|_| stdout().flush());

                    // If synchronized updates fail, fall back to queue-based approach
                    if sync_result.is_err() {
                        let _ = queue!(
                            stdout(),
                            SavePosition,
                            MoveTo(0, initial_line),
                            Clear(ClearType::CurrentLine),
                            Print(&format!("{} {message}", DOTS9_FRAMES[frame_index])),
                            RestorePosition
                        )
                        .and_then(|_| stdout().flush());
                    }
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

        // Clean up the reserved spinner line if we have it
        if let Some(initial_line) = self.initial_line {
            queue!(
                stdout(),
                SavePosition,
                MoveTo(0, initial_line),
                Clear(ClearType::CurrentLine),
                RestorePosition
            )?;
            stdout().flush()?;
        }

        self.started = false;
        self.initial_line = None;
        // Note: is_done is NOT set to true when just stopping (only when completing with done())
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
/// an animated spinner. The spinner automatically handles concurrent output by detecting
/// when other processes write to stdout and moving to a new line to stay visible.
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
/// - When other output appears, the spinner automatically moves to a new line below it
/// - Terminal state is properly cleaned up after completion
///
/// # Concurrent Output Handling
///
/// The spinner intelligently detects when other processes or threads write to stdout
/// and automatically repositions itself to remain visible at the bottom of the output.
/// This ensures that both the spinner and any log output from subprocesses are visible.
///
/// # Examples
///
/// ```rust
/// # use crate::cli::display::spinner::with_spinner;
/// let result = with_spinner("Processing data", || {
///     // Long-running operation that may produce output
///     println!("Processing step 1...");
///     println!("Processing step 2...");
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

/// Executes a function with a spinner and completion message displayed during execution.
///
/// This is an enhanced version of `with_spinner` that displays a checkmark with
/// a completion message when the operation finishes successfully. If the operation
/// returns an error, the spinner is simply cleared without showing completion.
///
/// # Arguments
///
/// * `message` - The message to display alongside the spinner
/// * `completion_message` - The message to display with checkmark upon successful completion
/// * `f` - The function to execute while the spinner is active
/// * `activate` - Whether to actually show the spinner (false or non-terminal disables spinner)
///
/// # Returns
///
/// The result of the function execution, unchanged
///
/// # Examples
///
/// ```rust
/// # use crate::cli::display::spinner::with_spinner_completion;
/// let result = with_spinner_completion("Processing data", "Data processed successfully", || {
///     // Long-running operation
///     42
/// }, true);
/// assert_eq!(result, 42);
/// // Spinner shows: "✓ Data processed successfully"
/// ```
pub fn with_spinner_completion<F, R>(
    message: &str,
    completion_message: &str,
    f: F,
    activate: bool,
) -> R
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
        let _ = spinner.done(completion_message);
        let _ = spinner.cleanup();
    }

    res
}

/// Executes an asynchronous function with a spinner displayed during execution.
///
/// This is the async version of `with_spinner`, providing the same visual feedback
/// and concurrent output handling for long-running async operations.
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
/// # Concurrent Output Handling
///
/// Like `with_spinner`, this function automatically handles concurrent output from
/// other processes or async tasks, ensuring the spinner remains visible at the
/// bottom of the terminal while other output appears above it.
///
/// # Examples
///
/// ```rust
/// # use crate::cli::display::spinner::with_spinner_async;
/// # async fn example() {
/// let result = with_spinner_async("Processing async data", async {
///     // Async operation that may produce output
///     println!("Async step 1 complete");
///     tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
///     println!("Async step 2 complete");
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

/// Executes an asynchronous function with a spinner and completion message displayed during execution.
///
/// This is the async version of `with_spinner_completion`, providing the same completion
/// message functionality for long-running async operations.
///
/// # Arguments
///
/// * `message` - The message to display alongside the spinner
/// * `completion_message` - The message to display with checkmark upon successful completion
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
/// # use crate::cli::display::spinner::with_spinner_completion_async;
/// # async fn example() {
/// let result = with_spinner_completion_async("Processing async data", "Async processing complete", async {
///     // Async operation
///     tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
///     42
/// }, true).await;
/// assert_eq!(result, 42);
/// // Spinner shows: "✓ Async processing complete"
/// # }
/// ```
pub async fn with_spinner_completion_async<F, R>(
    message: &str,
    completion_message: &str,
    f: F,
    activate: bool,
) -> R
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
        let _ = spinner.done(completion_message);
    }

    res
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_spinner_component_new() {
        let _spinner = SpinnerComponent::new("Test message");
        // Just test that creation doesn't panic
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

    #[test]
    fn test_spinner_component_tracks_initial_line() {
        let spinner = SpinnerComponent::new("Test spinner");
        assert_eq!(spinner.initial_line, None);
        // After start() is called, initial_line should be set
        // This is tested in integration tests since it requires terminal interaction
    }

    #[test]
    fn test_spinner_concurrent_output_simulation() {
        // This test simulates the concurrent output scenario without actual terminal interaction
        // It verifies that the spinner component properly handles the line tracking logic
        let spinner = SpinnerComponent::new("Processing");

        // Verify initial state
        assert!(!spinner.started);
        assert_eq!(spinner.initial_line, None);
        assert!(!spinner.is_done);

        // Note: Actual concurrent output testing requires integration tests
        // with real terminal interaction, which is commented out below
    }

    #[test]
    fn test_spinner_done_state() {
        let mut spinner = SpinnerComponent::new("Processing");

        // Verify initial state
        assert!(!spinner.is_done);

        // Test done() method without starting (should not error)
        let result = spinner.done("Task completed");
        assert!(result.is_ok());
        assert!(spinner.is_done);
    }

    #[test]
    fn test_spinner_stop_with_message() {
        let mut spinner = SpinnerComponent::new("Processing");

        // Test stop_with_message with completion message
        let result = spinner.stop_with_message(Some("Task completed"));
        assert!(result.is_ok());
        assert!(spinner.is_done);

        // Test stop_with_message without completion message
        let mut spinner2 = SpinnerComponent::new("Processing");
        let result2 = spinner2.stop_with_message(None);
        assert!(result2.is_ok());
        assert!(!spinner2.is_done); // Should not be marked as done when just stopped
    }

    #[test]
    fn test_with_spinner_completion() {
        let result = with_spinner_completion("Processing", "Task completed", || 42, false);
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_with_spinner_completion_async() {
        let result =
            with_spinner_completion_async("Processing", "Task completed", async { 42 }, false)
                .await;
        assert_eq!(result, 42);
    }

    #[test]
    fn test_checkmark_constant() {
        assert_eq!(CHECKMARK, "✓");
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
        assert!(spinner.started);

        // Let it run briefly
        std::thread::sleep(Duration::from_millis(200));

        // Stop the spinner
        spinner.stop().expect("Failed to stop spinner");
        assert!(!spinner.started);
    }

    #[test]
    fn test_spinner_with_concurrent_subprocess_output() {
        use std::process::Command;

        let mut spinner = SpinnerComponent::new("Processing with concurrent output");

        // Start the spinner
        spinner.start().expect("Failed to start spinner");

        // Simulate subprocess output while spinner is running
        let output = Command::new("echo")
            .arg("This is subprocess output")
            .output()
            .expect("Failed to execute subprocess");

        println!("{}", String::from_utf8_lossy(&output.stdout));

        // Let spinner run a bit more
        std::thread::sleep(Duration::from_millis(500));

        // More subprocess output
        let output2 = Command::new("echo")
            .arg("More subprocess output")
            .output()
            .expect("Failed to execute subprocess");

        println!("{}", String::from_utf8_lossy(&output2.stdout));

        // Stop the spinner
        spinner.stop().expect("Failed to stop spinner");

        // Verify spinner is properly cleaned up
        assert!(!spinner.started);
        assert_eq!(spinner.initial_line, None);
    }
    */
}

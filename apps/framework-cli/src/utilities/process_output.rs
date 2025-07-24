use log::{info, warn};
use std::io::{BufRead, BufReader as StdBufReader};
use std::process::{Command, Stdio};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{ChildStderr, ChildStdout};

/// Utility for safely managing subprocess output while preventing terminal corruption
pub struct ProcessOutputProxy {
    stdout_task: tokio::task::JoinHandle<()>,
    stderr_task: tokio::task::JoinHandle<()>,
}

impl ProcessOutputProxy {
    /// Create a new output proxy that forwards subprocess output to logs
    pub fn new(stdout: ChildStdout, stderr: ChildStderr, label: &str) -> Self {
        let label_stdout = format!("[{label}]");
        let label_stderr = label_stdout.clone();

        let stdout_task = tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                info!("{} {}", label_stdout, line);
            }
        });

        let stderr_task = tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                warn!("{} {}", label_stderr, line);
            }
        });

        Self {
            stdout_task,
            stderr_task,
        }
    }

    /// Wait for all output to be processed
    pub async fn wait_for_completion(self) {
        let _ = tokio::join!(self.stdout_task, self.stderr_task);
    }
}

/// Run a command with safe stdio piping and output proxying
pub async fn run_command_with_output_proxy(
    mut command: tokio::process::Command,
    label: &str,
) -> Result<std::process::ExitStatus, Box<dyn std::error::Error + Send + Sync>> {
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let stderr = child.stderr.take().expect("Failed to capture stderr");

    let proxy = ProcessOutputProxy::new(stdout, stderr, label);
    let status = child.wait().await?;
    proxy.wait_for_completion().await;

    Ok(status)
}

/// Synchronous version for use in non-async contexts (like tests)
pub fn run_command_with_output_proxy_sync(
    mut command: Command,
    label: &str,
) -> Result<std::process::ExitStatus, Box<dyn std::error::Error + Send + Sync>> {
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let stderr = child.stderr.take().expect("Failed to capture stderr");

    let label_stdout = format!("[{label}]");
    let label_stderr = label_stdout.clone();

    // Spawn threads to handle output
    let stdout_handle = std::thread::spawn(move || {
        let reader = StdBufReader::new(stdout);
        for line in reader.lines().flatten() {
            info!("{label_stdout} {line}");
        }
    });

    let stderr_handle = std::thread::spawn(move || {
        let reader = StdBufReader::new(stderr);
        for line in reader.lines().flatten() {
            warn!("{label_stderr} {line}");
        }
    });

    let status = child.wait()?;

    // Wait for output threads to complete
    let _ = stdout_handle.join();
    let _ = stderr_handle.join();

    Ok(status)
}

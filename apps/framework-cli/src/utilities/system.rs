//! System utilities
use log::{debug, error, info, warn};
use std::fmt::Debug;
use std::time::Duration;
use std::{
    path::{Path, PathBuf},
    process::Command,
};
use tokio::process::Child;
use tokio::select;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Instant};

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum KillProcessError {
    #[error("Failed to kill process")]
    Kill(#[from] std::io::Error),

    #[error("Child has no process ID")]
    ProcessNotFound,
}

pub fn copy_directory(source: &PathBuf, destination: &PathBuf) -> Result<(), std::io::Error> {
    //! Recursively copy a directory from source to destination.
    let mut command = Command::new("cp");
    command.arg("-r");
    command.arg(source);
    command.arg(destination);

    command.output()?;

    Ok(())
}

pub fn file_name_contains(path: &Path, arbitry_string: &str) -> bool {
    path.file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .contains(arbitry_string)
}

pub async fn kill_child(child: &Child) -> Result<(), KillProcessError> {
    let id = match child.id() {
        Some(id) => id,
        None => return Err(KillProcessError::ProcessNotFound),
    };

    let mut kill = tokio::process::Command::new("kill")
        .args(["-s", "SIGTERM", &id.to_string()])
        .spawn()?;

    kill.wait().await?;

    Ok(())
}

pub struct RestartingProcess {
    monitor_task: JoinHandle<()>,
    kill: tokio::sync::oneshot::Sender<()>,
}

pub type StartChildFn<E> = Box<dyn Fn() -> Result<Child, E> + Send + Sync>;

impl RestartingProcess {
    pub fn create<E: Debug + 'static>(
        process_id: String,
        start: StartChildFn<E>,
    ) -> Result<RestartingProcess, E> {
        let child = start()?;
        let (sender, mut receiver) = tokio::sync::oneshot::channel::<()>();

        Ok(RestartingProcess {
            monitor_task: tokio::task::spawn(async move {
                let mut child = child;
                const INITIAL_DELAY_MS: u64 = 1000;
                const MAX_DELAY_MS: u64 = 60_000; // 1 minute
                const MIN_RUNTIME_FOR_RESET: Duration = Duration::from_secs(10); // Process must run 10s to reset backoff
                let mut delay_ms: u64 = INITIAL_DELAY_MS;
                let mut process_start_time = Instant::now();

                loop {
                    select! {
                        _ = &mut receiver => {
                            info!("Received kill signal, stopping process monitor for {}", process_id);
                            break;
                        }
                        exit_status_result = child.wait() => {
                            let process_runtime = process_start_time.elapsed();
                            match exit_status_result {
                                Ok(exit_status) => {
                                    if exit_status.success() {
                                        info!("Process {} exited successfully after running for {:?}", process_id, process_runtime);
                                    } else {
                                        warn!("Process {} exited with non-zero status: {:?} after running for {:?}", process_id, exit_status, process_runtime);
                                    }
                                }
                                Err(e) => {
                                    error!("Error waiting for process {}: {:?} after running for {:?}", process_id, e, process_runtime);
                                }
                            }

                            // Set initial delay based on whether the previous process ran long enough
                            if process_runtime >= MIN_RUNTIME_FOR_RESET {
                                debug!("Previous process ran for {:?}, resetting backoff delay", process_runtime);
                                delay_ms = INITIAL_DELAY_MS;
                            } else {
                                delay_ms = (delay_ms * 2).min(MAX_DELAY_MS);
                            }

                            'restart: loop {
                                debug!("Delaying {}ms before restarting {}", delay_ms, process_id);
                                sleep(Duration::from_millis(delay_ms)).await;

                                info!("Attempting to restart process {}...", process_id);
                                match start() {
                                    Ok(new_child) => {
                                        info!("Process {} restarted successfully", process_id);
                                        process_start_time = Instant::now();
                                        child = new_child;
                                        break 'restart;
                                    }
                                    Err(e) => {
                                        error!("Failed to restart process {}: {:?}", process_id, e);
                                        delay_ms = (delay_ms * 2).min(MAX_DELAY_MS);
                                    }
                                }
                            }
                        }
                    }
                }
                let _ = kill_child(&child).await;
            }),
            kill: sender,
        })
    }

    pub async fn stop(self) {
        let _ = self.kill.send(());

        if let Err(e) = self.monitor_task.await {
            error!("Error waiting for monitor task to complete: {:?}", e)
        }
    }
}

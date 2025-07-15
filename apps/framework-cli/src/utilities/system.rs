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
use tokio::time::sleep;

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
    pub fn create<E: Debug + 'static>(start: StartChildFn<E>) -> Result<RestartingProcess, E> {
        let child = start()?;
        let (sender, mut receiver) = tokio::sync::oneshot::channel::<()>();

        Ok(RestartingProcess {
            monitor_task: tokio::task::spawn(async move {
                let mut child = child;
                loop {
                    select! {
                        _ = &mut receiver => {
                            info!("Received kill signal, stopping process monitor");
                            break;
                        }
                        exit_status_result = child.wait() => {
                            match exit_status_result {
                                Ok(exit_status) => {
                                    if exit_status.success() {
                                        info!("Process exited successfully");
                                    } else {
                                        warn!("Process exited with non-zero status: {:?}", exit_status);
                                    }
                                }
                                Err(e) => {
                                    error!("Error waiting for process: {:?}", e);
                                }
                            }
                            'restart: loop {
                                debug!("Delaying 1 second before restarting");
                                sleep(Duration::from_secs(1)).await;

                                info!("Attempting to restart process...");
                                match start() {
                                    Ok(new_child) => {
                                        info!("Process restarted successfully");
                                        child = new_child;
                                        break 'restart;
                                    }
                                    Err(e) => {
                                        error!("Failed to restart process: {:?}", e);
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

    pub async fn stop(self) -> () {
        let _ = self.kill.send(());

        if let Err(e) = self.monitor_task.await {
            error!("Error waiting for monitor task to complete: {:?}", e)
        }
    }
}

//! System utilities
use std::{
    path::{Path, PathBuf},
    process::Command,
};
use tokio::process::Child;

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
        .args(&["-s", "SIGTERM", &id.to_string()])
        .spawn()?;

    kill.wait().await?;

    Ok(())
}
